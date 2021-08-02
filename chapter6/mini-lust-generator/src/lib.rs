mod code_gen;
mod errors;
mod generator;
mod node;

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use thrift_parser::Parser;

    #[test]
    fn parse_idl() {
        let mut idl_path =
            std::path::PathBuf::from_str(&std::env::var("CARGO_MANIFEST_DIR").unwrap()).unwrap();
        idl_path.extend(vec!["thrift", "demo_base.thrift"]);
        let idl = std::fs::read_to_string(idl_path).unwrap();
        let (remains, document) = thrift_parser::document::DocumentRef::parse(&idl).unwrap();
        println!("Parser remains: {:?}, document: {:?}", remains, document);
    }
}
