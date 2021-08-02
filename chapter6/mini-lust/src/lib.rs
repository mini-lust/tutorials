#![cfg_attr(feature = "unstable", feature(core_intrinsics))]

pub use client::{Client, ClientBuilder};
pub use codec::DefaultMakeCodec;
pub use connection::{DefaultMakeConnection, SocketOrUnix};
pub use context::MsgContext;
// Export the error
pub use errors::*;
pub use message::Message;
pub use protocol::{
    TFieldIdentifier, TInputProtocol, TListIdentifier, TMapIdentifier, TMessageType,
    TOutputProtocol, TStructIdentifier, TType,
};
pub use server::{Server, ServerError};
pub use transport::FramedMakeTransport;
pub use types::OrigType;
pub use utils::{ttype_comparing, BoxFuture};

pub type Result<T> = std::result::Result<T, Error>;

mod binary;
mod client;
mod codec;
mod connection;
mod context;
mod errors;
mod message;
mod protocol;
mod server;
mod transport;
mod types;
mod utils;
