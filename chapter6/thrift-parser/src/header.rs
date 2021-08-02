use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::combinator::map;
use nom::sequence::{pair, preceded, tuple};
use nom::IResult;

use crate::basic::{Identifier, IdentifierRef, Literal, LiteralRef, Separator};
use crate::Parser;

// Include         ::=  'include' Literal
#[derive(derive_newtype::NewType, Eq, PartialEq, Debug, Clone)]
pub struct IncludeRef<'a>(LiteralRef<'a>);

impl<'a> Parser<'a> for IncludeRef<'a> {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        map(
            preceded(pair(tag("include"), Separator::parse), LiteralRef::parse),
            Self,
        )(input)
    }
}

#[derive(derive_newtype::NewType, Eq, PartialEq, Debug, Clone)]
pub struct Include(Literal);

impl<'a> From<IncludeRef<'a>> for Include {
    fn from(r: IncludeRef<'a>) -> Self {
        Self(r.0.into())
    }
}

impl<'a> Parser<'a> for Include {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        IncludeRef::parse(input).map(|(remains, parsed)| (remains, parsed.into()))
    }
}

// CppInclude      ::=  'cpp_include' Literal
#[derive(derive_newtype::NewType, Eq, PartialEq, Debug, Clone)]
pub struct CppIncludeRef<'a>(LiteralRef<'a>);

impl<'a> Parser<'a> for CppIncludeRef<'a> {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        map(
            preceded(
                pair(tag("cpp_include"), Separator::parse),
                LiteralRef::parse,
            ),
            Self,
        )(input)
    }
}

#[derive(derive_newtype::NewType, Eq, PartialEq, Debug, Clone)]
pub struct CppInclude(Literal);

impl<'a> From<CppIncludeRef<'a>> for CppInclude {
    fn from(r: CppIncludeRef<'a>) -> Self {
        Self(r.0.into())
    }
}

impl<'a> Parser<'a> for CppInclude {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        CppIncludeRef::parse(input).map(|(remains, parsed)| (remains, parsed.into()))
    }
}

// Namespace       ::=  ( 'namespace' ( NamespaceScope Identifier ) )
#[derive(Eq, PartialEq, Debug, Clone)]
pub struct NamespaceRef<'a> {
    pub scope: NamespaceScopeRef<'a>,
    pub name: IdentifierRef<'a>,
}

// NamespaceScope  ::=  '*' | 'c_glib' | 'rs' | 'cpp' | 'delphi' | 'haxe' | 'go' | 'java' |
// 'js' | 'lua' | 'netstd' | 'perl' | 'php' | 'py' | 'py.twisted' | 'rb' | 'st' | 'xsd'
// We add rust into it.
// Ref: https://github.com/apache/thrift/blob/master/lib/rs/test_recursive/src/transit/Transporters.thrift
#[derive(derive_newtype::NewType, Eq, PartialEq, Debug, Clone)]
pub struct NamespaceScopeRef<'a>(&'a str);

impl<'a> Parser<'a> for NamespaceRef<'a> {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        map(
            tuple((
                tag("namespace"),
                preceded(Separator::parse, NamespaceScopeRef::parse),
                preceded(Separator::parse, IdentifierRef::parse),
            )),
            |(_, scope, name)| Self { scope, name },
        )(input)
    }
}

impl<'a> Parser<'a> for NamespaceScopeRef<'a> {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        map(
            alt((
                tag("*"),
                tag("c_glib"),
                tag("rs"),
                tag("cpp"),
                tag("delphi"),
                tag("haxe"),
                tag("go"),
                tag("java"),
                tag("js"),
                tag("lua"),
                tag("netstd"),
                tag("perl"),
                tag("php"),
                tag("py"),
                tag("py.twisted"),
                tag("rb"),
                tag("st"),
                tag("xsd"),
            )),
            Self,
        )(input)
    }
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct Namespace {
    pub scope: NamespaceScope,
    pub name: Identifier,
}

#[derive(derive_newtype::NewType, Eq, PartialEq, Debug, Clone)]
pub struct NamespaceScope(String);

impl<'a> From<NamespaceRef<'a>> for Namespace {
    fn from(r: NamespaceRef<'a>) -> Self {
        Self {
            scope: r.scope.into(),
            name: r.name.into(),
        }
    }
}

impl<'a> From<NamespaceScopeRef<'a>> for NamespaceScope {
    fn from(r: NamespaceScopeRef<'a>) -> Self {
        Self(r.0.into())
    }
}

impl<'a> Parser<'a> for Namespace {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        NamespaceRef::parse(input).map(|(remains, parsed)| (remains, parsed.into()))
    }
}

impl<'a> Parser<'a> for NamespaceScope {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        NamespaceScopeRef::parse(input).map(|(remains, parsed)| (remains, parsed.into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_include() {
        assert_eq!(
            IncludeRef::parse("include 'another.thrift'").unwrap().1,
            IncludeRef::from(LiteralRef::from("another.thrift"))
        )
    }

    #[test]
    fn test_namespace() {
        assert_eq!(
            NamespaceRef::parse("namespace * MyNamespace").unwrap().1,
            NamespaceRef {
                scope: NamespaceScopeRef::from("*"),
                name: IdentifierRef::from("MyNamespace")
            }
        )
    }
}
