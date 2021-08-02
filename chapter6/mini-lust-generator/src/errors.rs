use std::path::PathBuf;
use crate::code_gen::errors::CodeGenError;

pub type GenerateResult<R> = Result<R, GenerateError>;

#[derive(thiserror::Error, Debug)]
pub enum GenerateError {
    #[error("io error for path {path:?}: {err:?}")]
    Io{
        err: std::io::Error,
        path: PathBuf,
    },
    #[error("invalid file path")]
    Path(#[from] std::path::StripPrefixError),
    #[error("parse failed: {0}")]
    Parse(String),
    #[error("code generate error: {0}")]
    CodeGen(#[from] CodeGenError),
    #[error("dep loop")]
    Loop(),
    #[error("env error")]
    Env(#[from] std::env::VarError),
    #[error("unknown error: {0}")]
    Unknown(String)
}
