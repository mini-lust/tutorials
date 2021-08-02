use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::char as cchar;
use nom::combinator::{map, opt};
use nom::sequence::{delimited, pair, preceded, separated_pair, terminated, tuple};
use nom::IResult;

use crate::basic::{Identifier, IdentifierRef, Literal, LiteralRef, Separator};
use crate::Parser;

// FieldType       ::=  Identifier | BaseType | ContainerType
// BaseType        ::=  'bool' | 'byte' | 'i8' | 'i16' | 'i32' | 'i64' | 'double' | 'string' | 'binary'
// ContainerType   ::=  MapType | SetType | ListType
// MapType         ::=  'map' CppType? '<' FieldType ',' FieldType '>'
// SetType         ::=  'set' CppType? '<' FieldType '>'
// ListType        ::=  'list' '<' FieldType '>' CppType?
// CppType         ::=  'cpp_type' Literal
// Note: CppType is not fully supported in out impl.
#[derive(Debug, Clone, PartialEq)]
pub enum FieldTypeRef<'a> {
    Identifier(IdentifierRef<'a>),
    Bool,
    Byte,
    I8,
    I16,
    I32,
    I64,
    Double,
    String,
    Binary,
    Map(Box<FieldTypeRef<'a>>, Box<FieldTypeRef<'a>>),
    Set(Box<FieldTypeRef<'a>>),
    List(Box<FieldTypeRef<'a>>),
}

impl<'a> FieldTypeRef<'a> {
    pub fn parse_base_type(input: &'a str) -> IResult<&'a str, Self> {
        alt((
            map(tag("bool"), |_| Self::Bool),
            map(tag("byte"), |_| Self::Byte),
            map(tag("i8"), |_| Self::I8),
            map(tag("i16"), |_| Self::I16),
            map(tag("i32"), |_| Self::I32),
            map(tag("i64"), |_| Self::I64),
            map(tag("double"), |_| Self::Double),
            map(tag("string"), |_| Self::String),
            map(tag("binary"), |_| Self::Binary),
        ))(input)
    }

    pub fn parse_container_type(input: &'a str) -> IResult<&'a str, Self> {
        alt((
            map(
                preceded(
                    tuple((
                        tag("map"),
                        opt(Separator::parse),
                        opt(terminated(CppTypeRef::parse, opt(Separator::parse))),
                    )),
                    delimited(
                        pair(cchar('<'), opt(Separator::parse)),
                        separated_pair(
                            FieldTypeRef::parse,
                            tuple((opt(Separator::parse), cchar(','), opt(Separator::parse))),
                            FieldTypeRef::parse,
                        ),
                        pair(opt(Separator::parse), cchar('>')),
                    ),
                ),
                |(k, v)| Self::Map(Box::new(k), Box::new(v)),
            ),
            map(
                preceded(
                    tuple((
                        tag("set"),
                        opt(Separator::parse),
                        opt(terminated(CppTypeRef::parse, opt(Separator::parse))),
                    )),
                    delimited(
                        pair(cchar('<'), opt(Separator::parse)),
                        FieldTypeRef::parse,
                        pair(opt(Separator::parse), cchar('>')),
                    ),
                ),
                |v| Self::Set(Box::new(v)),
            ),
            map(
                delimited(
                    pair(tag("list"), opt(Separator::parse)),
                    delimited(
                        pair(cchar('<'), opt(Separator::parse)),
                        FieldTypeRef::parse,
                        pair(opt(Separator::parse), cchar('>')),
                    ),
                    opt(pair(opt(Separator::parse), CppTypeRef::parse)),
                ),
                |v| Self::List(Box::new(v)),
            ),
            map(IdentifierRef::parse, Self::Identifier),
        ))(input)
    }

    pub fn parse_identifier_type(input: &'a str) -> IResult<&'a str, Self> {
        map(IdentifierRef::parse, Self::Identifier)(input)
    }
}

impl<'a> Parser<'a> for FieldTypeRef<'a> {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        alt((
            Self::parse_base_type,
            Self::parse_container_type,
            Self::parse_identifier_type,
        ))(input)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FieldType {
    Identifier(Identifier),
    Bool,
    Byte,
    I8,
    I16,
    I32,
    I64,
    Double,
    String,
    Binary,
    Map(Box<FieldType>, Box<FieldType>),
    Set(Box<FieldType>),
    List(Box<FieldType>),
}

impl<'a> From<FieldTypeRef<'a>> for FieldType {
    fn from(r: FieldTypeRef<'a>) -> Self {
        match r {
            FieldTypeRef::Identifier(i) => FieldType::Identifier(i.into()),
            FieldTypeRef::Bool => FieldType::Bool,
            FieldTypeRef::Byte => FieldType::Byte,
            FieldTypeRef::I8 => FieldType::I8,
            FieldTypeRef::I16 => FieldType::I16,
            FieldTypeRef::I32 => FieldType::I32,
            FieldTypeRef::I64 => FieldType::I64,
            FieldTypeRef::Double => FieldType::Double,
            FieldTypeRef::String => FieldType::String,
            FieldTypeRef::Binary => FieldType::Binary,
            FieldTypeRef::Map(k, v) => {
                FieldType::Map(Box::new(k.as_ref().into()), Box::new(v.as_ref().into()))
            }
            FieldTypeRef::Set(v) => FieldType::Set(Box::new(v.as_ref().into())),
            FieldTypeRef::List(v) => FieldType::List(Box::new(v.as_ref().into())),
        }
    }
}

impl<'a> From<&FieldTypeRef<'a>> for FieldType {
    fn from(r: &FieldTypeRef<'a>) -> Self {
        match r {
            FieldTypeRef::Identifier(i) => FieldType::Identifier(i.clone().into()),
            FieldTypeRef::Bool => FieldType::Bool,
            FieldTypeRef::Byte => FieldType::Byte,
            FieldTypeRef::I8 => FieldType::I8,
            FieldTypeRef::I16 => FieldType::I16,
            FieldTypeRef::I32 => FieldType::I32,
            FieldTypeRef::I64 => FieldType::I64,
            FieldTypeRef::Double => FieldType::Double,
            FieldTypeRef::String => FieldType::String,
            FieldTypeRef::Binary => FieldType::Binary,
            FieldTypeRef::Map(k, v) => {
                FieldType::Map(Box::new(k.as_ref().into()), Box::new(v.as_ref().into()))
            }
            FieldTypeRef::Set(v) => FieldType::Set(Box::new(v.as_ref().into())),
            FieldTypeRef::List(v) => FieldType::List(Box::new(v.as_ref().into())),
        }
    }
}

impl<'a> Parser<'a> for FieldType {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        FieldTypeRef::parse(input).map(|(remains, parsed)| (remains, parsed.into()))
    }
}

// CppType         ::=  'cpp_type' Literal
#[derive(derive_newtype::NewType, Eq, PartialEq, Debug, Clone)]
pub struct CppTypeRef<'a>(LiteralRef<'a>);

impl<'a> Parser<'a> for CppTypeRef<'a> {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        map(
            preceded(
                tag("cpp_type"),
                preceded(Separator::parse, LiteralRef::parse),
            ),
            Self,
        )(input)
    }
}
#[derive(derive_newtype::NewType, Eq, PartialEq, Debug, Clone)]
pub struct CppType(Literal);

impl<'a> From<CppTypeRef<'a>> for CppType {
    fn from(r: CppTypeRef<'a>) -> Self {
        Self(r.0.into())
    }
}

impl<'a> Parser<'a> for CppType {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        CppTypeRef::parse(input).map(|(remains, parsed)| (remains, parsed.into()))
    }
}

#[cfg(test)]
mod test {
    use crate::utils::*;

    use super::*;

    #[test]
    fn test_cpp_type() {
        assert_list_eq_with_f(
            vec!["cpp_type \"MINI-LUST\"", "cpp_type 'ihciah'"],
            vec![LiteralRef::from("MINI-LUST"), LiteralRef::from("ihciah")],
            CppTypeRef::parse,
            CppTypeRef,
        );
    }

    #[test]
    fn test_field_type() {
        assert_list_eq_with_f(
            vec!["bool", "i16"],
            vec![FieldTypeRef::Bool, FieldTypeRef::I16],
            FieldTypeRef::parse,
            |x| x,
        );
        assert_eq!(
            FieldTypeRef::parse("map <bool, bool>").unwrap().1,
            FieldTypeRef::Map(Box::new(FieldTypeRef::Bool), Box::new(FieldTypeRef::Bool))
        );
        assert_eq!(
            FieldTypeRef::parse("map<bool,bool>").unwrap().1,
            FieldTypeRef::Map(Box::new(FieldTypeRef::Bool), Box::new(FieldTypeRef::Bool))
        );
        assert_eq!(
            FieldTypeRef::parse("set <bool>").unwrap().1,
            FieldTypeRef::Set(Box::new(FieldTypeRef::Bool))
        );
        assert_eq!(
            FieldTypeRef::parse("set<bool>").unwrap().1,
            FieldTypeRef::Set(Box::new(FieldTypeRef::Bool))
        );
        assert_eq!(
            FieldTypeRef::parse("list <bool>").unwrap().1,
            FieldTypeRef::List(Box::new(FieldTypeRef::Bool))
        );
        assert_eq!(
            FieldTypeRef::parse("list<bool>").unwrap().1,
            FieldTypeRef::List(Box::new(FieldTypeRef::Bool))
        );
        assert_eq!(
            FieldTypeRef::parse("ihc_iah").unwrap().1,
            FieldTypeRef::Identifier(IdentifierRef::from("ihc_iah"))
        );
    }
}
