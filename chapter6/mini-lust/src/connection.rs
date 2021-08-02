use std::io;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures_util::FutureExt;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tower::Service;

use crate::utils::BoxFuture;

pub trait Io: AsyncWrite + AsyncRead + Send + 'static {}

impl<T> Io for T where T: AsyncRead + AsyncWrite + Send + 'static {}

pub struct BoxedIo(Pin<Box<dyn Io>>);

impl BoxedIo {
    pub fn new<I: Io>(io: I) -> BoxedIo {
        BoxedIo(Box::pin(io))
    }
}

impl AsyncRead for BoxedIo {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        Pin::new(&mut self.0).poll_read(cx, buf)
    }
}

impl AsyncWrite for BoxedIo {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.0).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.0).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.0).poll_shutdown(cx)
    }
}

#[derive(Debug, Clone)]
pub struct DefaultMakeConnection;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum SocketOrUnix {
    Socket(SocketAddr),
    #[cfg(unix)]
    Unix(PathBuf),
}

impl Service<SocketAddr> for DefaultMakeConnection {
    type Response = tokio::net::TcpStream;
    type Error = std::io::Error;
    type Future = BoxFuture<Self::Response, Self::Error>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: SocketAddr) -> Self::Future {
        Box::pin(tokio::net::TcpStream::connect(req))
    }
}

#[cfg(unix)]
impl Service<PathBuf> for DefaultMakeConnection {
    type Response = tokio::net::UnixStream;
    type Error = std::io::Error;
    type Future = BoxFuture<Self::Response, Self::Error>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: PathBuf) -> Self::Future {
        Box::pin(tokio::net::UnixStream::connect(req))
    }
}

impl Service<SocketOrUnix> for DefaultMakeConnection {
    type Response = BoxedIo;
    type Error = std::io::Error;
    type Future = BoxFuture<Self::Response, Self::Error>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: SocketOrUnix) -> Self::Future {
        match req {
            SocketOrUnix::Socket(addr) => {
                Box::pin(FutureExt::map(tokio::net::TcpStream::connect(addr), |r| {
                    r.map(BoxedIo::new)
                }))
            }
            #[cfg(unix)]
            SocketOrUnix::Unix(path) => {
                Box::pin(FutureExt::map(tokio::net::UnixStream::connect(path), |r| {
                    r.map(BoxedIo::new)
                }))
            }
        }
    }
}
