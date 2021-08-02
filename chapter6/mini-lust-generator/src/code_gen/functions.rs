use thrift_parser::functions::Function;
use crate::code_gen::{CodeGenContext, FunctionGen, CodeGen, FieldGen};
use proc_macro2::{TokenStream, Ident};
use crate::code_gen::errors::CodeGenResult;
use thrift_parser::types::FieldType;

impl FunctionGen for Function {
    // Generate struct AnonymousServiceCallArgs and AnonymousServiceCallResult
    fn anonymous(&self, service_ident: &Ident) -> CodeGenResult<TokenStream> {
        let mut output = TokenStream::new();

        // Generate struct AnonymousServiceCallArgs
        let struct_name = quote::format_ident!("Anonymous{}{}Args", service_ident, quote::format_ident!("{}", self.name.clone().into_inner()));
        let mut fields = Vec::with_capacity(self.parameters.len());
        for field in self.parameters.iter() {
            fields.push(field.gen_for_struct()?)
        }

        output.extend(quote::quote! {
            #[derive(::mini_lust_macros::Message, Debug, Clone, PartialEq)]
            pub struct #struct_name {
                #(#fields)*
            }
            impl ::mini_lust_chap6::OrigType for #struct_name {}
        });

        // Generate enum AnonymousServiceCallResult
        if !self.oneway {
            let enum_name = quote::format_ident!("Anonymous{}{}Result", service_ident, quote::format_ident!("{}", self.name.clone().into_inner()));
            let ret = self.returns.gen_token()?;

            let mut exps = Vec::new();
            for exception in self.exceptions.as_ref().iter().flat_map(|exps| exps.iter()) {
                let name = quote::format_ident!("{}", exception.name.clone().into_inner());
                let field_id = exception.id.expect("exception id is required").into_inner() as i16;
                let exp_type = exception.type_.gen_token()?;
                exps.push(quote::quote! {
                    #[mini_lust(field_id = #field_id)]
                    #name(#exp_type),
                });
            }

            output.extend(quote::quote! {
                #[derive(::mini_lust_macros::Message, Debug, Clone, PartialEq)]
                pub enum #enum_name {
                    #[mini_lust(field_id = 1)]
                    Success(#ret),
                    // User defined exceptions here
                    #(#exps)*
                }
                impl ::mini_lust_chap6::OrigType for #enum_name {}
            });
        }

        Ok(output)
    }

    // pub async fn get_user(
    //         &mut self,
    //         req: GetUserRequest,
    //         shuffle: bool,
    //     ) -> ::mini_lust_chap6::Result<AnonymousItemServiceGetUserResult> {
    //         let anonymous_request =
    //             AnonymousItemServiceRequest::GetUser(AnonymousItemServiceGetUserArgs {
    //                 req: Some(req),
    //                 shuffle: Some(shuffle),
    //             });
    //         let resp = self.inner_client.call("GetUser", anonymous_request).await?;
    //
    //         #[allow(irrefutable_let_patterns)]
    //         if let AnonymousItemServiceResponse::GetUser(r) = resp {
    //             return Ok(r);
    //         }
    //         Err(::mini_lust_chap6::new_application_error(
    //             ::mini_lust_chap6::ApplicationErrorKind::Unknown,
    //             "unable to get response",
    //         ))
    //     }
    fn impl_for_client(&self, service_ident: &Ident) -> CodeGenResult<TokenStream> {
        let anonymous_args = quote::format_ident!("Anonymous{}{}Args", service_ident, self.name.clone().into_inner());
        let anonymous_result = quote::format_ident!("Anonymous{}{}Result", service_ident, self.name.clone().into_inner());
        let anonymous_request = quote::format_ident!("Anonymous{}Request", service_ident);
        let anonymous_response = quote::format_ident!("Anonymous{}Response", service_ident);
        let func_name = quote::format_ident!("{}", self.name.clone().into_inner());
        let func_name_string = self.name.clone().into_inner();

        let mut named_parameters = Vec::new();
        let mut assignments = Vec::new();
        for field in self.parameters.iter() {
            named_parameters.push(field.gen_name_type(true)?);
            let field_name = quote::format_ident!("{}", field.name.clone().into_inner());
            if field.required == None {
                assignments.push(quote::quote! { #field_name: Some(#field_name), })
            } else {
                assignments.push(quote::quote! { #field_name, })
            }
        }

        // TODO: call or oneway
        Ok(quote::quote! {
            pub async fn #func_name(
                &mut self,
                #(#named_parameters)*
            ) -> ::mini_lust_chap6::Result<#anonymous_result> {
                let anonymous_request =
                    #anonymous_request::GetUser(#anonymous_args {
                        #(#assignments)*
                    });
                let resp = self.inner_client.call(#func_name_string, anonymous_request).await?;

                #[allow(irrefutable_let_patterns)]
                if let #anonymous_response::#func_name(r) = resp {
                    return Ok(r);
                }
                Err(::mini_lust_chap6::new_application_error(
                    ::mini_lust_chap6::ApplicationErrorKind::Unknown,
                    "unable to get response",
                ))
            }
        })
    }

    // async fn get_user(
    //         &self,
    //         req: Option<GetUserRequest>,
    //         shuffle: Option<bool>,
    //     ) -> ::mini_lust_chap6::ApplicationResult<AnonymousItemServiceGetUserResult>;
    fn fn_for_trait(&self, service_ident: &Ident) -> CodeGenResult<TokenStream> {
        let mut fields = Vec::new();
        for field in self.parameters.iter() {
            fields.push(field.gen_name_type(false)?);
        }
        let anonymous_result = quote::format_ident!("Anonymous{}{}Result", service_ident, self.name.clone().into_inner());
        let func_name = quote::format_ident!("{}", self.name.clone().into_inner());
        Ok(quote::quote! {
            async fn #func_name(
                &self,
                #(#fields)*
            ) -> ::mini_lust_chap6::ApplicationResult<#anonymous_result>;
        })
    }

    // Ok(AnonymousItemServiceRequest::GetUser(r)) => {
    //     let ret = inner.get_user(r.req, r.shuffle).await;
    //     match ret {
    //         Ok(r) => {
    //             cx.identifier.message_type = ::mini_lust_chap6::TMessageType::Reply;
    //             Ok(Some((cx, Ok(AnonymousItemServiceResponse::GetUser(r)))))
    //         }
    //         Err(e) => {
    //             cx.identifier.message_type = ::mini_lust_chap6::TMessageType::Exception;
    //             Ok(Some((cx, Err(e))))
    //         }
    //     }
    // }
    fn server_match_arm(&self, service_ident: &Ident) -> CodeGenResult<TokenStream> {
        let anonymous_request = quote::format_ident!("Anonymous{}Request", service_ident);
        let anonymous_response = quote::format_ident!("Anonymous{}Response", service_ident);
        let func_name = quote::format_ident!("{}", self.name.clone().into_inner());
        let r_parameters = self.parameters.iter().map(|f| {
            let name = quote::format_ident!("{}", f.name.clone().into_inner());
            quote::quote! { r.#name }
        });

        if self.oneway {
            return Ok(quote::quote! {
                Ok(#anonymous_request::#func_name(r)) => {
                    let ret = inner.#func_name(#(#r_parameters),*).await;
                    Ok(None)
                }
            });
        }
        Ok(quote::quote! {
            Ok(#anonymous_request::#func_name(r)) => {
                let ret = inner.#func_name(#(#r_parameters),*).await;
                match ret {
                    Ok(r) => {
                        cx.identifier.message_type = ::mini_lust_chap6::TMessageType::Reply;
                        Ok(Some((cx, Ok(#anonymous_response::#func_name(r)))))
                    }
                    Err(e) => {
                        cx.identifier.message_type = ::mini_lust_chap6::TMessageType::Exception;
                        Ok(Some((cx, Err(e))))
                    }
                }
            }
        })
    }
}