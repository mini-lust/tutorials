use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::char as cchar;
use nom::combinator::{map, opt};
use nom::multi::separated_list0;
use nom::sequence::{delimited, pair, preceded, terminated, tuple};
use nom::IResult;

use crate::basic::{Identifier, IdentifierRef, ListSeparator, Separator};
use crate::field::{Field, FieldRef};
use crate::types::{FieldType, FieldTypeRef};
use crate::Parser;

// Function        ::=  'oneway'? FunctionType Identifier '(' Field* ')' Throws? ListSeparator?
// FunctionType    ::=  FieldType | 'void'
// Throws          ::=  'throws' '(' Field* ')'
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionRef<'a> {
    pub oneway: bool,
    // returns None means void
    pub returns: Option<FieldTypeRef<'a>>,
    pub name: IdentifierRef<'a>,
    pub parameters: Vec<FieldRef<'a>>,
    pub exceptions: Option<Vec<FieldRef<'a>>>,
}

impl<'a> Parser<'a> for FunctionRef<'a> {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        map(
            tuple((
                map(opt(terminated(tag("oneway"), Separator::parse)), |x| {
                    x.is_some()
                }),
                terminated(
                    alt((map(tag("void"), |_| None), map(FieldTypeRef::parse, Some))),
                    Separator::parse,
                ),
                terminated(IdentifierRef::parse, opt(Separator::parse)),
                terminated(
                    delimited(
                        cchar('('),
                        separated_list0(Separator::parse, FieldRef::parse),
                        cchar(')'),
                    ),
                    opt(Separator::parse),
                ),
                opt(preceded(
                    pair(tag("throws"), Separator::parse),
                    delimited(
                        cchar('('),
                        separated_list0(Separator::parse, FieldRef::parse),
                        cchar(')'),
                    ),
                )),
                opt(pair(opt(Separator::parse), ListSeparator::parse)),
            )),
            |(oneway, returns, name, parameters, exceptions, _)| Self {
                oneway,
                returns,
                name,
                parameters,
                exceptions,
            },
        )(input)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub oneway: bool,
    // returns None means void
    pub returns: Option<FieldType>,
    pub name: Identifier,
    pub parameters: Vec<Field>,
    pub exceptions: Option<Vec<Field>>,
}

impl<'a> From<FunctionRef<'a>> for Function {
    fn from(r: FunctionRef<'a>) -> Self {
        Self {
            oneway: r.oneway,
            returns: r.returns.map(Into::into),
            name: r.name.into(),
            parameters: r.parameters.into_iter().map(Into::into).collect(),
            exceptions: r
                .exceptions
                .map(|x| x.into_iter().map(Into::into).collect()),
        }
    }
}

impl<'a> Parser<'a> for Function {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        FunctionRef::parse(input).map(|(remains, parsed)| (remains, parsed.into()))
    }
}

#[cfg(test)]
mod test {
    use crate::basic::LiteralRef;
    use crate::constant::{ConstValueRef, IntConstant};

    use super::*;

    #[test]
    fn test_function() {
        let expected = FunctionRef {
            oneway: false,
            returns: Some(FieldTypeRef::String),
            name: IdentifierRef::from("GetUser"),
            parameters: vec![FieldRef {
                id: None,
                required: Some(true),
                type_: FieldTypeRef::String,
                name: IdentifierRef::from("name"),
                default: Some(ConstValueRef::Literal(LiteralRef::from("ihciah"))),
            }],
            exceptions: None,
        };
        assert_eq!(
            FunctionRef::parse("string GetUser(required string name='ihciah')")
                .unwrap()
                .1,
            expected
        );

        let expected = FunctionRef {
            oneway: true,
            returns: None,
            name: IdentifierRef::from("DeleteUser"),
            parameters: vec![FieldRef {
                id: Some(IntConstant::from(10086)),
                required: Some(false),
                type_: FieldTypeRef::I32,
                name: IdentifierRef::from("age"),
                default: None,
            }],
            exceptions: None,
        };
        assert_eq!(
            FunctionRef::parse("oneway void DeleteUser(10086:optional i32 age)")
                .unwrap()
                .1,
            expected
        );
    }
}
