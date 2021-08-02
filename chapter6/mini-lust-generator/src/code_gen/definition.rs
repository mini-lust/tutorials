use proc_macro2::TokenStream;

use thrift_parser::definition::{Struct, Service};

use crate::code_gen::errors::CodeGenResult;
use crate::code_gen::{CodeGenWithContext, CodeGenContext, IdentifierGen, CodeGen, FunctionGen, FieldGen};

impl CodeGen for Struct {
    fn write_token(&self, output: &mut TokenStream) -> CodeGenResult<()> {
        let struct_name = self.name.struct_name()?;
        let mut fields = Vec::with_capacity(self.fields.len());
        for field in self.fields.iter() {
            fields.push(field.gen_for_struct()?)
        }

        output.extend(quote::quote! {
            #[derive(::mini_lust_macros::Message, Debug, Clone, PartialEq)]
            pub struct #struct_name {
                #(#fields)*
            }
            impl ::mini_lust_chap6::OrigType for #struct_name {}
        });

        Ok(())
    }
}

impl CodeGen for Service {
    fn write_token(&self, output: &mut TokenStream) -> CodeGenResult<()> {
        // Generate struct AnonymousServiceCallArgs and AnonymousServiceCallResult
        let serv_name = quote::format_ident!("{}", self.name.clone().into_inner());
        for func in self.functions.iter() {
            output.extend(func.anonymous(&serv_name)?);
        }

        // Generate enum AnonymousServiceRequest and enum AnonymousServiceResponse
        let request_name = quote::format_ident!("Anonymous{}Request", serv_name);
        let response_name = quote::format_ident!("Anonymous{}Response", serv_name);
        let func_names = self.functions.iter().map(|f|quote::format_ident!("{}", f.name.clone().into_inner())).collect::<Vec<_>>();
        let func_args = func_names.iter().map(|n| quote::format_ident!("Anonymous{}{}Args", serv_name, n));
        let func_results = self.functions.iter().map(|f| {
            if f.oneway {
                return quote::format_ident!("()");
            }
            let name = quote::format_ident!("{}", f.name.clone().into_inner());
            quote::format_ident!("Anonymous{}{}Result", serv_name, name)
        });

        output.extend(quote::quote! {
            #[derive(::mini_lust_macros::Message, Debug, Clone, PartialEq)]
            #[mini_lust(dispatch_only = true)]
            pub enum #request_name {
                #(#func_names(#func_args),)*
            }
            impl ::mini_lust_chap6::OrigType for #request_name {}
        });
        output.extend(quote::quote! {
            #[derive(::mini_lust_macros::Message, Debug, Clone, PartialEq)]
            #[mini_lust(dispatch_only = true)]
            pub enum #response_name {
                #(#func_names(#func_results),)*
            }
            impl ::mini_lust_chap6::OrigType for #response_name {}
        });

        // Generate ServiceClientBuilder and its impl
        let client_builder_name = quote::format_ident!("{}ClientBuilder", serv_name);
        let client_name = quote::format_ident!("{}Client", serv_name);

        output.extend(quote::quote! {
            pub struct #client_builder_name {
                client_builder: ::mini_lust_chap6::ClientBuilder<
                    ::mini_lust_chap6::DefaultMakeCodec<
                        #request_name,
                        #response_name,
                    >,
                >,
            }

            impl #client_builder_name {
                pub fn new(target: ::mini_lust_chap6::SocketOrUnix) -> Self {
                    let client_builder = ::mini_lust_chap6::ClientBuilder::new(target);
                    Self { client_builder }
                }

                pub fn build(self) -> #client_name {
                    #client_name::new(self.client_builder.build())
                }
            }
        });

        // Generate ServiceClient and its impl
        output.extend(quote::quote! {
            #[derive(Clone)]
            pub struct #client_name {
                inner_client:
                    ::mini_lust_chap6::Client<#request_name, #response_name>,
            }

            impl #client_name {
                pub fn new(
                    inner: ::mini_lust_chap6::Client<#request_name, #response_name>,
                ) -> Self {
                    Self {
                        inner_client: inner,
                    }
                }
            }
        });

        // impl ServiceClient call
        let mut impls = Vec::new();
        for func in self.functions.iter() {
            impls.push(func.impl_for_client(&serv_name)?);
        }
        output.extend(quote::quote! {
            impl #client_name {
                #(#impls)*
            }
        });

        // Generate trait Service
        let mut trait_funcs = Vec::new();
        for func in self.functions.iter() {
            trait_funcs.push(func.fn_for_trait(&serv_name)?);
        }
        output.extend(quote::quote! {
            #[async_trait::async_trait]
            pub trait #serv_name {
                #(#trait_funcs)*
            }
        });

        // Generate ServiceServer and its impl
        let server_name = quote::format_ident!("{}Server", serv_name);
        output.extend(quote::quote! {
            pub struct #server_name<S> {
                inner: ::std::sync::Arc<S>,
            }

            impl<S> #server_name<S> {
                pub fn new(inner: S) -> Self {
                    Self {
                        inner: ::std::sync::Arc::new(inner),
                    }
                }
            }
        });

        let mut server_match_arms = Vec::new();
        for func in self.functions.iter() {
            server_match_arms.push(func.server_match_arm(&serv_name)?);
        }

        output.extend(quote::quote! {
            impl<S>
                ::tower::Service<(
                    ::mini_lust_chap6::MsgContext,
                    ::mini_lust_chap6::ApplicationResult<#request_name>,
                )> for #server_name<S>
            where
                S: #serv_name + Send + Sync + 'static,
            {
                // Option since we may or may not return due to call or oneway
                type Response = Option<(
                    ::mini_lust_chap6::MsgContext,
                    ::mini_lust_chap6::ApplicationResult<#response_name>,
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
                        ::mini_lust_chap6::ApplicationResult<#request_name>,
                    ),
                ) -> Self::Future {
                    let inner = self.inner.clone();
                    ::std::boxed::Box::pin(async move {
                        let (mut cx, req) = req;
                        match req {
                            #(#server_match_arms)*
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
        });

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use thrift_parser::definition::Struct;
    use thrift_parser::Parser;

    use crate::code_gen::{CodeGenWithContext, CodeGenContext, CodeGen};

    #[test]
    fn test_gen_struct() {
        // #[derive(Debug)]
        // pub struct MyStruct {}
        let s = Struct::parse("struct MyStruct {}").unwrap().1;
        assert_eq!(
            s.gen_token().unwrap().to_string(),
            "# [derive (:: mini_lust_macros :: Message , Debug , Clone , PartialEq)] pub struct MyStruct { } impl :: mini_lust_chap6 :: OrigType for MyStruct { }"
        );

        // #[derive(Debug)]
        // pub struct my_struct {
        //     pub count: ::std::option::Option<i32>,
        //     pub user_list:
        //         ::std::collections::BTreeMap<::std::string::String, ::std::string::String>,
        // }
        let s = Struct::parse(
            "struct my_struct {1: required i32 count, 2: optional map<string, string> UserList}",
        )
        .unwrap()
        .1;
        assert_eq!(s.gen_token().unwrap().to_string(), "# [derive (:: mini_lust_macros :: Message , Debug , Clone , PartialEq)] pub struct my_struct { # [mini_lust (field_id = 1i16 , required = \"true\" , field_type = \"i32\")] pub count : i32 , # [mini_lust (field_id = 2i16 , required = \"false\" , field_type = \"map(string, string)\")] pub user_list : :: std :: option :: Option < :: std :: collections :: BTreeMap < :: std :: string :: String , :: std :: string :: String > > , } impl :: mini_lust_chap6 :: OrigType for my_struct { }");
    }
}
