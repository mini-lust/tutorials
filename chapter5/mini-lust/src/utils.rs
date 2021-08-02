use std::future::Future;
use std::pin::Pin;

pub type BoxFuture<T, E> = Pin<Box<dyn Future<Output = std::result::Result<T, E>> + Send>>;
