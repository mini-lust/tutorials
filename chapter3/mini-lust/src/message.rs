use crate::context::MsgContext;
use crate::protocol::{TInputProtocol, TOutputProtocol};

pub trait Message: Sized {
    fn encode<T: TOutputProtocol>(&self, cx: &MsgContext, protocol: &mut T) -> crate::Result<()>;
    fn decode<T: TInputProtocol>(cx: &mut MsgContext, protocol: &mut T) -> crate::Result<Self>;
}
