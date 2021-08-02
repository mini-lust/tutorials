use nom::branch::alt;
use nom::bytes::complete::{tag, take_till, take_until, take_while};
use nom::character::complete::{char as cchar, multispace1, one_of, satisfy};
use nom::combinator::{map, opt, recognize};
use nom::multi::many1;
use nom::sequence::{delimited, preceded, tuple};
use nom::IResult;

use crate::Parser;

// Literal         ::=  ('"' [^"]* '"') | ("'" [^']* "'")
#[derive(derive_newtype::NewType, Eq, PartialEq, Debug, Clone)]
pub struct LiteralRef<'a>(&'a str);

impl<'a> Parser<'a> for LiteralRef<'a> {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        map(
            alt((
                delimited(cchar('"'), take_until("\""), cchar('"')),
                delimited(cchar('\''), take_until("'"), cchar('\'')),
            )),
            Self,
        )(input)
    }
}

#[derive(derive_newtype::NewType, Eq, PartialEq, Debug, Clone)]
pub struct Literal(String);

impl<'a> From<LiteralRef<'a>> for Literal {
    fn from(r: LiteralRef<'a>) -> Self {
        Self(r.0.into())
    }
}

impl<'a> Parser<'a> for Literal {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        LiteralRef::parse(input).map(|(remains, parsed)| (remains, parsed.into()))
    }
}

// Identifier      ::=  ( Letter | '_' ) ( Letter | Digit | '.' | '_' )*
#[derive(derive_newtype::NewType, Eq, PartialEq, Debug, Clone)]
pub struct IdentifierRef<'a>(&'a str);

impl<'a> Parser<'a> for IdentifierRef<'a> {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        map(
            recognize(tuple((
                opt(cchar('_')),
                satisfy(|c| c.is_ascii_alphabetic()),
                take_while(|c: char| c.is_ascii_alphanumeric() || c == '.' || c == '_'),
            ))),
            Self,
        )(input)
    }
}

#[derive(derive_newtype::NewType, Eq, PartialEq, Debug, Clone)]
pub struct Identifier(String);

impl<'a> From<IdentifierRef<'a>> for Identifier {
    fn from(r: IdentifierRef<'a>) -> Self {
        Self(r.0.into())
    }
}

impl<'a> Parser<'a> for Identifier {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        IdentifierRef::parse(input).map(|(remains, parsed)| (remains, parsed.into()))
    }
}

// ListSeparator   ::=  ',' | ';'
#[derive(Eq, PartialEq, Debug, Copy, Clone)]
pub struct ListSeparator;

impl<'a> Parser<'a> for ListSeparator {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        map(one_of(",;"), |_: char| Self)(input)
    }
}

// 1. The line begins with // or #
// 2. The content between /* and */
#[derive(Eq, PartialEq, Debug, Clone)]
pub struct CommentRef<'a>(&'a str);

impl<'a> Parser<'a> for CommentRef<'a> {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        map(
            alt((
                preceded(tag("//"), take_till(|c| c == '\n')),
                preceded(cchar('#'), take_till(|c| c == '\n')),
                delimited(tag("/*"), take_until("*/"), tag("*/")),
            )),
            Self,
        )(input)
    }
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct Comment(String);

impl<'a> From<CommentRef<'a>> for Comment {
    fn from(r: CommentRef<'a>) -> Self {
        Self(r.0.into())
    }
}

impl<'a> Parser<'a> for Comment {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        CommentRef::parse(input).map(|(remains, parsed)| (remains, parsed.into()))
    }
}

// 1. Comment
// 2. Space
#[derive(Eq, PartialEq, Debug, Copy, Clone)]
pub struct Separator;

impl<'a> Parser<'a> for Separator {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        map(
            many1(alt((
                map(CommentRef::parse, |_| ()),
                map(multispace1, |_| ()),
            ))),
            |_| Self,
        )(input)
    }
}

#[cfg(test)]
mod test {
    use crate::utils::*;

    use super::*;

    #[test]
    fn test_literal() {
        assert_list_eq_with_f(
            vec![
                "'ihciah'balabala",
                "'ihcia\"h'''''",
                "\"ihciah\"balabala",
                "\"ihcia'h\"''''",
            ],
            vec!["ihciah", "ihcia\"h", "ihciah", "ihcia'h"],
            LiteralRef::parse,
            LiteralRef,
        );
        assert_list_err_with_f(vec!["'ihcia\"aa"], LiteralRef::parse);
    }

    #[test]
    fn test_identifier() {
        assert_list_eq_with_f(
            vec!["_ihc123iah,", "ihc123iah,"],
            vec!["_ihc123iah", "ihc123iah"],
            IdentifierRef::parse,
            IdentifierRef,
        );
        assert_list_err_with_f(vec!["_123", "_", "123"], IdentifierRef::parse);
    }

    #[test]
    fn test_list_separator() {
        assert!(ListSeparator::parse(";").is_ok());
        assert!(ListSeparator::parse(",").is_ok());
        assert!(ListSeparator::parse("a").is_err());
    }
    #[test]
    fn test_comment() {
        assert_list_eq_with_f(
            vec![
                "//ihciah's #content",
                "//ihciah's #content balabala\nNextLine",
                "#ihciah's ///#content",
                "/*ihciah's con@#tent*///aaa",
            ],
            vec![
                "ihciah's #content",
                "ihciah's #content balabala",
                "ihciah's ///#content",
                "ihciah's con@#tent",
            ],
            CommentRef::parse,
            CommentRef,
        );
    }
}
