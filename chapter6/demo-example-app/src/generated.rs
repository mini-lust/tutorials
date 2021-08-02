use mini_lust_chap6::{OrigType, ProtocolErrorKind};

#[derive(mini_lust_macros::Message, Debug, Clone, Default, PartialEq)]
pub struct User {
    #[mini_lust(field_id = 1, required = "true", field_type = "i32")]
    pub user_id: i32,
    #[mini_lust(field_id = 2, required = "true", field_type = "string")]
    pub user_name: String,
    #[mini_lust(field_id = 3, required = "true", field_type = "bool")]
    pub is_male: bool,

    #[mini_lust(field_id = 10, required = "false", field_type = "map(string, string)")]
    pub extra: Option<::std::collections::BTreeMap<String, String>>,
}

impl OrigType for User {}

/// Generated(manually) GetUserRequest struct
#[derive(mini_lust_macros::Message, Debug, Clone, Default, PartialEq)]
pub struct GetUserRequest {
    #[mini_lust(field_id = 1, field_type = "i32")]
    pub user_id: Option<i32>,
    #[mini_lust(field_id = 2, field_type = "string")]
    pub user_name: Option<String>,
    #[mini_lust(field_id = 3, field_type = "bool")]
    pub is_male: Option<bool>,
}

impl OrigType for GetUserRequest {}

/// Generated(manually) GetUserResponse struct
#[derive(mini_lust_macros::Message, Debug, Clone, Default, PartialEq)]
pub struct GetUserResponse {
    #[mini_lust(field_id = 1, required = "true", field_type = "list(ident(User))")]
    pub users: Vec<User>,
}

impl OrigType for GetUserResponse {}

/// Generated Anonymous Structs

/// Generated(manually) ItemService GetUser argument
/// Named as "Anonymous{:ServiceName}{:Method}Args"
#[derive(mini_lust_macros::Message, Debug, Clone, Default, PartialEq)]
pub struct AnonymousItemServiceGetUserArgs {
    #[mini_lust(field_id = 1, field_type = "ident(GetUserRequest)")]
    pub req: Option<GetUserRequest>,
    #[mini_lust(field_id = 2, field_type = "bool")]
    pub shuffle: Option<bool>,
}

impl OrigType for AnonymousItemServiceGetUserArgs {}

/// Generated(manually) ItemService GetUser result
/// Named as "Anonymous{:ServiceName}{:Method}Result"
#[derive(mini_lust_macros::Message, Debug, Clone, PartialEq)]
pub enum AnonymousItemServiceGetUserResult {
    #[mini_lust(field_id = 1)]
    Success(GetUserResponse),
    // User defined exceptions here
}

impl OrigType for AnonymousItemServiceGetUserResult {}

/// Generated(manually) ItemService argument
/// Named as "Anonymous{:ServiceName}Request"
#[derive(mini_lust_macros::Message, Debug, Clone, PartialEq)]
#[mini_lust(dispatch_only = true)]
pub enum AnonymousItemServiceRequest {
    GetUser(AnonymousItemServiceGetUserArgs),
}

/// Generated(manually) ItemService result
/// Named as "Anonymous{:ServiceName}Response"
#[derive(mini_lust_macros::Message, Debug, Clone, PartialEq)]
#[mini_lust(dispatch_only = true)]
pub enum AnonymousItemServiceResponse {
    GetUser(AnonymousItemServiceGetUserResult),
}

/// Generated Message impl

// /// Generated(manually) impl Message for User
// impl ::mini_lust_chap6::Message for User {
//     fn encode<T: ::mini_lust_chap6::TOutputProtocol>(
//         &self,
//         _cx: &::mini_lust_chap6::MsgContext,
//         protocol: &mut T,
//     ) -> ::mini_lust_chap6::Result<()> {
//         protocol.write_struct_begin(&::mini_lust_chap6::TStructIdentifier {
//             name: "User".to_string(),
//         })?;
//
//         // user_id
//         protocol.write_field_begin(&::mini_lust_chap6::TFieldIdentifier {
//             name: Some("user_id".to_string()),
//             field_type: ::mini_lust_chap6::TType::I32,
//             id: Some(1),
//         })?;
//         protocol.write_i32(self.user_id)?;
//         protocol.write_field_end()?;
//
//         // user_name
//         protocol.write_field_begin(&::mini_lust_chap6::TFieldIdentifier {
//             name: Some("user_name".to_string()),
//             field_type: ::mini_lust_chap6::TType::String,
//             id: Some(2),
//         })?;
//         protocol.write_string(&self.user_name)?;
//         protocol.write_field_end()?;
//
//         // is_male
//         protocol.write_field_begin(&::mini_lust_chap6::TFieldIdentifier {
//             name: Some("is_male".to_string()),
//             field_type: ::mini_lust_chap6::TType::Bool,
//             id: Some(3),
//         })?;
//         protocol.write_bool(self.is_male)?;
//         protocol.write_field_end()?;
//
//         // extra
//         if let Some(extra) = self.extra.as_ref() {
//             protocol.write_field_begin(&::mini_lust_chap6::TFieldIdentifier {
//                 name: Some("extra".to_string()),
//                 field_type: ::mini_lust_chap6::TType::Map,
//                 id: Some(10),
//             })?;
//             protocol.write_map_begin(&::mini_lust_chap6::TMapIdentifier {
//                 key_type: Some(::mini_lust_chap6::TType::String),
//                 value_type: Some(::mini_lust_chap6::TType::String),
//                 size: extra.len() as i32,
//             })?;
//
//             // key-values
//             for (k, v) in extra.iter() {
//                 protocol.write_string(k)?;
//                 protocol.write_string(v)?;
//             }
//
//             protocol.write_map_end()?;
//             protocol.write_field_end()?;
//         }
//
//         protocol.write_field_stop()?;
//         protocol.write_struct_end()?;
//         Ok(())
//     }
//
//     fn decode<T: ::mini_lust_chap6::TInputProtocol>(
//         _cx: &mut ::mini_lust_chap6::MsgContext,
//         protocol: &mut T,
//     ) -> ::mini_lust_chap6::Result<Self> {
//         let mut field_user_id = None;
//         let mut field_user_name = None;
//         let mut field_is_male = None;
//         let mut field_extra = None;
//
//         protocol.read_struct_begin()?;
//
//         loop {
//             let ident = protocol.read_field_begin()?;
//             if ident.field_type == ::mini_lust_chap6::TType::Stop {
//                 break;
//             }
//             match ident.id {
//                 Some(1) => {
//                     ::mini_lust_chap6::ttype_comparing(
//                         ident.field_type,
//                         ::mini_lust_chap6::TType::I32,
//                     )?;
//
//                     // read i32
//                     let content = protocol.read_i32()?;
//                     field_user_id = Some(content);
//                 }
//                 Some(2) => {
//                     ::mini_lust_chap6::ttype_comparing(
//                         ident.field_type,
//                         ::mini_lust_chap6::TType::String,
//                     )?;
//
//                     // read string
//                     let content = protocol.read_string()?;
//                     field_user_name = Some(content);
//                 }
//                 Some(3) => {
//                     ::mini_lust_chap6::ttype_comparing(
//                         ident.field_type,
//                         ::mini_lust_chap6::TType::Bool,
//                     )?;
//
//                     // read bool
//                     let content = protocol.read_bool()?;
//                     field_is_male = Some(content);
//                 }
//                 Some(10) => {
//                     ::mini_lust_chap6::ttype_comparing(
//                         ident.field_type,
//                         ::mini_lust_chap6::TType::Map,
//                     )?;
//
//                     // read optional map
//                     let map_ident = protocol.read_map_begin()?;
//                     let mut map = ::std::collections::HashMap::new();
//                     for _ in 0..map_ident.size {
//                         let key = protocol.read_string()?;
//                         let value = protocol.read_string()?;
//                         map.insert(key, value);
//                     }
//                     field_extra = Some(map);
//                     protocol.read_map_end()?;
//                 }
//                 _ => {
//                     protocol.skip(ident.field_type)?;
//                 }
//             }
//             protocol.read_field_end()?;
//         }
//
//         protocol.read_struct_end()?;
//
//         let output = Self {
//             user_id: field_user_id.ok_or_else(|| {
//                 ::mini_lust_chap6::new_protocol_error(
//                     ProtocolErrorKind::InvalidData,
//                     "field user_id is required",
//                 )
//             })?,
//             user_name: field_user_name.ok_or_else(|| {
//                 ::mini_lust_chap6::new_protocol_error(
//                     ProtocolErrorKind::InvalidData,
//                     "field user_name is required",
//                 )
//             })?,
//             is_male: field_is_male.ok_or_else(|| {
//                 ::mini_lust_chap6::new_protocol_error(
//                     ProtocolErrorKind::InvalidData,
//                     "field is_male is required",
//                 )
//             })?,
//             extra: field_extra,
//         };
//
//         Ok(output)
//     }
// }
//
// /// Generated(manually) impl Message for GetUserRequest
// impl ::mini_lust_chap6::Message for GetUserRequest {
//     fn encode<T: ::mini_lust_chap6::TOutputProtocol>(
//         &self,
//         _cx: &::mini_lust_chap6::MsgContext,
//         protocol: &mut T,
//     ) -> ::mini_lust_chap6::Result<()> {
//         protocol.write_struct_begin(&::mini_lust_chap6::TStructIdentifier {
//             name: "GetUserRequest".to_string(),
//         })?;
//
//         // user_id
//         protocol.write_field_begin(&::mini_lust_chap6::TFieldIdentifier {
//             name: Some("user_id".to_string()),
//             field_type: ::mini_lust_chap6::TType::I32,
//             id: Some(1),
//         })?;
//         protocol.write_i32(self.user_id)?;
//         protocol.write_field_end()?;
//
//         // user_name
//         protocol.write_field_begin(&::mini_lust_chap6::TFieldIdentifier {
//             name: Some("user_name".to_string()),
//             field_type: ::mini_lust_chap6::TType::String,
//             id: Some(2),
//         })?;
//         protocol.write_string(&self.user_name)?;
//         protocol.write_field_end()?;
//
//         // is_male
//         protocol.write_field_begin(&::mini_lust_chap6::TFieldIdentifier {
//             name: Some("is_male".to_string()),
//             field_type: ::mini_lust_chap6::TType::Bool,
//             id: Some(3),
//         })?;
//         protocol.write_bool(self.is_male)?;
//         protocol.write_field_end()?;
//
//         protocol.write_field_stop()?;
//         protocol.write_struct_end()?;
//         Ok(())
//     }
//
//     fn decode<T: ::mini_lust_chap6::TInputProtocol>(
//         _cx: &mut ::mini_lust_chap6::MsgContext,
//         protocol: &mut T,
//     ) -> ::mini_lust_chap6::Result<Self> {
//         let mut output = Self::default();
//         protocol.read_struct_begin()?;
//
//         loop {
//             let ident = protocol.read_field_begin()?;
//             if ident.field_type == ::mini_lust_chap6::TType::Stop {
//                 break;
//             }
//             match ident.id {
//                 Some(1) => {
//                     ::mini_lust_chap6::ttype_comparing(
//                         ident.field_type,
//                         ::mini_lust_chap6::TType::I32,
//                     )?;
//
//                     // read i32
//                     let content = protocol.read_i32()?;
//                     output.user_id = content;
//                 }
//                 Some(2) => {
//                     ::mini_lust_chap6::ttype_comparing(
//                         ident.field_type,
//                         ::mini_lust_chap6::TType::String,
//                     )?;
//
//                     // read string
//                     let content = protocol.read_string()?;
//                     output.user_name = content;
//                 }
//                 Some(3) => {
//                     ::mini_lust_chap6::ttype_comparing(
//                         ident.field_type,
//                         ::mini_lust_chap6::TType::Bool,
//                     )?;
//
//                     // read bool
//                     let content = protocol.read_bool()?;
//                     output.is_male = content;
//                 }
//                 _ => {
//                     protocol.skip(ident.field_type)?;
//                 }
//             }
//             protocol.read_field_end()?;
//         }
//
//         protocol.read_struct_end()?;
//         Ok(output)
//     }
// }
//
// /// Generated(manually) impl Message for GetUserResponse
// impl ::mini_lust_chap6::Message for GetUserResponse {
//     fn encode<T: ::mini_lust_chap6::TOutputProtocol>(
//         &self,
//         cx: &::mini_lust_chap6::MsgContext,
//         protocol: &mut T,
//     ) -> ::mini_lust_chap6::Result<()> {
//         protocol.write_struct_begin(&::mini_lust_chap6::TStructIdentifier {
//             name: "GetUserResponse".to_string(),
//         })?;
//
//         // users
//         protocol.write_field_begin(&::mini_lust_chap6::TFieldIdentifier {
//             name: Some("users".to_string()),
//             field_type: ::mini_lust_chap6::TType::List,
//             id: Some(1),
//         })?;
//
//         protocol.write_list_begin(&::mini_lust_chap6::TListIdentifier {
//             element_type: ::mini_lust_chap6::TType::Struct,
//             size: self.users.len() as i32,
//         })?;
//         for user in self.users.iter() {
//             user.encode(cx, protocol)?;
//         }
//         protocol.write_list_end()?;
//
//         protocol.write_field_end()?;
//
//         protocol.write_field_stop()?;
//         protocol.write_struct_end()?;
//         Ok(())
//     }
//
//     fn decode<T: ::mini_lust_chap6::TInputProtocol>(
//         cx: &mut ::mini_lust_chap6::MsgContext,
//         protocol: &mut T,
//     ) -> ::mini_lust_chap6::Result<Self> {
//         let mut output = Self::default();
//         protocol.read_struct_begin()?;
//
//         loop {
//             let ident = protocol.read_field_begin()?;
//             if ident.field_type == ::mini_lust_chap6::TType::Stop {
//                 break;
//             }
//             match ident.id {
//                 Some(1) => {
//                     ::mini_lust_chap6::ttype_comparing(
//                         ident.field_type,
//                         ::mini_lust_chap6::TType::List,
//                     )?;
//
//                     // read list
//                     let list_ident = protocol.read_list_begin()?;
//                     output.users = Vec::with_capacity(list_ident.size as usize);
//                     for _ in 0..list_ident.size {
//                         let user = User::decode(cx, protocol)?;
//                         output.users.push(user);
//                     }
//                     protocol.read_list_end()?;
//                 }
//                 _ => {
//                     protocol.skip(ident.field_type)?;
//                 }
//             }
//             protocol.read_field_end()?;
//         }
//
//         protocol.read_struct_end()?;
//         Ok(output)
//     }
// }
//
// impl ::mini_lust_chap6::Message for AnonymousItemServiceGetUserArgs {
//     fn encode<T: ::mini_lust_chap6::TOutputProtocol>(
//         &self,
//         cx: &::mini_lust_chap6::MsgContext,
//         protocol: &mut T,
//     ) -> ::mini_lust_chap6::Result<()> {
//         protocol.write_struct_begin(&::mini_lust_chap6::TStructIdentifier {
//             name: "AnonymousItemServiceGetUserRequest".to_string(),
//         })?;
//         protocol.write_field_begin(&::mini_lust_chap6::TFieldIdentifier {
//             name: Some("req".to_string()),
//             field_type: ::mini_lust_chap6::TType::Struct,
//             id: Some(1),
//         })?;
//         self.req.encode(cx, protocol)?;
//         protocol.write_field_end()?;
//
//         protocol.write_field_begin(&::mini_lust_chap6::TFieldIdentifier {
//             name: Some("shuffle".to_string()),
//             field_type: ::mini_lust_chap6::TType::Bool,
//             id: Some(2),
//         })?;
//         protocol.write_bool(self.shuffle)?;
//         protocol.write_field_end()?;
//         protocol.write_field_stop()?;
//         protocol.write_struct_end()?;
//         Ok(())
//     }
//
//     fn decode<T: ::mini_lust_chap6::TInputProtocol>(
//         cx: &mut ::mini_lust_chap6::MsgContext,
//         protocol: &mut T,
//     ) -> ::mini_lust_chap6::Result<Self> {
//         protocol.read_struct_begin()?;
//         if cx.identifier.message_type == ::mini_lust_chap6::TMessageType::Exception {
//             // If message type is exception, we should not try decoding
//             // AnonymousItemServiceGetUserArgs
//             return Err(::mini_lust_chap6::new_application_error(
//                 ::mini_lust_chap6::ApplicationErrorKind::Unknown,
//                 "internal error: try decoding message on exception",
//             ));
//         }
//
//         let mut output = Self::default();
//         loop {
//             let ident = protocol.read_field_begin()?;
//             if ident.field_type == ::mini_lust_chap6::TType::Stop {
//                 break;
//             }
//             match ident.id {
//                 Some(1) => {
//                     ::mini_lust_chap6::ttype_comparing(
//                         ident.field_type,
//                         ::mini_lust_chap6::TType::Struct,
//                     )?;
//
//                     // read struct
//                     output.req = GetUserRequest::decode(cx, protocol)?;
//                 }
//                 Some(2) => {
//                     ::mini_lust_chap6::ttype_comparing(
//                         ident.field_type,
//                         ::mini_lust_chap6::TType::Bool,
//                     )?;
//
//                     // read bool
//                     output.shuffle = protocol.read_bool()?;
//                 }
//                 _ => {
//                     protocol.skip(ident.field_type)?;
//                 }
//             }
//             protocol.read_field_end()?;
//         }
//         protocol.read_struct_end()?;
//         Ok(output)
//     }
// }
//
// impl ::mini_lust_chap6::Message for AnonymousItemServiceGetUserResult {
//     fn encode<T: ::mini_lust_chap6::TOutputProtocol>(
//         &self,
//         cx: &::mini_lust_chap6::MsgContext,
//         protocol: &mut T,
//     ) -> ::mini_lust_chap6::Result<()> {
//         protocol.write_struct_begin(&::mini_lust_chap6::TStructIdentifier {
//             name: "AnonymousItemServiceGetUserResult".to_string(),
//         })?;
//         match self {
//             Self::Success(c) => {
//                 protocol.write_field_begin(&::mini_lust_chap6::TFieldIdentifier {
//                     name: Some("GetUserResponse".to_string()),
//                     field_type: ::mini_lust_chap6::TType::Struct,
//                     id: Some(0),
//                 })?;
//                 c.encode(cx, protocol)?;
//                 protocol.write_field_end()?;
//             } // Exception if some
//         }
//         protocol.write_field_stop()?;
//         protocol.write_struct_end()?;
//         Ok(())
//     }
//
//     fn decode<T: ::mini_lust_chap6::TInputProtocol>(
//         cx: &mut ::mini_lust_chap6::MsgContext,
//         protocol: &mut T,
//     ) -> ::mini_lust_chap6::Result<Self> {
//         protocol.read_struct_begin()?;
//         let ident = protocol.read_field_begin()?;
//         // There must be only one field
//         let output = match ident.id {
//             Some(0) => {
//                 let resp = GetUserResponse::decode(cx, protocol)?;
//                 Self::Success(resp)
//             }
//             _ => {
//                 return Err(::mini_lust_chap6::new_protocol_error(
//                     ::mini_lust_chap6::ProtocolErrorKind::InvalidData,
//                     "unexpected result field",
//                 ));
//             }
//         };
//         protocol.read_field_end()?;
//         protocol.read_struct_end()?;
//         Ok(output)
//     }
// }
//
// impl ::mini_lust_chap6::Message for AnonymousItemServiceRequest {
//     // Encode just forward encode to inner
//     fn encode<T: ::mini_lust_chap6::TOutputProtocol>(
//         &self,
//         cx: &::mini_lust_chap6::MsgContext,
//         protocol: &mut T,
//     ) -> ::mini_lust_chap6::Result<()> {
//         match self {
//             Self::GetUser(req) => req.encode(cx, protocol), // Other patterns if there are other methods
//         }
//     }
//
//     // Decode just forward decode to inner with information inside the context
//     fn decode<T: ::mini_lust_chap6::TInputProtocol>(
//         cx: &mut ::mini_lust_chap6::MsgContext,
//         protocol: &mut T,
//     ) -> ::mini_lust_chap6::Result<Self> {
//         match cx.identifier.name.as_ref() {
//             "GetUser" => Ok(Self::GetUser(::mini_lust_chap6::Message::decode(
//                 cx, protocol,
//             )?)),
//             // Other patterns if there are other methods
//             method @ _ => Err(::mini_lust_chap6::new_application_error(
//                 ::mini_lust_chap6::ApplicationErrorKind::UnknownMethod,
//                 std::format!("unknown method {}", method),
//             )),
//         }
//     }
// }
//
// impl ::mini_lust_chap6::Message for AnonymousItemServiceResponse {
//     fn encode<T: ::mini_lust_chap6::TOutputProtocol>(
//         &self,
//         cx: &::mini_lust_chap6::MsgContext,
//         protocol: &mut T,
//     ) -> ::mini_lust_chap6::Result<()> {
//         match self {
//             Self::GetUser(r) => r.encode(cx, protocol),
//         }
//     }
//
//     fn decode<T: ::mini_lust_chap6::TInputProtocol>(
//         cx: &mut ::mini_lust_chap6::MsgContext,
//         protocol: &mut T,
//     ) -> ::mini_lust_chap6::Result<Self> {
//         match cx.identifier.name.as_ref() {
//             "GetUser" => Ok(Self::GetUser(AnonymousItemServiceGetUserResult::decode(
//                 cx, protocol,
//             )?)),
//             _ => Err(::mini_lust_chap6::new_protocol_error(
//                 ::mini_lust_chap6::ProtocolErrorKind::InvalidData,
//                 "unrecognized method name",
//             )),
//         }
//     }
// }

pub struct ItemServiceClientBuilder {
    client_builder: ::mini_lust_chap6::ClientBuilder<
        ::mini_lust_chap6::DefaultMakeCodec<
            AnonymousItemServiceRequest,
            AnonymousItemServiceResponse,
        >,
    >,
}

impl ItemServiceClientBuilder {
    pub fn new(target: ::mini_lust_chap6::SocketOrUnix) -> Self {
        let client_builder = ::mini_lust_chap6::ClientBuilder::new(target);
        Self { client_builder }
    }

    pub fn build(self) -> ItemServiceClient {
        ItemServiceClient::new(self.client_builder.build())
    }
}

#[derive(Clone)]
pub struct ItemServiceClient {
    inner_client:
        ::mini_lust_chap6::Client<AnonymousItemServiceRequest, AnonymousItemServiceResponse>,
}

impl ItemServiceClient {
    pub fn new(
        inner: ::mini_lust_chap6::Client<AnonymousItemServiceRequest, AnonymousItemServiceResponse>,
    ) -> Self {
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
    ) -> ::mini_lust_chap6::Result<AnonymousItemServiceGetUserResult> {
        let anonymous_request =
            AnonymousItemServiceRequest::GetUser(AnonymousItemServiceGetUserArgs {
                req: Some(req),
                shuffle: Some(shuffle),
            });
        let resp = self.inner_client.call("GetUser", anonymous_request).await?;

        #[allow(irrefutable_let_patterns)]
        if let AnonymousItemServiceResponse::GetUser(r) = resp {
            return Ok(r);
        }
        Err(::mini_lust_chap6::new_application_error(
            ::mini_lust_chap6::ApplicationErrorKind::Unknown,
            "unable to get response",
        ))
    }
}

#[async_trait::async_trait]
pub trait ItemService {
    async fn get_user(
        &self,
        req: Option<GetUserRequest>,
        shuffle: Option<bool>,
    ) -> ::mini_lust_chap6::ApplicationResult<AnonymousItemServiceGetUserResult>;
}

pub struct ItemServiceServer<S> {
    inner: ::std::sync::Arc<S>,
}

impl<S> ItemServiceServer<S> {
    pub fn new(inner: S) -> Self {
        Self {
            inner: ::std::sync::Arc::new(inner),
        }
    }
}

impl<S>
    tower::Service<(
        ::mini_lust_chap6::MsgContext,
        ::mini_lust_chap6::ApplicationResult<AnonymousItemServiceRequest>,
    )> for ItemServiceServer<S>
where
    S: ItemService + Send + Sync + 'static,
{
    // Option since we may or may not return due to call or oneway
    type Response = Option<(
        ::mini_lust_chap6::MsgContext,
        ::mini_lust_chap6::ApplicationResult<AnonymousItemServiceResponse>,
    )>;
    type Error = ::mini_lust_chap6::Error;
    type Future = ::mini_lust_chap6::BoxFuture<Self::Response, Self::Error>;

    fn poll_ready(
        &mut self,
        _cx: &mut ::std::task::Context,
    ) -> ::std::task::Poll<std::result::Result<(), Self::Error>> {
        ::std::task::Poll::Ready(Ok(()))
    }

    fn call(
        &mut self,
        req: (
            ::mini_lust_chap6::MsgContext,
            ::mini_lust_chap6::ApplicationResult<AnonymousItemServiceRequest>,
        ),
    ) -> Self::Future {
        let inner = self.inner.clone();
        ::std::boxed::Box::pin(async move {
            let (mut cx, req) = req;
            match req {
                Ok(AnonymousItemServiceRequest::GetUser(r)) => {
                    let ret = inner.get_user(r.req, r.shuffle).await;
                    match ret {
                        Ok(r) => {
                            cx.identifier.message_type = ::mini_lust_chap6::TMessageType::Reply;
                            Ok(Some((cx, Ok(AnonymousItemServiceResponse::GetUser(r)))))
                        }
                        Err(e) => {
                            cx.identifier.message_type = ::mini_lust_chap6::TMessageType::Exception;
                            Ok(Some((cx, Err(e))))
                        }
                    }
                }
                Err(e) => {
                    log::error!("unexpected client error: {}", e);
                    Err(::mini_lust_chap6::new_application_error(
                        ::mini_lust_chap6::ApplicationErrorKind::Unknown,
                        "unexpected client error",
                    ))
                }
            }
        })
    }
}
