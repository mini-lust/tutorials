use crate::protocol::TMessageIdentifier;

#[derive(Debug, Default, Eq, PartialEq)]
pub struct MsgContext {
    pub identifier: TMessageIdentifier,
}
