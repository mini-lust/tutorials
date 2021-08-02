pub type CodeGenResult<T> = Result<T, CodeGenError>;

#[derive(thiserror::Error, Debug)]
pub enum CodeGenError {}
