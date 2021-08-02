use std::str::FromStr;

use nom::branch::alt;
use nom::character::complete::{char as cchar, digit0, digit1};
use nom::combinator::{map, map_res, opt, recognize};
use nom::multi::separated_list0;
use nom::sequence::{delimited, pair, separated_pair, tuple};
use nom::IResult;

use crate::basic::{Identifier, IdentifierRef, ListSeparator, Literal, LiteralRef, Separator};
use crate::Parser;

// ConstValue      ::=  IntConstant | DoubleConstant | Literal | Identifier | ConstList | ConstMap
#[derive(Debug, Clone, PartialEq)]
pub enum ConstValueRef<'a> {
    Identifier(IdentifierRef<'a>),
    Literal(LiteralRef<'a>),
    Double(DoubleConstant),
    Int(IntConstant),
    List(ConstListRef<'a>),
    Map(ConstMapRef<'a>),
}

impl<'a> Parser<'a> for ConstValueRef<'a> {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        alt((
            map(IdentifierRef::parse, ConstValueRef::Identifier),
            map(LiteralRef::parse, ConstValueRef::Literal),
            map(DoubleConstant::parse2, ConstValueRef::Double),
            map(IntConstant::parse, ConstValueRef::Int),
            map(ConstListRef::parse, ConstValueRef::List),
            map(ConstMapRef::parse, ConstValueRef::Map),
        ))(input)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConstValue {
    Identifier(Identifier),
    Literal(Literal),
    Double(DoubleConstant),
    Int(IntConstant),
    List(ConstList),
    Map(ConstMap),
}

impl<'a> From<ConstValueRef<'a>> for ConstValue {
    fn from(r: ConstValueRef<'a>) -> Self {
        match r {
            ConstValueRef::Identifier(i) => Self::Identifier(i.into()),
            ConstValueRef::Literal(i) => Self::Literal(i.into()),
            ConstValueRef::Double(i) => Self::Double(i),
            ConstValueRef::Int(i) => Self::Int(i),
            ConstValueRef::List(i) => Self::List(i.into()),
            ConstValueRef::Map(i) => Self::Map(i.into()),
        }
    }
}

impl<'a> Parser<'a> for ConstValue {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        ConstValueRef::parse(input).map(|(remains, parsed)| (remains, parsed.into()))
    }
}

// IntConstant     ::=  ('+' | '-')? Digit+
#[derive(derive_newtype::NewType, Hash, Eq, PartialEq, Debug, Copy, Clone)]
pub struct IntConstant(i64);

impl<'a> Parser<'a> for IntConstant {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        map_res(
            recognize(tuple((opt(alt((cchar('-'), cchar('+')))), digit1))),
            |d_str| -> Result<Self, std::num::ParseIntError> {
                let d = FromStr::from_str(d_str)?;
                Ok(Self(d))
            },
        )(input)
    }
}

// DoubleConstant  ::=  ('+' | '-')? Digit* ('.' Digit+)? ( ('E' | 'e') IntConstant )?
#[derive(derive_newtype::NewType, Debug, Copy, Clone)]
pub struct DoubleConstant(f64);

impl<'a> Parser<'a> for DoubleConstant {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        map_res(
            recognize(tuple((
                opt(alt((cchar('-'), cchar('+')))),
                digit0,
                opt(pair(cchar('.'), digit1)),
                opt(pair(alt((cchar('E'), cchar('e'))), IntConstant::parse)),
            ))),
            |d_str| -> Result<Self, std::num::ParseFloatError> {
                let d = FromStr::from_str(d_str)?;
                Ok(Self(d))
            },
        )(input)
    }
}
// Double except int: If the double is indeed a int, it will fail!
impl DoubleConstant {
    fn parse2(input: &str) -> IResult<&str, Self> {
        map_res(
            recognize(tuple((
                opt(alt((cchar('-'), cchar('+')))),
                digit0,
                opt(pair(cchar('.'), digit1)),
                opt(pair(alt((cchar('E'), cchar('e'))), IntConstant::parse)),
            ))),
            |d_str| -> Result<Self, std::num::ParseFloatError> {
                if !d_str.contains('.') && !d_str.contains('e') && !d_str.contains('E') {
                    return Err(f64::from_str("").unwrap_err());
                }
                let d = FromStr::from_str(d_str)?;
                Ok(Self(d))
            },
        )(input)
    }
}

impl PartialEq for DoubleConstant {
    fn eq(&self, other: &Self) -> bool {
        float_cmp::approx_eq!(f64, self.0, other.0)
    }
}

// ConstList       ::=  '[' (ConstValue ListSeparator?)* ']'
#[derive(derive_newtype::NewType, PartialEq, Debug, Clone)]
pub struct ConstListRef<'a>(Vec<ConstValueRef<'a>>);

impl<'a> Parser<'a> for ConstListRef<'a> {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        map(
            delimited(
                pair(cchar('['), opt(Separator::parse)),
                separated_list0(parse_list_separator, ConstValueRef::parse),
                pair(opt(Separator::parse), cchar(']')),
            ),
            Self,
        )(input)
    }
}

#[derive(derive_newtype::NewType, PartialEq, Debug, Clone)]
pub struct ConstList(Vec<ConstValue>);

impl<'a> From<ConstListRef<'a>> for ConstList {
    fn from(r: ConstListRef<'a>) -> Self {
        Self(r.0.into_iter().map(Into::into).collect())
    }
}

impl<'a> Parser<'a> for ConstList {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        ConstListRef::parse(input).map(|(remains, parsed)| (remains, parsed.into()))
    }
}

// ConstMap        ::=  '{' (ConstValue ':' ConstValue ListSeparator?)* '}'
#[derive(derive_newtype::NewType, PartialEq, Debug, Clone)]
pub struct ConstMapRef<'a>(Vec<(ConstValueRef<'a>, ConstValueRef<'a>)>);

impl<'a> Parser<'a> for ConstMapRef<'a> {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        map(
            delimited(
                pair(cchar('{'), opt(Separator::parse)),
                separated_list0(
                    parse_list_separator,
                    separated_pair(
                        ConstValueRef::parse,
                        delimited(opt(Separator::parse), cchar(':'), opt(Separator::parse)),
                        ConstValueRef::parse,
                    ),
                ),
                pair(opt(Separator::parse), cchar('}')),
            ),
            Self,
        )(input)
    }
}

#[derive(derive_newtype::NewType, PartialEq, Debug, Clone)]
pub struct ConstMap(Vec<(ConstValue, ConstValue)>);

impl<'a> From<ConstMapRef<'a>> for ConstMap {
    fn from(r: ConstMapRef<'a>) -> Self {
        Self(r.0.into_iter().map(|(a, b)| (a.into(), b.into())).collect())
    }
}

impl<'a> Parser<'a> for ConstMap {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        ConstMapRef::parse(input).map(|(remains, parsed)| (remains, parsed.into()))
    }
}

// At least one Separator or ListSeparator
pub fn parse_list_separator(input: &str) -> IResult<&str, ()> {
    alt((
        map(
            tuple((
                Separator::parse,
                opt(ListSeparator::parse),
                opt(Separator::parse),
            )),
            |_| (),
        ),
        map(tuple((ListSeparator::parse, opt(Separator::parse))), |_| ()),
    ))(input)
}

#[cfg(test)]
mod test {
    use crate::utils::*;

    use super::*;

    #[test]
    fn test_int_constant() {
        assert_list_eq_with_f(
            vec!["123", "+123", "-123"],
            vec![123, 123, -123],
            IntConstant::parse,
            IntConstant,
        );
        assert_list_err_with_f(
            vec![
                "-+123",
                "+-123",
                "+",
                "-",
                "10000000000000000000000000000000000000000000000",
            ],
            IntConstant::parse,
        );
    }

    #[test]
    fn test_double_constant() {
        assert_list_eq_with_f(
            vec![
                "123.0",
                ".5",
                "-.5",
                "+123.2333333e10",
                "+123.2333333E100",
                "+123.1.THE.FOLLOWING",
                "1.1",
            ],
            vec![
                123.0,
                0.5,
                -0.5,
                123.2333333e10,
                123.2333333E100,
                123.1,
                1.1,
            ],
            DoubleConstant::parse,
            DoubleConstant,
        );
        assert_list_err_with_f(vec!["+-123.THE.FOLLOWING"], DoubleConstant::parse);
    }

    #[test]
    fn test_const_list() {
        assert_list_eq_with_f(
            vec![
                "[ 1,  3 ; 5  6/**/7 , ihciah 1.1]",
                "[6/**/7 ihciah 1.1   A ]",
                "[]",
            ],
            vec![
                vec![
                    ConstValueRef::Int(IntConstant(1)),
                    ConstValueRef::Int(IntConstant(3)),
                    ConstValueRef::Int(IntConstant(5)),
                    ConstValueRef::Int(IntConstant(6)),
                    ConstValueRef::Int(IntConstant(7)),
                    ConstValueRef::Identifier(IdentifierRef::from("ihciah")),
                    ConstValueRef::Double(DoubleConstant(1.1)),
                ],
                vec![
                    ConstValueRef::Int(IntConstant(6)),
                    ConstValueRef::Int(IntConstant(7)),
                    ConstValueRef::Identifier(IdentifierRef::from("ihciah")),
                    ConstValueRef::Double(DoubleConstant(1.1)),
                    ConstValueRef::Identifier(IdentifierRef::from("A")),
                ],
                vec![],
            ],
            ConstListRef::parse,
            ConstListRef,
        );
        assert_list_err_with_f(vec!["[1,2,3A]"], ConstListRef::parse);
    }

    #[test]
    fn test_const_map() {
        assert_list_eq_with_f(
            vec!["{1:2, 3:4}", "{}"],
            vec![
                vec![
                    (
                        ConstValueRef::Int(IntConstant(1)),
                        ConstValueRef::Int(IntConstant(2)),
                    ),
                    (
                        ConstValueRef::Int(IntConstant(3)),
                        ConstValueRef::Int(IntConstant(4)),
                    ),
                ],
                vec![],
            ],
            ConstMapRef::parse,
            ConstMapRef,
        );
        assert_list_err_with_f(vec!["{1:34:5}"], ConstMapRef::parse);
    }
}
