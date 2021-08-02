use proc_macro2::TokenStream;
use quote::ToTokens;

use thrift_parser::field::Field;
use thrift_parser::types::FieldType;

use crate::code_gen::errors::CodeGenResult;
use crate::code_gen::{CodeGenWithContext, CodeGenContext, IdentifierGen, CodeGen, FieldGen};
use thrift_parser::constant::IntConstant;

trait FormatFieldType {
    fn format(&self) -> String;
}

impl FormatFieldType for FieldType {
    fn format(&self) -> String {
        match self {
            FieldType::String => "string".to_string(),
            FieldType::Bool => "bool".to_string(),
            FieldType::I8 => "i8".to_string(),
            FieldType::I16 => "i16".to_string(),
            FieldType::I32 => "i32".to_string(),
            FieldType::I64 => "i64".to_string(),
            FieldType::Double => "double".to_string(),
            FieldType::Byte => "byte".to_string(),
            FieldType::Identifier(ident) => {
                format!("ident({})", ident.as_str())
            }
            FieldType::List(inner) => {
                format!("list({})", inner.format())
            }
            FieldType::Map(k, v) => {
                format!("map({}, {})", k.format(), v.format())
            }
            FieldType::Set(inner) => {
                format!("set({})", inner.format())
            }
            FieldType::Binary => "binary".to_string(),
        }
    }
}

impl FieldGen for Field {
    fn gen_for_struct(&self) -> CodeGenResult<TokenStream> {
        let name = self.name.field_name()?;
        let mut type_ = self.type_.gen_token()?;
        if self.required != Some(true) {
            type_ = quote::quote! { ::std::option::Option<#type_> }
        }

        let annotation_id = self.id.map(|x| x.into_inner()).unwrap_or(0) as i16;
        let annotation_type = self.type_.format();
        let annotation = match self.required {
            None => quote::quote! {
                #[mini_lust(field_id = #annotation_id, field_type = #annotation_type)]
            },
            Some(true) => quote::quote! {
                #[mini_lust(field_id = #annotation_id, required = "true", field_type = #annotation_type)]
            },
            Some(false) => quote::quote! {
                #[mini_lust(field_id = #annotation_id, required = "false", field_type = #annotation_type)]
            }
        };

        Ok(quote::quote! {
            #annotation
            pub #name: #type_,
        })
    }

    fn gen_name_type(&self, is_encode: bool) -> CodeGenResult<TokenStream> {
        let name = self.name.field_name()?;
        let mut type_ = self.type_.gen_token()?;

        match self.required {
            Some(false) => { type_ = quote::quote! { ::std::option::Option<#type_> }; },
            None if !is_encode => { type_ = quote::quote! { ::std::option::Option<#type_> }; },
            _ => {}
        }

        Ok(quote::quote! {
            #name: #type_,
        })
    }
}
