use heck::{CamelCase, SnakeCase};
use proc_macro2::TokenStream;
use quote::ToTokens;

use thrift_parser::basic::Identifier;

use crate::code_gen::errors::CodeGenResult;
use crate::code_gen::IdentifierGen;

impl IdentifierGen for Identifier {
    fn field_name(&self) -> CodeGenResult<TokenStream> {
        let name = quote::format_ident!("{}", self.as_str().to_snake_case().to_lowercase());
        Ok(name.to_token_stream())
    }

    fn ident_name(&self) -> CodeGenResult<TokenStream> {
        let name = quote::format_ident!("{}", self.as_str().to_camel_case());
        Ok(name.to_token_stream())
    }

    fn struct_name(&self) -> CodeGenResult<TokenStream> {
        let name = quote::format_ident!("{}", self.as_str().replace(".", "::"));
        Ok(name.to_token_stream())
    }
}
