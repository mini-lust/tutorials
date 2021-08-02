use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::char as cchar;
use nom::combinator::{map, opt};
use nom::sequence::{delimited, terminated, tuple};
use nom::IResult;

use crate::basic::{Identifier, IdentifierRef, ListSeparator, Separator};
use crate::constant::{ConstValue, ConstValueRef, IntConstant};
use crate::types::{FieldType, FieldTypeRef};
use crate::Parser;

// Field           ::=  FieldID? FieldReq? FieldType Identifier ('=' ConstValue)? ListSeparator?
// FieldID         ::=  IntConstant ':'
// FieldReq        ::=  'required' | 'optional'
// Note: XsdFieldOptions is not supported in out impl and strongly discouraged in official docs.
#[derive(Debug, Clone, PartialEq)]
pub struct FieldRef<'a> {
    pub id: Option<IntConstant>,
    pub required: Option<bool>,
    pub type_: FieldTypeRef<'a>,
    pub name: IdentifierRef<'a>,
    pub default: Option<ConstValueRef<'a>>,
}

impl<'a> Parser<'a> for FieldRef<'a> {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        map(
            tuple((
                opt(terminated(
                    IntConstant::parse,
                    delimited(opt(Separator::parse), cchar(':'), opt(Separator::parse)),
                )),
                opt(terminated(
                    alt((
                        map(tag("required"), |_| true),
                        map(tag("optional"), |_| false),
                    )),
                    Separator::parse,
                )),
                terminated(FieldTypeRef::parse, Separator::parse),
                terminated(IdentifierRef::parse, opt(Separator::parse)),
                opt(map(
                    tuple((cchar('='), opt(Separator::parse), ConstValueRef::parse)),
                    |(_, _, cv)| cv,
                )),
                opt(Separator::parse),
                opt(ListSeparator::parse),
            )),
            |(id, required, type_, name, default, _, _)| Self {
                id,
                required,
                type_,
                name,
                default,
            },
        )(input)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub id: Option<IntConstant>,
    pub required: Option<bool>,
    pub type_: FieldType,
    pub name: Identifier,
    pub default: Option<ConstValue>,
}

impl<'a> From<FieldRef<'a>> for Field {
    fn from(r: FieldRef<'a>) -> Self {
        Self {
            id: r.id,
            required: r.required,
            type_: r.type_.into(),
            name: r.name.into(),
            default: r.default.map(Into::into),
        }
    }
}

impl<'a> Parser<'a> for Field {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        FieldRef::parse(input).map(|(remains, parsed)| (remains, parsed.into()))
    }
}

#[cfg(test)]
mod test {
    use crate::basic::LiteralRef;

    use super::*;

    #[test]
    fn test_field() {
        let expected = FieldRef {
            id: None,
            required: Some(true),
            type_: FieldTypeRef::String,
            name: IdentifierRef::from("name"),
            default: Some(ConstValueRef::Literal(LiteralRef::from("ihciah"))),
        };
        assert_eq!(
            FieldRef::parse("required  string  name  =  'ihciah'")
                .unwrap()
                .1,
            expected
        );
        assert_eq!(
            FieldRef::parse("required string name='ihciah';").unwrap().1,
            expected
        );

        let expected = FieldRef {
            id: Some(IntConstant::from(3)),
            required: Some(true),
            type_: FieldTypeRef::String,
            name: IdentifierRef::from("name"),
            default: Some(ConstValueRef::Literal(LiteralRef::from("ihciah"))),
        };
        assert_eq!(
            FieldRef::parse("3 : required  string  name  =  'ihciah'")
                .unwrap()
                .1,
            expected
        );
        assert_eq!(
            FieldRef::parse("3:required string name='ihciah';")
                .unwrap()
                .1,
            expected
        );
    }
}
