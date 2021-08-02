pub use nom::Finish;
use nom::IResult;
pub use nom::{
    error::{Error, ErrorKind},
    Err,
};

pub mod basic;
pub mod constant;
pub mod definition;
pub mod document;
pub mod field;
pub mod functions;
pub mod header;
pub mod types;
mod utils;

pub trait Parser<'a>: Sized {
    fn parse(input: &'a str) -> IResult<&'a str, Self>;
}
