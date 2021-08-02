use crate::protocol::TMessageIdentifier;
use crate::connection::SocketOrUnix;

/// MsgContext can only be used across our framework and middleware.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct MsgContext {
    /// Thrift TMessageIdentifier
    pub identifier: TMessageIdentifier,
    /// target
    pub target: Option<SocketOrUnix>,
}
