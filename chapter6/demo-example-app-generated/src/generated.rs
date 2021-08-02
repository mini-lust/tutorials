pub mod demo {
    #[derive(mini_lust_macros :: Message, Debug, Clone, PartialEq)]
    pub struct User {
        #[mini_lust(field_id = 1i16, required = "true", field_type = "i32")]
        pub user_id: i32,
        #[mini_lust(field_id = 2i16, required = "true", field_type = "string")]
        pub user_name: ::std::string::String,
        #[mini_lust(field_id = 3i16, required = "true", field_type = "bool")]
        pub is_male: bool,
        #[mini_lust(
            field_id = 10i16,
            required = "false",
            field_type = "map(string, string)"
        )]
        pub extra: ::std::option::Option<
            ::std::collections::BTreeMap<::std::string::String, ::std::string::String>,
        >,
    }
    impl ::mini_lust_chap6::OrigType for User {}
    #[derive(mini_lust_macros :: Message, Debug, Clone, PartialEq)]
    pub struct GetUserRequest {
        #[mini_lust(field_id = 1i16, field_type = "i32")]
        pub user_id: ::std::option::Option<i32>,
        #[mini_lust(field_id = 2i16, field_type = "string")]
        pub user_name: ::std::option::Option<::std::string::String>,
        #[mini_lust(field_id = 3i16, field_type = "bool")]
        pub is_male: ::std::option::Option<bool>,
    }
    impl ::mini_lust_chap6::OrigType for GetUserRequest {}
    #[derive(mini_lust_macros :: Message, Debug, Clone, PartialEq)]
    pub struct GetUserResponse {
        #[mini_lust(field_id = 1i16, required = "true", field_type = "list(ident(User))")]
        pub users: ::std::vec::Vec<User>,
    }
    impl ::mini_lust_chap6::OrigType for GetUserResponse {}
    #[derive(mini_lust_macros :: Message, Debug, Clone, PartialEq)]
    pub struct AnonymousItemServiceGetUserArgs {
        #[mini_lust(field_id = 1i16, field_type = "ident(GetUserRequest)")]
        pub req: ::std::option::Option<GetUserRequest>,
        #[mini_lust(field_id = 2i16, field_type = "bool")]
        pub shuffle: ::std::option::Option<bool>,
    }
    impl ::mini_lust_chap6::OrigType for AnonymousItemServiceGetUserArgs {}
    #[derive(mini_lust_macros :: Message, Debug, Clone, PartialEq)]
    pub enum AnonymousItemServiceGetUserResult {
        #[mini_lust(field_id = 1)]
        Success(GetUserResponse),
    }
    impl ::mini_lust_chap6::OrigType for AnonymousItemServiceGetUserResult {}
    #[derive(mini_lust_macros :: Message, Debug, Clone, PartialEq)]
    #[mini_lust(dispatch_only = true)]
    pub enum AnonymousItemServiceRequest {
        GetUser(AnonymousItemServiceGetUserArgs),
    }
    impl ::mini_lust_chap6::OrigType for AnonymousItemServiceRequest {}
    #[derive(mini_lust_macros :: Message, Debug, Clone, PartialEq)]
    #[mini_lust(dispatch_only = true)]
    pub enum AnonymousItemServiceResponse {
        GetUser(AnonymousItemServiceGetUserResult),
    }
    impl ::mini_lust_chap6::OrigType for AnonymousItemServiceResponse {}
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
            inner: ::mini_lust_chap6::Client<
                AnonymousItemServiceRequest,
                AnonymousItemServiceResponse,
            >,
        ) -> Self {
            Self {
                inner_client: inner,
            }
        }
    }
    impl ItemServiceClient {
        pub async fn GetUser(
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
        async fn GetUser(
            &self,
            req: ::std::option::Option<GetUserRequest>,
            shuffle: ::std::option::Option<bool>,
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
        ::tower::Service<(
            ::mini_lust_chap6::MsgContext,
            ::mini_lust_chap6::ApplicationResult<AnonymousItemServiceRequest>,
        )> for ItemServiceServer<S>
    where
        S: ItemService + Send + Sync + 'static,
    {
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
                        let ret = inner.GetUser(r.req, r.shuffle).await;
                        match ret {
                            Ok(r) => {
                                cx.identifier.message_type = ::mini_lust_chap6::TMessageType::Reply;
                                Ok(Some((cx, Ok(AnonymousItemServiceResponse::GetUser(r)))))
                            }
                            Err(e) => {
                                cx.identifier.message_type =
                                    ::mini_lust_chap6::TMessageType::Exception;
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
}
