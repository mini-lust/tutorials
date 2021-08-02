use std::task::{Context, Poll};

use futures::sink::SinkExt;
use futures::stream::TryStreamExt;
use tokio::io::{AsyncRead, AsyncWrite};
use tower::buffer::{Buffer, BufferLayer};
use tower::util::BoxService;
use tower::{Layer, Service, ServiceExt};

use crate::codec::MakeCodec;
use crate::connection::SocketOrUnix;
use crate::context::MsgContext;
use crate::protocol::{TMessageIdentifier, TMessageType};
use crate::utils::BoxFuture;
use crate::{ApplicationResult, DefaultMakeCodec, DefaultMakeConnection, FramedMakeTransport};

pub struct ClientBuilder<MCC> {
    target: SocketOrUnix,
    make_codec: MCC,
}

impl<E, D> ClientBuilder<DefaultMakeCodec<E, D>> {
    pub fn new(target: SocketOrUnix) -> Self {
        Self {
            target,
            make_codec: DefaultMakeCodec::new(),
        }
    }
}

impl<MCC> ClientBuilder<MCC> {
    pub fn make_codec(self, make_codec: MCC) -> Self {
        Self {
            target: self.target,
            make_codec,
        }
    }
}

const DEFAULT_BUFFER: usize = usize::MAX >> 3;

impl<MCC, Req, Resp> ClientBuilder<MCC>
where
    Req: Send + 'static,
    Resp: Send + 'static,
    MCC: MakeCodec<
            EncodeItem = (MsgContext, ApplicationResult<Req>),
            DecodeItem = (MsgContext, ApplicationResult<Resp>),
            Error = crate::Error,
        > + Send
        + 'static,
    MCC::Codec: Send + 'static,
{
    pub fn build(self) -> Client<Req, Resp> {
        let make_connection = DefaultMakeConnection;
        let make_codec = self.make_codec;
        let transport_client = TransportClient::new(make_connection, make_codec);
        let inner = Buffer::new(BoxService::new(transport_client), DEFAULT_BUFFER);
        Client {
            inner,
            target: self.target,
        }
    }
}

#[derive(Clone)]
pub struct Client<Req, Resp> {
    inner: Buffer<
        BoxService<
            (MsgContext, ApplicationResult<Req>),
            Option<(MsgContext, ApplicationResult<Resp>)>,
            crate::Error,
        >,
        (MsgContext, ApplicationResult<Req>),
    >,
    target: SocketOrUnix,
}

impl<Req, Resp> Client<Req, Resp> {
    /// Call with method and Req and returns Result<Resp>
    pub async fn call(&mut self, method: &'static str, req: Req) -> crate::Result<Resp> {
        let context = MsgContext {
            identifier: TMessageIdentifier {
                name: method.to_string(),
                message_type: TMessageType::Call,

                ..TMessageIdentifier::default()
            },
            target: Some(self.target.clone()),
        };
        let req = (context, Ok(req));
        // Option<(MsgContext, ApplicationResult<Resp>)>
        let resp = self.inner.ready().await?.call(req).await?;
        resp.expect("returning resp is expected")
            .1
            .map_err(Into::into)
    }

    pub async fn oneway(&mut self, method: &'static str, req: Req) -> crate::Result<()> {
        let context = MsgContext {
            identifier: TMessageIdentifier {
                name: method.to_string(),
                message_type: TMessageType::OneWay,

                ..TMessageIdentifier::default()
            },
            target: Some(self.target.clone()),
        };
        let req = (context, Ok(req));
        self.inner.ready().await?.call(req).await?;
        Ok(())
    }
}

pub(crate) struct TransportClient<MCN, MCC> {
    make_transport: FramedMakeTransport<MCC, MCN>,
    seq_id: i32,
}

impl<MCN, MCC> TransportClient<MCN, MCC> {
    #[allow(unused)]
    pub fn new(make_connection: MCN, make_codec: MCC) -> Self {
        Self {
            make_transport: FramedMakeTransport::new(make_connection, make_codec),
            seq_id: 1,
        }
    }
}

/// Call with (MsgContext, ApplicationResult<Req>) returns
/// Option<(MsgContext, ApplicationResult<Resp>)>.
impl<MCN, MCC, Req, Resp> Service<(MsgContext, ApplicationResult<Req>)>
    for TransportClient<MCN, MCC>
where
    MCN: Service<SocketOrUnix> + Clone + Send + 'static,
    MCN::Response: AsyncRead + AsyncWrite + Unpin + Send,
    MCN::Error: std::error::Error + Send + Sync + 'static,
    MCN::Future: Send,
    Req: Send + 'static,
    Resp: Send,
    MCC: MakeCodec<
        EncodeItem = (MsgContext, ApplicationResult<Req>),
        DecodeItem = (MsgContext, ApplicationResult<Resp>),
        Error = crate::Error,
    >,
    MCC::Codec: Send + 'static,
    crate::Error: From<<MCN as tower::Service<SocketOrUnix>>::Error>,
{
    // If oneway, the Response is None
    type Response = Option<(MsgContext, ApplicationResult<Resp>)>;
    type Error = crate::Error;
    type Future = BoxFuture<Self::Response, Self::Error>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.make_transport.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, (mut cx, req): (MsgContext, ApplicationResult<Req>)) -> Self::Future {
        // TODO: use service discovery
        let target = cx
            .target
            .clone()
            .expect("unable to retrieve target from context");
        let transport_fut = self.make_transport.call(target);

        self.seq_id += 1;
        cx.identifier.sequence_number = self.seq_id;
        let oneway = cx.identifier.message_type == TMessageType::OneWay;
        Box::pin(async move {
            let mut transport = transport_fut.await?;
            transport.send((cx, req)).await?;
            if oneway {
                return Ok(None);
            }
            transport.try_next().await.map_err(Into::into)
        })
    }
}
