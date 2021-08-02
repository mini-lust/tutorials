#![cfg_attr(feature = "unstable", feature(core_intrinsics))]

use std::collections::HashMap;
use std::sync::Arc;
use std::task::{Context, Poll};

use tower::Service;

pub use codec::DefaultMakeCodec;
pub use connection::DefaultMakeConnection;
pub use connection::SocketOrUnix;
// Export the error
pub use errors::*;
pub use server::{Server, ServerError};
pub use transport::FramedMakeTransport;
use utils::BoxFuture;

pub use crate::client::{Client, ClientBuilder};
use crate::context::MsgContext;
use crate::message::Message;
use crate::protocol::{
    TFieldIdentifier, TInputProtocol, TListIdentifier, TMapIdentifier, TMessageType,
    TOutputProtocol, TStructIdentifier, TType,
};

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
mod utils;

/// Generated Named Structs

/// Generated(manually) User struct
#[derive(Debug, Clone, Default, PartialEq)]
pub struct User {
    pub user_id: i32,
    pub user_name: String,
    pub is_male: bool,

    pub extra: Option<HashMap<String, String>>,
}

/// Generated(manually) GetUserRequest struct
#[derive(Debug, Clone, Default, PartialEq)]
pub struct GetUserRequest {
    pub user_id: i32,
    pub user_name: String,
    pub is_male: bool,
}

/// Generated(manually) GetUserResponse struct
#[derive(Debug, Clone, Default, PartialEq)]
pub struct GetUserResponse {
    pub users: Vec<User>,
}

/// Generated Anonymous Structs

/// Generated(manually) ItemService GetUser argument
/// Named as "Anonymous{:ServiceName}{:Method}Args"
#[derive(Debug, Clone, Default, PartialEq)]
pub struct AnonymousItemServiceGetUserArgs {
    pub req: GetUserRequest,
    pub shuffle: bool,
}

/// Generated(manually) ItemService GetUser result
/// Named as "Anonymous{:ServiceName}{:Method}Result"
#[derive(Debug, Clone, PartialEq)]
pub enum AnonymousItemServiceGetUserResult {
    Success(GetUserResponse),
    // User defined exceptions here
}

/// Generated(manually) ItemService argument
/// Named as "Anonymous{:ServiceName}Request"
#[derive(Debug, Clone, PartialEq)]
pub enum AnonymousItemServiceRequest {
    GetUser(AnonymousItemServiceGetUserArgs),
}

/// Generated(manually) ItemService result
/// Named as "Anonymous{:ServiceName}Response"
#[derive(Debug, Clone, PartialEq)]
pub enum AnonymousItemServiceResponse {
    GetUser(AnonymousItemServiceGetUserResult),
}

/// Generated Message impl

/// Generated(manually) impl Message for User
impl Message for User {
    fn encode<T: TOutputProtocol>(&self, _cx: &MsgContext, protocol: &mut T) -> Result<()> {
        protocol.write_struct_begin(&TStructIdentifier {
            name: "User".to_string(),
        })?;

        // user_id
        protocol.write_field_begin(&TFieldIdentifier {
            name: Some("user_id".to_string()),
            field_type: TType::I32,
            id: Some(1),
        })?;
        protocol.write_i32(self.user_id)?;
        protocol.write_field_end()?;

        // user_name
        protocol.write_field_begin(&TFieldIdentifier {
            name: Some("user_name".to_string()),
            field_type: TType::String,
            id: Some(2),
        })?;
        protocol.write_string(&self.user_name)?;
        protocol.write_field_end()?;

        // is_male
        protocol.write_field_begin(&TFieldIdentifier {
            name: Some("is_male".to_string()),
            field_type: TType::Bool,
            id: Some(3),
        })?;
        protocol.write_bool(self.is_male)?;
        protocol.write_field_end()?;

        // extra
        if let Some(extra) = self.extra.as_ref() {
            protocol.write_field_begin(&TFieldIdentifier {
                name: Some("extra".to_string()),
                field_type: TType::Map,
                id: Some(10),
            })?;
            protocol.write_map_begin(&TMapIdentifier {
                key_type: Some(TType::String),
                value_type: Some(TType::String),
                size: extra.len() as i32,
            })?;

            // key-values
            for (k, v) in extra.iter() {
                protocol.write_string(k)?;
                protocol.write_string(v)?;
            }

            protocol.write_map_end()?;
            protocol.write_field_end()?;
        }

        protocol.write_field_stop()?;
        protocol.write_struct_end()?;
        Ok(())
    }

    fn decode<T: TInputProtocol>(_cx: &mut MsgContext, protocol: &mut T) -> Result<Self> {
        let mut output = Self::default();
        protocol.read_struct_begin()?;

        loop {
            let ident = protocol.read_field_begin()?;
            if ident.field_type == TType::Stop {
                break;
            }
            match ident.id {
                Some(1) => {
                    ttype_comparing(ident.field_type, TType::I32)?;

                    // read i32
                    let content = protocol.read_i32()?;
                    output.user_id = content;
                }
                Some(2) => {
                    ttype_comparing(ident.field_type, TType::String)?;

                    // read string
                    let content = protocol.read_string()?;
                    output.user_name = content;
                }
                Some(3) if ident.field_type == TType::Bool => {
                    ttype_comparing(ident.field_type, TType::Bool)?;

                    // read bool
                    let content = protocol.read_bool()?;
                    output.is_male = content;
                }
                Some(10) if ident.field_type == TType::Map => {
                    ttype_comparing(ident.field_type, TType::Map)?;

                    // read optional map
                    let map_ident = protocol.read_map_begin()?;
                    let mut map = HashMap::new();
                    for _ in 0..map_ident.size {
                        let key = protocol.read_string()?;
                        let value = protocol.read_string()?;
                        map.insert(key, value);
                    }
                    output.extra = Some(map);
                    protocol.read_map_end()?;
                }
                _ => {
                    protocol.skip(ident.field_type)?;
                }
            }
            protocol.read_field_end()?;
        }

        protocol.read_struct_end()?;
        Ok(output)
    }
}

/// Generated(manually) impl Message for GetUserRequest
impl Message for GetUserRequest {
    fn encode<T: TOutputProtocol>(&self, _cx: &MsgContext, protocol: &mut T) -> Result<()> {
        protocol.write_struct_begin(&TStructIdentifier {
            name: "GetUserRequest".to_string(),
        })?;

        // user_id
        protocol.write_field_begin(&TFieldIdentifier {
            name: Some("user_id".to_string()),
            field_type: TType::I32,
            id: Some(1),
        })?;
        protocol.write_i32(self.user_id)?;
        protocol.write_field_end()?;

        // user_name
        protocol.write_field_begin(&TFieldIdentifier {
            name: Some("user_name".to_string()),
            field_type: TType::String,
            id: Some(2),
        })?;
        protocol.write_string(&self.user_name)?;
        protocol.write_field_end()?;

        // is_male
        protocol.write_field_begin(&TFieldIdentifier {
            name: Some("is_male".to_string()),
            field_type: TType::Bool,
            id: Some(3),
        })?;
        protocol.write_bool(self.is_male)?;
        protocol.write_field_end()?;

        protocol.write_field_stop()?;
        protocol.write_struct_end()?;
        Ok(())
    }

    fn decode<T: TInputProtocol>(_cx: &mut MsgContext, protocol: &mut T) -> Result<Self> {
        let mut output = Self::default();
        protocol.read_struct_begin()?;

        loop {
            let ident = protocol.read_field_begin()?;
            if ident.field_type == TType::Stop {
                break;
            }
            match ident.id {
                Some(1) => {
                    ttype_comparing(ident.field_type, TType::I32)?;

                    // read i32
                    let content = protocol.read_i32()?;
                    output.user_id = content;
                }
                Some(2) => {
                    ttype_comparing(ident.field_type, TType::String)?;

                    // read string
                    let content = protocol.read_string()?;
                    output.user_name = content;
                }
                Some(3) => {
                    ttype_comparing(ident.field_type, TType::Bool)?;

                    // read bool
                    let content = protocol.read_bool()?;
                    output.is_male = content;
                }
                _ => {
                    protocol.skip(ident.field_type)?;
                }
            }
            protocol.read_field_end()?;
        }

        protocol.read_struct_end()?;
        Ok(output)
    }
}

/// Generated(manually) impl Message for GetUserResponse
impl Message for GetUserResponse {
    fn encode<T: TOutputProtocol>(&self, cx: &MsgContext, protocol: &mut T) -> Result<()> {
        protocol.write_struct_begin(&TStructIdentifier {
            name: "GetUserResponse".to_string(),
        })?;

        // users
        protocol.write_field_begin(&TFieldIdentifier {
            name: Some("users".to_string()),
            field_type: TType::List,
            id: Some(1),
        })?;

        protocol.write_list_begin(&TListIdentifier {
            element_type: TType::Struct,
            size: self.users.len() as i32,
        })?;
        for user in self.users.iter() {
            user.encode(cx, protocol)?;
        }
        protocol.write_list_end()?;

        protocol.write_field_end()?;

        protocol.write_field_stop()?;
        protocol.write_struct_end()?;
        Ok(())
    }

    fn decode<T: TInputProtocol>(cx: &mut MsgContext, protocol: &mut T) -> Result<Self> {
        let mut output = Self::default();
        protocol.read_struct_begin()?;

        loop {
            let ident = protocol.read_field_begin()?;
            if ident.field_type == TType::Stop {
                break;
            }
            match ident.id {
                Some(1) => {
                    ttype_comparing(ident.field_type, TType::List)?;

                    // read list
                    let list_ident = protocol.read_list_begin()?;
                    output.users = Vec::with_capacity(list_ident.size as usize);
                    for _ in 0..list_ident.size {
                        let user = User::decode(cx, protocol)?;
                        output.users.push(user);
                    }
                    protocol.read_list_end()?;
                }
                _ => {
                    protocol.skip(ident.field_type)?;
                }
            }
            protocol.read_field_end()?;
        }

        protocol.read_struct_end()?;
        Ok(output)
    }
}

impl Message for AnonymousItemServiceGetUserArgs {
    fn encode<T: TOutputProtocol>(&self, cx: &MsgContext, protocol: &mut T) -> Result<()> {
        protocol.write_struct_begin(&TStructIdentifier {
            name: "AnonymousItemServiceGetUserRequest".to_string(),
        })?;
        protocol.write_field_begin(&TFieldIdentifier {
            name: Some("req".to_string()),
            field_type: TType::Struct,
            id: Some(1),
        })?;
        self.req.encode(cx, protocol)?;
        protocol.write_field_end()?;

        protocol.write_field_begin(&TFieldIdentifier {
            name: Some("shuffle".to_string()),
            field_type: TType::Bool,
            id: Some(2),
        })?;
        protocol.write_bool(self.shuffle)?;
        protocol.write_field_end()?;
        protocol.write_field_stop()?;
        protocol.write_struct_end()?;
        Ok(())
    }

    fn decode<T: TInputProtocol>(cx: &mut MsgContext, protocol: &mut T) -> Result<Self> {
        protocol.read_struct_begin()?;
        if cx.identifier.message_type == TMessageType::Exception {
            // If message type is exception, we should not try decoding
            // AnonymousItemServiceGetUserArgs
            return Err(new_application_error(
                ApplicationErrorKind::Unknown,
                "internal error: try decoding message on exception",
            ));
        }

        let mut output = Self::default();
        loop {
            let ident = protocol.read_field_begin()?;
            if ident.field_type == TType::Stop {
                break;
            }
            match ident.id {
                Some(1) => {
                    ttype_comparing(ident.field_type, TType::Struct)?;

                    // read struct
                    output.req = GetUserRequest::decode(cx, protocol)?;
                }
                Some(2) => {
                    ttype_comparing(ident.field_type, TType::Bool)?;

                    // read bool
                    output.shuffle = protocol.read_bool()?;
                }
                _ => {
                    protocol.skip(ident.field_type)?;
                }
            }
            protocol.read_field_end()?;
        }
        protocol.read_struct_end()?;
        Ok(output)
    }
}

impl Message for AnonymousItemServiceGetUserResult {
    fn encode<T: TOutputProtocol>(&self, cx: &MsgContext, protocol: &mut T) -> Result<()> {
        protocol.write_struct_begin(&TStructIdentifier {
            name: "AnonymousItemServiceGetUserResult".to_string(),
        })?;
        match self {
            AnonymousItemServiceGetUserResult::Success(c) => {
                protocol.write_field_begin(&TFieldIdentifier {
                    name: Some("GetUserResponse".to_string()),
                    field_type: TType::Struct,
                    id: Some(0),
                })?;
                c.encode(cx, protocol)?;
                protocol.write_field_end()?;
            } // Exception if some
        }
        protocol.write_field_stop()?;
        protocol.write_struct_end()?;
        Ok(())
    }

    fn decode<T: TInputProtocol>(cx: &mut MsgContext, protocol: &mut T) -> Result<Self> {
        protocol.read_struct_begin()?;
        let ident = protocol.read_field_begin()?;
        // There must be only one field
        let output = match ident.id {
            Some(0) => {
                let resp = GetUserResponse::decode(cx, protocol)?;
                protocol.read_field_end()?;
                Self::Success(resp)
            }
            _ => {
                return Err(new_protocol_error(
                    ProtocolErrorKind::InvalidData,
                    "unexpected result field",
                ));
            }
        };
        // Read field stop. TODO: check it
        protocol.read_field_begin()?;
        protocol.read_field_end()?;

        protocol.read_struct_end()?;
        Ok(output)
    }
}

impl Message for AnonymousItemServiceRequest {
    // Encode just forward encode to inner
    fn encode<T: TOutputProtocol>(&self, cx: &MsgContext, protocol: &mut T) -> Result<()> {
        match self {
            AnonymousItemServiceRequest::GetUser(req) => {
                req.encode(cx, protocol)?;
            } // Other patterns if there are other methods
        }
        Ok(())
    }

    // Decode just forward decode to inner with information inside the context
    fn decode<T: TInputProtocol>(cx: &mut MsgContext, protocol: &mut T) -> Result<Self> {
        let resp = match cx.identifier.name.as_ref() {
            "GetUser" => Ok(Self::GetUser(AnonymousItemServiceGetUserArgs::decode(
                cx, protocol,
            )?)),
            // Other patterns if there are other methods
            method @ _ => Err(new_application_error(
                ApplicationErrorKind::UnknownMethod,
                format!("unknown method {}", method),
            )),
        };
        resp
    }
}

impl Message for AnonymousItemServiceResponse {
    fn encode<T: TOutputProtocol>(&self, cx: &MsgContext, protocol: &mut T) -> Result<()> {
        match self {
            AnonymousItemServiceResponse::GetUser(r) => r.encode(cx, protocol),
        }
    }

    fn decode<T: TInputProtocol>(cx: &mut MsgContext, protocol: &mut T) -> Result<Self> {
        match cx.identifier.name.as_ref() {
            "GetUser" => Ok(Self::GetUser(AnonymousItemServiceGetUserResult::decode(
                cx, protocol,
            )?)),
            _ => Err(new_protocol_error(
                ProtocolErrorKind::InvalidData,
                "unrecognized method name",
            )),
        }
    }
}

#[inline(always)]
fn ttype_comparing(x: TType, y: TType) -> crate::Result<()> {
    if x != y {
        return Err(new_protocol_error(
            ProtocolErrorKind::InvalidData,
            format!("invalid ttype: {}, expect: {}", x, y),
        ));
    }
    Ok(())
}

pub struct ItemServiceClientBuilder {
    client_builder: ClientBuilder<DefaultMakeCodec<AnonymousItemServiceRequest, AnonymousItemServiceResponse>>,
}

impl ItemServiceClientBuilder {
    pub fn new(target: SocketOrUnix) -> Self {
        let client_builder = ClientBuilder::new(target);
        Self { client_builder }
    }

    pub fn build(self) -> ItemServiceClient {
        ItemServiceClient::new(self.client_builder.build())
    }
}

#[derive(Clone)]
pub struct ItemServiceClient {
    inner_client: Client<AnonymousItemServiceRequest, AnonymousItemServiceResponse>,
}

impl ItemServiceClient {
    pub fn new(inner: Client<AnonymousItemServiceRequest, AnonymousItemServiceResponse>) -> Self {
        Self {
            inner_client: inner,
        }
    }
}

impl ItemServiceClient {
    pub async fn get_user(
        &mut self,
        req: GetUserRequest,
        shuffle: bool,
    ) -> Result<AnonymousItemServiceGetUserResult> {
        let anonymous_request =
            AnonymousItemServiceRequest::GetUser(AnonymousItemServiceGetUserArgs { req, shuffle });
        let resp = self.inner_client.call("GetUser", anonymous_request).await?;

        #[allow(irrefutable_let_patterns)]
        if let AnonymousItemServiceResponse::GetUser(r) = resp {
            return Ok(r);
        }
        Err(new_application_error(
            ApplicationErrorKind::Unknown,
            "unable to get response",
        ))
    }
}

#[async_trait::async_trait]
pub trait ItemService {
    async fn get_user(
        &self,
        req: GetUserRequest,
        shuffle: bool,
    ) -> ApplicationResult<AnonymousItemServiceGetUserResult>;
}

pub struct ItemServiceServer<S> {
    inner: Arc<S>,
}

impl<S> ItemServiceServer<S> {
    pub fn new(inner: S) -> Self {
        Self {
            inner: Arc::new(inner),
        }
    }
}

impl<S> Service<(MsgContext, ApplicationResult<AnonymousItemServiceRequest>)>
    for ItemServiceServer<S>
where
    S: ItemService + Send + Sync + 'static,
{
    // Option since we may or may not return due to call or oneway
    type Response = Option<(MsgContext, ApplicationResult<AnonymousItemServiceResponse>)>;
    type Error = crate::Error;
    type Future = BoxFuture<Self::Response, Self::Error>;

    fn poll_ready(&mut self, _cx: &mut Context) -> Poll<std::result::Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(
        &mut self,
        req: (MsgContext, ApplicationResult<AnonymousItemServiceRequest>),
    ) -> Self::Future {
        let inner = self.inner.clone();
        Box::pin(async move {
            let (mut cx, req) = req;
            match req {
                Ok(AnonymousItemServiceRequest::GetUser(r)) => {
                    let ret = inner.get_user(r.req, r.shuffle).await;
                    match ret {
                        Ok(r) => {
                            cx.identifier.message_type = TMessageType::Reply;
                            Ok(Some((cx, Ok(AnonymousItemServiceResponse::GetUser(r)))))
                        }
                        Err(e) => {
                            cx.identifier.message_type = TMessageType::Exception;
                            Ok(Some((cx, Err(e))))
                        }
                    }
                }
                Err(e) => {
                    log::error!("unexpected client error: {}", e);
                    Err(new_application_error(
                        ApplicationErrorKind::Unknown,
                        "unexpected client error",
                    ))
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::binary::{TBinaryInputProtocol, TBinaryOutputProtocol};
    use crate::context::MsgContext;
    use crate::message::Message;
    use crate::protocol::{TMessageIdentifier, TMessageType};
    use crate::{
        AnonymousItemServiceGetUserArgs, AnonymousItemServiceGetUserResult,
        AnonymousItemServiceRequest, GetUserRequest, GetUserResponse,
    };

    use super::User;

    fn compare_encode_decode<T>(mut cx: MsgContext, input: T)
    where
        T: Message + std::cmp::PartialEq + std::fmt::Debug,
    {
        let mut buffer = bytes::BytesMut::new();

        // Encode
        let mut output_protocol = TBinaryOutputProtocol::new(&mut buffer, true);
        input.encode(&cx, &mut output_protocol).unwrap();

        // Decode
        let mut input_protocol = TBinaryInputProtocol::new(buffer.as_ref(), true);
        let decoded = T::decode(&mut cx, &mut input_protocol).unwrap();

        assert_eq!(
            decoded, input,
            "message after encoding and decoding must equal"
        );
    }

    fn user() -> User {
        let mut extra = HashMap::new();
        extra.insert("test_key1".to_string(), "test_val1".to_string());
        extra.insert("test_key2".to_string(), "test_val2".to_string());
        let user = User {
            user_id: 7,
            user_name: "ihciah".to_string(),
            is_male: true,
            extra: Some(extra),
        };
        user
    }

    fn get_user_request() -> GetUserRequest {
        GetUserRequest {
            user_id: 7,
            user_name: "ChiHai".to_string(),
            is_male: true,
        }
    }

    fn get_user_response() -> GetUserResponse {
        GetUserResponse {
            users: vec![user(), user()],
        }
    }

    #[test]
    fn test_user_encode_decode() {
        compare_encode_decode(Default::default(), user());
    }

    #[test]
    fn test_get_user_request_encode_decode() {
        compare_encode_decode(Default::default(), get_user_request());
    }

    #[test]
    fn test_get_user_response_encode_decode() {
        compare_encode_decode(Default::default(), get_user_response());
    }

    #[test]
    fn test_anonymous_item_service_get_user_args_encode_decode() {
        compare_encode_decode(
            Default::default(),
            AnonymousItemServiceGetUserArgs {
                req: get_user_request(),
                shuffle: true,
            },
        );
    }

    #[test]
    fn test_anonymous_item_service_get_user_result_encode_decode() {
        compare_encode_decode(
            Default::default(),
            AnonymousItemServiceGetUserResult::Success(get_user_response()),
        );
    }

    #[test]
    fn test_anonymous_item_service_request_encode_decode() {
        let cx = MsgContext {
            identifier: TMessageIdentifier {
                name: "GetUser".to_string(),
                message_type: TMessageType::Call,
                sequence_number: 1,
            },
            ..MsgContext::default()
        };
        compare_encode_decode(
            cx,
            AnonymousItemServiceRequest::GetUser(AnonymousItemServiceGetUserArgs {
                req: get_user_request(),
                shuffle: true,
            }),
        );
    }
}
