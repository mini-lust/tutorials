use std::io::Write;

use quote::quote;

use thrift_parser::Parser;

pub struct SimpleBuilder {
    file: Option<std::path::PathBuf>,
}

impl SimpleBuilder {
    pub fn new() -> Self {
        Self { file: None }
    }

    pub fn with_file<P: Into<std::path::PathBuf>>(mut self, p: P) -> Self {
        self.file = Some(p.into());
        self
    }

    pub fn build(self) {
        let idl = std::fs::read_to_string(self.file.expect("idl path must be specified")).unwrap();
        let (_, document) = thrift_parser::document::Document::parse(&idl).unwrap();

        // TODO: document -> code
        let code = quote! {
            pub fn demo() -> String {
                "DEMO".to_string()
            }
        };

        // We will get OUT_DIR when build. However, in test the env not exists, so we use
        // ${CARGO_MANIFEST_DIR}/target. It's not a precise path.
        let output_dir = std::env::var("OUT_DIR")
            .unwrap_or_else(|_| std::env::var("CARGO_MANIFEST_DIR").unwrap() + "/target");
        std::fs::create_dir(&output_dir);
        let mut output_path = std::path::PathBuf::from(output_dir);
        output_path.push("gen.rs");
        let mut output_file = std::fs::File::create(output_path).unwrap();
        let _ = output_file.write(code.to_string().as_ref()).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::SimpleBuilder;

    #[test]
    fn simple_build() {
        let mut idl_path =
            std::path::PathBuf::from_str(&std::env::var("CARGO_MANIFEST_DIR").unwrap()).unwrap();
        idl_path.extend(vec!["thrift", "demo.thrift"]);
        SimpleBuilder::new().with_file(idl_path).build();
    }
}
