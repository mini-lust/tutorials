use std::task::{Context, Poll};

use tokio_util::codec::Framed;
use tower::make::MakeConnection;
use tower::Service;

use crate::codec::MakeCodec;
use crate::utils::BoxFuture;

pub struct FramedMakeTransport<MCC, MCN> {
    make_connection: MCN,
    make_codec: MCC,
}

impl<MCC, MCN> FramedMakeTransport<MCC, MCN> {
    #[allow(unused)]
    pub fn new(make_connection: MCN, make_codec: MCC) -> Self {
        Self {
            make_connection,
            make_codec,
        }
    }
}

impl<MCC, MCN, TG> Service<TG> for FramedMakeTransport<MCC, MCN>
where
    MCN: MakeConnection<TG>,
    MCN::Future: 'static + Send,
    MCC: MakeCodec,
    MCC::Codec: 'static + Send,
{
    type Response = Framed<MCN::Connection, MCC::Codec>;
    type Error = MCN::Error;
    type Future = BoxFuture<Self::Response, Self::Error>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.make_connection.poll_ready(cx)
    }

    fn call(&mut self, target: TG) -> Self::Future {
        let conn_fut = self.make_connection.make_connection(target);
        let codec = self.make_codec.make_codec();
        Box::pin(async move {
            let conn = conn_fut.await?;
            Ok(Framed::new(conn, codec))
        })
    }
}
