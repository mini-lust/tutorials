use proc_macro2::TokenStream;

use thrift_parser::types::FieldType;

use crate::code_gen::errors::CodeGenResult;
use crate::code_gen::{CodeGen, CodeGenContext, CodeGenWithContext, IdentifierGen};

impl CodeGen for FieldType {
    fn write_token(&self, output: &mut TokenStream) -> CodeGenResult<()> {
        output.extend(match self {
            FieldType::Identifier(ident) => ident.struct_name()?,
            FieldType::Bool => quote::quote! { bool },
            FieldType::Byte => quote::quote! { u8 },
            FieldType::I8 => quote::quote! { i8 },
            FieldType::I16 => quote::quote! { i16 },
            FieldType::I32 => quote::quote! { i32 },
            FieldType::I64 => quote::quote! { i64 },
            FieldType::Double => quote::quote! { f64 },
            FieldType::String => quote::quote! { ::std::string::String },
            FieldType::Binary => quote::quote! { ::std::vec::Vec<u8> },
            FieldType::Map(k, v) => {
                let k = k.gen_token()?;
                let v = v.gen_token()?;
                quote::quote! { ::std::collections::BTreeMap<#k, #v> }
            }
            FieldType::Set(v) => {
                let v = v.gen_token()?;
                quote::quote! { ::std::collections::BTreeSet<#v> }
            }
            FieldType::List(v) => {
                let v = v.gen_token()?;
                quote::quote! { ::std::vec::Vec<#v> }
            }
        });
        Ok(())
    }
}

impl CodeGen for Option<FieldType> {
    fn write_token(&self, output: &mut TokenStream) -> CodeGenResult<()> {
        match self.as_ref() {
            None => output.extend(quote::quote! { () }),
            Some(t) => {
                t.write_token(output);
            }
        }
        Ok(())
    }
}
