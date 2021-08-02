//! This file is meant to illustrate how to use nom to parse.

use std::str::FromStr;

use nom::branch::alt;
use nom::bytes::complete::{tag, take_while};
use nom::character::complete::{char as cchar, digit1, satisfy, space0};
use nom::combinator::{map, map_res, opt, recognize};
use nom::sequence::{preceded, tuple};
use nom::IResult;

#[derive(Debug, Clone, Copy)]
pub struct IntConstant(pub i64);

// Parse a int constant value like "+123", "123", "-123".
// IntConstant     ::=  ('+' | '-')? Digit+
fn parse_int_constant(input: &str) -> IResult<&str, IntConstant> {
    map_res(
        recognize(tuple((opt(alt((tag("-"), tag("+")))), digit1))),
        |d_str| -> Result<IntConstant, std::num::ParseIntError> {
            let d = FromStr::from_str(d_str)?;
            Ok(IntConstant(d))
        },
    )(input)
}

#[derive(Debug, Clone)]
pub struct Identifier(pub String);

// Parse a identifier like "my_age", "my_salary", "my_name".
// Identifier      ::=  ( Letter | '_' ) ( Letter | Digit | '.' | '_' )*
// Note: Identifier is not strictly following the BNF above!
// Instead, "_" and "_123" are not allowed since in rust they are invalid parameter names.
fn parse_identifier(input: &str) -> IResult<&str, Identifier> {
    map(
        recognize(tuple((
            opt(cchar('_')),
            satisfy(|c| c.is_ascii_alphabetic()),
            take_while(|c: char| c.is_ascii_alphanumeric() || c == '.' || c == '_'),
        ))),
        |ident: &str| -> Identifier { Identifier(ident.to_string()) },
    )(input)
}

#[derive(Debug, Clone)]
pub struct IntConstantExpr {
    pub name: Identifier,
    pub value: IntConstant,
}

// Parse a int const expr like "const my_age = +24", "const my_salary = 0".
// Note: This is not thrift definition, it's just for demo.
// IntConstant           ::=  'const' Identifier '=' IntConstant
fn parse_int_constant_expr(input: &str) -> IResult<&str, IntConstantExpr> {
    map(
        tuple((
            tag("const"),
            preceded(space0, parse_identifier),
            preceded(space0, tag("=")),
            preceded(space0, parse_int_constant),
        )),
        |(_, name, _, value)| -> IntConstantExpr { IntConstantExpr { name, value } },
    )(input)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_identifier() {
        assert_eq!(parse_identifier("_ihc123iah,").unwrap().1 .0, "_ihc123iah");
        assert_eq!(parse_identifier("ihc123iah,").unwrap().1 .0, "ihc123iah");
        assert!(parse_identifier("_123").is_err());
        assert!(parse_identifier("_").is_err());
        assert!(parse_identifier("123").is_err());
    }
}

fn main() {
    println!("{:?}", parse_int_constant("+123").unwrap().1);
    println!("{:?}", parse_identifier("_My.N1me").unwrap().1);
    println!("{:?}", parse_int_constant_expr("const aGe = +1").unwrap().1);
}
