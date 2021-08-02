use std::io;
use std::marker::PhantomData;
use std::net::SocketAddr;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures::SinkExt;
use futures_core::{
    ready,
    stream::{Stream, TryStream},
};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpStream;
use tokio_stream::wrappers::TcpListenerStream;
use tokio_stream::StreamExt;
use tokio_util::codec::Framed;
use tower::{Service, ServiceBuilder, ServiceExt};

use crate::codec::{DefaultMakeCodec, MakeCodec};
use crate::context::MsgContext;
use crate::message::Message;
use crate::protocol::TMessageType;
use crate::{ApplicationError, ApplicationErrorKind, ApplicationResult};

#[async_trait::async_trait]
pub trait Listenable {
    type Conn: AsyncRead + AsyncWrite + Send + Unpin + 'static;
    type Stream: Stream<Item = io::Result<Self::Conn>> + Unpin;

    async fn bind(&self) -> io::Result<Self::Stream>;
}

#[async_trait::async_trait]
impl Listenable for SocketAddr {
    type Conn = TcpStream;
    type Stream = TcpListenerStream;

    async fn bind(&self) -> io::Result<Self::Stream> {
        let listener = tokio::net::TcpListener::bind(self).await?;
        Ok(TcpListenerStream::new(listener))
    }
}

#[cfg(unix)]
#[async_trait::async_trait]
impl Listenable for std::path::PathBuf {
    type Conn = tokio::net::UnixStream;
    type Stream = tokio_stream::wrappers::UnixListenerStream;

    async fn bind(&self) -> io::Result<Self::Stream> {
        let listener = tokio::net::UnixListener::bind(self)?;
        Ok(tokio_stream::wrappers::UnixListenerStream::new(listener))
    }
}

#[pin_project::pin_project]
pub struct Incoming<LS, MC> {
    #[pin]
    listener_stream: LS,
    make_codec: MC,
}

impl<LS, MC> Incoming<LS, MC> {
    #[allow(unused)]
    pub fn new(listener_stream: LS, make_codec: MC) -> Self {
        Self {
            listener_stream,
            make_codec,
        }
    }
}

impl<LS, MC> Stream for Incoming<LS, MC>
where
    LS: TryStream,
    LS::Ok: AsyncRead + AsyncWrite,
    MC: MakeCodec,
{
    type Item = Result<Framed<LS::Ok, MC::Codec>, LS::Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        let mut listener_stream = this.listener_stream;
        let codec = this.make_codec;
        match ready!(listener_stream.as_mut().try_poll_next(cx)) {
            Some(Ok(conn)) => {
                let f = Framed::new(conn, codec.make_codec());
                Poll::Ready(Some(Ok(f)))
            }
            Some(Err(e)) => Poll::Ready(Some(Err(e))),
            None => Poll::Ready(None),
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ServerError {
    #[error("IO error")]
    IO(#[from] io::Error),
}

const DEFAULT_BUFFER: usize = 1e3 as usize; // FIXME
const DEFAULT_CONCURRENCY_LIMIT: usize = 1e3 as usize; // FIXME

pub struct Server<S, Addr, Req, Resp> {
    concurrency_limit: Option<usize>,
    buffer: Option<usize>,
    inner: S,
    _marker: PhantomData<fn(Addr, Req, Resp)>,
}

impl<S, Addr, Req, Resp> Server<S, Addr, Req, Resp>
where
    S: Service<
            (MsgContext, ApplicationResult<Req>),
            Response = Option<(MsgContext, ApplicationResult<Resp>)>,
            Error = crate::Error,
        > + Send
        + 'static,
    S::Future: Send,
{
    pub fn new(inner: S) -> Self {
        Self {
            concurrency_limit: None,
            buffer: None,
            inner,
            _marker: PhantomData,
        }
    }
}

impl<S, Addr, Req, Resp> Server<S, Addr, Req, Resp>
where
    Addr: Listenable,
    S: Service<
            (MsgContext, ApplicationResult<Req>),
            Response = Option<(MsgContext, ApplicationResult<Resp>)>,
            Error = crate::Error,
        > + Send
        + 'static,
    S::Future: Send,
    Req: Send + Message + 'static,
    Resp: Send + Message + 'static,
{
    pub async fn serve(self, addr: Addr) -> Result<(), ServerError> {
        let listen_stream = addr.bind().await?;
        let make_codec = DefaultMakeCodec::<Resp, Req>::new();
        let mut incoming = Incoming::new(listen_stream, make_codec);

        let buffer = self.buffer.unwrap_or(DEFAULT_BUFFER);
        let concurrency_limit = self.concurrency_limit.unwrap_or(DEFAULT_CONCURRENCY_LIMIT);
        let service = ServiceBuilder::new()
            .buffer(buffer)
            .concurrency_limit(concurrency_limit)
            .service(self.inner);

        loop {
            match incoming.try_next().await? {
                Some(mut ts) => {
                    let mut service = service.clone();
                    tokio::spawn(async move {
                        loop {
                            match ts.try_next().await {
                                Ok(Some(req)) => {
                                    let ready_service = match service.ready().await {
                                        Ok(svc) => svc,
                                        Err(e) => {
                                            log::error!("service not ready error: {:?}", e);
                                            return;
                                        }
                                    };

                                    let mut cx = req.0.clone();
                                    match ready_service.call(req).await {
                                        Ok(Some((cx, resp))) => {
                                            if let Err(e) = ts.send((cx, resp)).await {
                                                log::error!("send reply back error: {}", e);
                                                return;
                                            }
                                        }
                                        Ok(None) => {
                                            // oneway does not need response
                                        }
                                        Err(e) => {
                                            // if oneway, we just return
                                            if cx.identifier.message_type == TMessageType::OneWay {
                                                return;
                                            }
                                            // if not oneway, we must send the exception back
                                            cx.identifier.message_type = TMessageType::Exception;
                                            let app_error = ApplicationError::new(
                                                ApplicationErrorKind::Unknown,
                                                e.to_string(),
                                            );
                                            if let Err(e) = ts.send((cx, Err(app_error))).await {
                                                log::error!("send error back error: {}", e);
                                                return;
                                            }
                                        }
                                    }
                                }
                                Ok(None) => {
                                    return;
                                }
                                Err(e) => {
                                    // receive message error
                                    log::error!("error receiving message {}", e);
                                    return;
                                }
                            }
                        }
                    });
                }
                None => return Ok(()),
            }
        }
    }
}
