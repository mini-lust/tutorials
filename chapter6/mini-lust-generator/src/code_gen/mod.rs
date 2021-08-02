use proc_macro2::{TokenStream, Ident};

use crate::code_gen::errors::CodeGenResult;

mod basic;
mod definition;
mod document;
pub mod errors;
mod field;
mod types;
mod functions;

#[derive(Default)]
pub struct CodeGenContext {
    // namespaces of all files that it contains, eg: a::b::c, c::d::e
    includes: Vec<String>,
    // namespaces, eg: a::b::C -> [a, b, c]
    namespaces: Vec<String>,
}

impl CodeGenContext {
    pub fn new(mut includes: Vec<String>, namespaces: String) -> Self {
        includes
            .iter_mut()
            .for_each(|item| *item = item.replace(".", "::"));
        let namespaces = namespaces.split("::").map(|x| x.to_string()).collect();
        Self {
            includes,
            namespaces,
        }
    }
}

pub trait CodeGenWithContext {
    fn gen_token(&self, cx: &CodeGenContext) -> CodeGenResult<TokenStream> {
        let mut stream = TokenStream::new();
        let _ = self.write_token(cx, &mut stream)?;
        Ok(stream)
    }
    fn write_token(&self, cx: &CodeGenContext, output: &mut TokenStream) -> CodeGenResult<()>;
}

pub trait CodeGen {
    fn gen_token(&self) -> CodeGenResult<TokenStream> {
        let mut stream = TokenStream::new();
        let _ = self.write_token(&mut stream)?;
        Ok(stream)
    }
    fn write_token(&self, output: &mut TokenStream) -> CodeGenResult<()>;
}

pub trait IdentifierGen {
    fn field_name(&self) -> CodeGenResult<TokenStream>;
    fn ident_name(&self) -> CodeGenResult<TokenStream>;
    fn struct_name(&self) -> CodeGenResult<TokenStream>;
}

pub trait FunctionGen {
    fn anonymous(&self, service_ident: &Ident) -> CodeGenResult<TokenStream>;
    fn impl_for_client(&self, service_ident: &Ident) -> CodeGenResult<TokenStream>;
    fn fn_for_trait(&self, service_ident: &Ident) -> CodeGenResult<TokenStream>;
    fn server_match_arm(&self, service_ident: &Ident) -> CodeGenResult<TokenStream>;
}

pub trait FieldGen {
    fn gen_for_struct(&self) -> CodeGenResult<TokenStream>;
    fn gen_name_type(&self, is_encode: bool) -> CodeGenResult<TokenStream>;
}
