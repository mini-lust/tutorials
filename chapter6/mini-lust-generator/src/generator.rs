use std::collections::HashSet;
use std::io::Write;

use proc_macro2::TokenStream;

use crate::errors::{GenerateError, GenerateResult};
use crate::node::Node;

pub struct Generator {
    // Input idls path
    idls: Vec<std::path::PathBuf>,
    // Output file path
    output: Option<std::path::PathBuf>,
}

impl Default for Generator {
    fn default() -> Self {
        Self::new()
    }
}

impl Generator {
    /// Create a generator
    pub fn new() -> Self {
        Self {
            idls: Default::default(),
            output: Default::default(),
        }
    }

    /// Add IDL file
    pub fn add_idl<P: Into<std::path::PathBuf>>(mut self, p: P) -> Self {
        self.idls.push(p.into());
        self
    }

    /// Set output file path
    pub fn output<P: Into<std::path::PathBuf>>(mut self, p: P) -> Self {
        self.output = Some(p.into());
        self
    }

    /// Generate all IDLs
    pub fn generate_token_stream(self) -> GenerateResult<TokenStream> {
        let mut output = TokenStream::new();
        let (mut generated, mut generating) = (HashSet::new(), Vec::new());
        for idl in self.idls {
            let node = Node::new(&idl)?;
            let _ = node.generate(&mut generated, &mut generating, &mut output)?;
        }
        Ok(output)
    }

    /// Generate all IDLs and write to file
    pub fn generate(self) -> GenerateResult<()> {
        let output = self.output.clone();

        // Generate token stream.
        let token_stream = self.generate_token_stream()?;

        // Get output file path form env or user specified.
        let output_file_path = match output {
            None => {
                // We will get OUT_DIR when build(It's a precise path).
                // However, in test the env not exists, so we use
                // ${CARGO_MANIFEST_DIR}/target(It's not a precise path).
                let output_dir = match std::env::var("OUT_DIR") {
                    Ok(p) => p,
                    Err(_) => std::env::var("CARGO_MANIFEST_DIR")? + "/target",
                };
                let _ = std::fs::create_dir(&output_dir);
                let mut output_path = std::path::PathBuf::from(output_dir);
                output_path.push("generated.rs");
                output_path
            }
            Some(p) => p,
        };

        // Open output file.
        let mut output_file =
            std::fs::File::create(output_file_path.clone()).map_err(|err| GenerateError::Io {
                err,
                path: output_file_path.clone(),
            })?;

        // Write token stream to output file.
        output_file
            .write(token_stream.to_string().as_ref())
            .map_err(|err| GenerateError::Io {
                err,
                path: output_file_path,
            })?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::generator::Generator;

    #[test]
    fn test_gen() {
        let g = Generator::new();
        let ts = g
            .add_idl("../mini-lust-generator/thrift/demo_base.thrift")
            .generate_token_stream()
            .unwrap();
        println!("{}", ts.to_string());
    }

    #[test]
    fn test_gen_demo() {
        let g = Generator::new();
        let ts = g
            .add_idl("../mini-lust-generator/thrift/demo.thrift")
            .generate_token_stream()
            .unwrap();
        println!("{}", ts.to_string());
    }
}
