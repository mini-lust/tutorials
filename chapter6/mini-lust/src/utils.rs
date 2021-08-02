use std::future::Future;
use std::pin::Pin;

pub type BoxFuture<T, E> = Pin<Box<dyn Future<Output = std::result::Result<T, E>> + Send>>;

#[inline(always)]
pub fn ttype_comparing(x: crate::protocol::TType, y: crate::protocol::TType) -> crate::Result<()> {
    if x != y {
        return Err(crate::errors::new_protocol_error(
            crate::errors::ProtocolErrorKind::InvalidData,
            format!("invalid ttype: {}, expect: {}", x, y),
        ));
    }
    Ok(())
}