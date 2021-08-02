use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use heck::SnakeCase;
use proc_macro2::TokenStream;

use thrift_parser::document::Document;
use thrift_parser::{Finish, Parser};

use crate::code_gen::{CodeGenWithContext, CodeGenContext};
use crate::errors::{GenerateError, GenerateResult};

/// Node represents a single IDL file and its meta info.
#[derive(Debug, Clone)]
pub struct Node {
    // document content, we read and parse it from file
    document: Document,
    // file_abs is IDL file abs path
    file_abs: PathBuf,
    // namespace is defined in IDL file like "a.b.c", if no namespace found, the
    // filename in snake case is used.
    namespace: String,
}

impl Node {
    /// Load node from file.
    pub fn new(file_path: &Path) -> GenerateResult<Self> {
        let file_abs = file_path.canonicalize().map_err(|e| GenerateError::Io {
            err: e,
            path: file_path.to_path_buf(),
        })?;

        let file_content = fs::read_to_string(&file_abs).map_err(|e| GenerateError::Io {
            err: e,
            path: file_abs.clone(),
        })?;
        let (_, document) = Document::parse(&file_content)
            .finish()
            .map_err(|e| GenerateError::Parse(format!("{}", e)))?;

        let (mut rs_namespace_ident, mut wildcard_namespace_ident) = (None, None);
        document
            .namespaces
            .iter()
            .for_each(|ns| match ns.scope.as_str() {
                "rs" => rs_namespace_ident = Some(ns.name.clone()),
                "*" => wildcard_namespace_ident = Some(ns.name.clone()),
                _ => {}
            });

        let namespace = match (rs_namespace_ident, wildcard_namespace_ident) {
            (Some(ns), _) => ns.into_inner(),
            (None, Some(ns)) => ns.into_inner(),
            (None, None) => file_abs
                .file_stem()
                .ok_or_else(|| {
                    GenerateError::Unknown(format!(
                        "unable to extract file stem from {:?}",
                        file_abs.clone()
                    ))
                })?
                .to_string_lossy()
                .to_string()
                .to_snake_case(),
        };

        Ok(Self {
            document,
            file_abs,
            namespace,
        })
    }

    /// Generate token recursively to output.
    pub fn generate(
        &self,
        generated: &mut HashSet<PathBuf>,
        generating: &mut Vec<PathBuf>,
        output: &mut TokenStream,
    ) -> GenerateResult<()> {
        // Loop checking.
        if generating.contains(&self.file_abs) {
            // There must be some loops...
            return Err(GenerateError::Loop());
        }

        // Bypass self if already generated.
        if generated.contains(&self.file_abs) {
            // We already generated this file!
            return Ok(());
        }

        // Mark self as generating to prevent loop.
        generating.push(self.file_abs.clone());

        let mut include_namespaces = Vec::with_capacity(self.document.includes.len());
        for inc in self.document.includes.iter() {
            // Get includes path.
            let path = PathBuf::from(inc.clone().into_inner().into_inner());
            let path_abs = if path.is_absolute() {
                path
            } else {
                self.file_abs.parent().unwrap().join(path)
            };

            // Construct includes nodes.
            let node_next = Node::new(&path_abs)?;
            // Generate includes.
            node_next.generate(generated, generating, output)?;
            // Save includes namespaces.
            include_namespaces.push(node_next.namespace);
        }

        // Create CodeGenContext.
        let context = CodeGenContext::new(include_namespaces, self.namespace.clone());
        // Generate document with the context.
        output.extend(self.document.gen_token(&context)?);

        // Moving self form generating to generated.
        generated.insert(generating.pop().unwrap());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn test_new_node() {
        let file = PathBuf::from("../mini-lust-generator/thrift/demo_base.thrift".to_string());
        let node = Node::new(&file).unwrap();
        assert_eq!(node.namespace, "demo");
    }
}
