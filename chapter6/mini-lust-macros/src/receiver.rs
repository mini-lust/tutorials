use darling::{ast, FromDeriveInput, FromField, FromMeta, FromVariant};
use proc_macro2::TokenStream;
use quote::ToTokens;

use crate::fields::{decode_content, encode_content};
use crate::types::FieldType;

#[derive(Debug, Clone, Copy, FromMeta)]
#[darling(default)]
pub enum Required {
    True,
    False,
    Unspecified,
}

impl Default for Required {
    fn default() -> Self {
        Self::Unspecified
    }
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(mini_lust))]
pub(crate) struct StructReceiver {
    pub ident: syn::Ident,
    pub generics: syn::Generics,
    pub data: ast::Data<EnumReceiver, FieldReceiver>,

    #[darling(default)]
    pub dispatch_only: bool,
}

#[derive(Debug, FromField)]
#[darling(attributes(mini_lust))]
pub(crate) struct FieldReceiver {
    pub ident: Option<syn::Ident>,
    pub ty: syn::Type,

    pub field_type: String,
    pub field_id: i32,
    #[darling(default)]
    pub required: Required,
}

#[derive(Debug, FromVariant)]
#[darling(attributes(mini_lust))]
pub(crate) struct EnumReceiver {
    pub ident: syn::Ident,

    pub fields: ast::Fields<syn::Field>,
    // pub field_type: Option<String>,
    #[darling(default)]
    pub field_id: Option<i16>,
}

impl FieldReceiver {
    fn to_encode_tokens(&self) -> TokenStream {
        let ident = self.ident.clone().expect("field ident is required");
        let field_id = self.field_id as i16;
        let field_type = FieldType::parse(&self.field_type).expect("unable to parse field type");

        let inner = encode_content(&field_type, &quote::format_ident!("inner"));
        let encode = quote::quote! {
            protocol.write_field_begin(&::mini_lust_chap6::TFieldIdentifier {
                name: Some(stringify!(#ident).to_string()),
                field_type: #field_type,
                id: Some(#field_id),
            })?;
            #inner
            protocol.write_field_end()?;
        };

        let conditional_encode = match self.required {
            // required = true, field is not Option, we can always read the value
            Required::True => {
                quote::quote! {
                    let inner = &self.#ident;
                    #encode
                }
            }
            // required = false, field is Option, encode it if it has value
            Required::False => {
                quote::quote! {
                    if let Some(inner) = self.#ident.as_ref() {
                        #encode
                    }
                }
            }
            // required = unspecified, field is Option, but it must be set when encode
            // Ref: https://stackoverflow.com/questions/53357745/thrift-converting-optional-to-default-or-required
            Required::Unspecified => {
                let error_msg = format!("field {} must be set", ident);
                quote::quote! {
                    if let Some(inner) = self.#ident.as_ref() {
                        #encode
                    } else {
                        return Err(::mini_lust_chap6::new_protocol_error(::mini_lust_chap6::ProtocolErrorKind::InvalidData, #error_msg));
                    }
                }
            }
        };

        conditional_encode
    }

    fn to_decode_option_declare(&self) -> TokenStream {
        let ident = self.ident.clone().expect("field ident is required");
        let field_prefix_name = quote::format_ident!("field_{}", ident);

        quote::quote! {
            let mut #field_prefix_name = None;
        }
    }

    fn to_decode_match_arm(&self) -> TokenStream {
        let ident = self.ident.clone().expect("field ident is required");
        let field_prefix_name = quote::format_ident!("field_{}", ident);
        let field_id = self.field_id as i16;
        let field_type = FieldType::parse(&self.field_type).expect("unable to parse field type");

        let decode_content = decode_content(&field_type);
        quote::quote! {
            Some(#field_id) => {
                ::mini_lust_chap6::ttype_comparing(ident.field_type, #field_type)?;

                let content = #decode_content;
                #field_prefix_name = Some(content);
            }
        }
    }

    fn to_decode_assign(&self) -> TokenStream {
        let ident = self.ident.clone().expect("field ident is required");
        let field_prefix_name = quote::format_ident!("field_{}", ident);
        let error_msg = format!("field {} is required", ident);

        match self.required {
            Required::True => {
                quote::quote! {
                    #ident: #field_prefix_name.ok_or_else(|| ::mini_lust_chap6::new_protocol_error(::mini_lust_chap6::ProtocolErrorKind::InvalidData, #error_msg))?,
                }
            }
            Required::False | Required::Unspecified => {
                quote::quote! {
                    #ident: #field_prefix_name,
                }
            }
        }
    }
}

impl EnumReceiver {
    pub fn to_encode_arms(&self) -> TokenStream {
        let ident = self.ident.clone();
        let field_id = self.field_id.expect("field id is required");
        let inner_name =if let syn::Type::Path(p) = self.fields.fields.first().expect("enum inner is required").ty.clone() {
            p.path.segments.last().expect("no ident found for enum inner").clone().ident
        } else {
            panic!("enum inner is not Path");
        };

        quote::quote! {
            Self::#ident(inner) => {
                protocol.write_field_begin(&::mini_lust_chap6::TFieldIdentifier {
                    name: Some(stringify!(#inner_name).to_string()),
                    field_type: ::mini_lust_chap6::TType::Struct,
                    id: Some(#field_id),
                })?;
                inner.encode(cx, protocol)?;
                protocol.write_field_end()?;
            }
        }
    }

    pub fn to_decode_arms(&self) -> TokenStream {
        let ident = self.ident.clone();
        let field_id = self.field_id.expect("field id is required");

        quote::quote! {
            Some(#field_id) => {
                let resp = ::mini_lust_chap6::Message::decode(cx, protocol)?;
                Self::#ident(resp)
            }
        }
    }

    pub fn to_encode_dispatch_arms(&self) -> TokenStream {
        let ident = self.ident.clone();

        quote::quote! {
            Self::#ident(req) => req.encode(cx, protocol),
        }
    }

    pub fn to_decode_dispatch_arms(&self) -> TokenStream {
        let ident = self.ident.clone();
        let ident_name = self.ident.clone().to_string();

        quote::quote! {
            #ident_name => Ok(Self::#ident(::mini_lust_chap6::Message::decode(
                cx, protocol,
            )?)),
        }
    }
}

pub(crate) fn fields_to_message(
    struct_ident: syn::Ident,
    fields: Vec<FieldReceiver>,
) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
    let inner_encode = fields
        .iter()
        .map(FieldReceiver::to_encode_tokens)
        .collect::<Vec<_>>();
    let encode = quote::quote! {
        protocol.write_struct_begin(&::mini_lust_chap6::TStructIdentifier {
            name: stringify!(#struct_ident).to_string(),
        })?;
        #(#inner_encode)*
        protocol.write_field_stop()?;
        protocol.write_struct_end()?;
        Ok(())
    };

    let decode_option_declare = fields
        .iter()
        .map(FieldReceiver::to_decode_option_declare)
        .collect::<Vec<_>>();
    let decode_match_arm = fields
        .iter()
        .map(FieldReceiver::to_decode_match_arm)
        .collect::<Vec<_>>();
    let decode_assign = fields
        .iter()
        .map(FieldReceiver::to_decode_assign)
        .collect::<Vec<_>>();

    let decode = quote::quote! {
        #(#decode_option_declare)*
        protocol.read_struct_begin()?;
        loop {
            let ident = protocol.read_field_begin()?;
            if ident.field_type == ::mini_lust_chap6::TType::Stop {
                break;
            }
            match ident.id {
                #(#decode_match_arm)*
                _ => {
                    protocol.skip(ident.field_type)?;
                }
            }
            protocol.read_field_end()?;
        }

        protocol.read_struct_end()?;

        let output = Self {
            #(#decode_assign)*
        };

        Ok(output)
    };
    (encode, decode)
}

pub(crate) fn enum_to_message(
    enum_ident: syn::Ident,
    enums: Vec<EnumReceiver>,
    dispatch_only: bool
) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
    if dispatch_only {
        let encode_dispatch_arms = enums.iter().map(EnumReceiver::to_encode_dispatch_arms).collect::<Vec<_>>();
        let decode_dispatch_arms = enums.iter().map(EnumReceiver::to_decode_dispatch_arms).collect::<Vec<_>>();

        let encode = quote::quote! {
            match self {
                #(#encode_dispatch_arms)*
            }
        };
        let decode = quote::quote! {
            match cx.identifier.name.as_ref() {
                #(#decode_dispatch_arms)*
                _ => Err(::mini_lust_chap6::new_application_error(
                    ::mini_lust_chap6::ApplicationErrorKind::UnknownMethod,
                    "unknown method {}",
                )),
            }
        };
        return (encode, decode);
    }

    // Non-dispatch code
    let encode_arms = enums.iter().map(EnumReceiver::to_encode_arms).collect::<Vec<_>>();
    let decode_arms = enums.iter().map(EnumReceiver::to_decode_arms).collect::<Vec<_>>();

    let encode = quote::quote! {
        protocol.write_struct_begin(&::mini_lust_chap6::TStructIdentifier {
            name: stringify!(#enum_ident).to_string(),
        })?;
        match self {
            #(#encode_arms)*
        }
        protocol.write_field_stop()?;
        protocol.write_struct_end()?;
        Ok(())
    };

    let decode = quote::quote! {
        protocol.read_struct_begin()?;
        let ident = protocol.read_field_begin()?;
        // There must be only one field
        let output = match ident.id {
            #(#decode_arms)*
            _ => {
                return Err(::mini_lust_chap6::new_protocol_error(
                    ::mini_lust_chap6::ProtocolErrorKind::InvalidData,
                    "unexpected result field",
                ));
            }
        };
        protocol.read_field_end()?;
        protocol.read_struct_end()?;
        Ok(output)
    };

    (encode, decode)
}
