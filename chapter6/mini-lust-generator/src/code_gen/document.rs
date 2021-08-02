use proc_macro2::TokenStream;

use thrift_parser::document::Document;

use crate::code_gen::errors::CodeGenResult;
use crate::code_gen::{CodeGen, CodeGenContext, CodeGenWithContext};

impl CodeGenWithContext for Document {
    fn write_token(&self, cx: &CodeGenContext, output: &mut TokenStream) -> CodeGenResult<()> {
        let mut generated = TokenStream::new();

        // generate include
        // We may not use includes of self since they are the file system path instead of
        // their namespace.
        // So the includes is set with CodeGenContext.
        for inc in cx.includes.iter() {
            let parts = inc
                .split("::")
                .map(|p| quote::format_ident!("{}", p))
                .collect::<Vec<_>>();
            generated.extend(quote::quote! {
                pub use #(#parts)::*;
            })
        }

        // generate struct
        for stut in self.structs.iter() {
            let _ = stut.write_token(&mut generated)?;
        }

        // generate service
        for service in self.services.iter() {
            let _ = service.write_token(&mut generated)?;
        }

        // generate namespaces, it will wrap the generated above.
        // We may not use namespaces of self since we only want to use scope rs or *.
        // Also, if no namespace exists, we want to use the file stem and self does not
        // know it.
        // So the namespace is set with CodeGenContext.
        for m in cx.namespaces.iter().rev() {
            let ident = quote::format_ident!("{}", m);
            generated = quote::quote! {
                pub mod #ident {
                    #generated
                }
            }
        }
        // write to output
        output.extend(generated);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use thrift_parser::document::Document;
    use thrift_parser::Parser;

    use crate::code_gen::{CodeGenContext, CodeGenWithContext};

    #[test]
    fn test_namespace() {
        let doc = Document::default();
        let cx = CodeGenContext {
            includes: vec![],
            namespaces: vec!["a".to_string(), "b".to_string(), "c".to_string()],
        };

        // pub mod a {
        //     pub mod b {
        //         pub mod c {}
        //     }
        // }
        assert_eq!(
            doc.gen_token(&cx).unwrap().to_string(),
            "pub mod a { pub mod b { pub mod c { } } }"
        );
    }

    #[test]
    fn test_include() {
        let doc = Document::default();
        let cx = CodeGenContext {
            includes: vec!["a::b::c".to_string(), "c::d::e".to_string()],
            namespaces: vec![],
        };

        // pub use a::b::c;
        // pub use c::d::e;
        assert_eq!(
            doc.gen_token(&cx).unwrap().to_string(),
            "pub use a :: b :: c ; pub use c :: d :: e ;"
        );
    }

    #[test]
    fn test_struct() {
        let doc = Document::parse(
            "struct MyBook { 1: string author, 2: i32 price } service TestSvc { void GetName(); }",
        )
        .unwrap()
        .1;
        let cx = CodeGenContext::default();
        println!("{}", doc.gen_token(&cx).unwrap().to_string());
    }
}
