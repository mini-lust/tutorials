use nom::branch::alt;
use nom::combinator::{map, opt};
use nom::multi::many0;
use nom::sequence::delimited;
use nom::IResult;

use crate::basic::Separator;
use crate::definition::{
    Const, ConstRef, Enum, EnumRef, Exception, ExceptionRef, Service, ServiceRef, Struct,
    StructRef, Typedef, TypedefRef, Union, UnionRef,
};
use crate::header::{CppInclude, CppIncludeRef, Include, IncludeRef, Namespace, NamespaceRef};
use crate::Parser;

#[derive(PartialEq, Debug, Clone, Default)]
pub struct DocumentRef<'a> {
    pub includes: Vec<IncludeRef<'a>>,
    pub cpp_includes: Vec<CppIncludeRef<'a>>,
    pub namespaces: Vec<NamespaceRef<'a>>,
    pub typedefs: Vec<TypedefRef<'a>>,
    pub consts: Vec<ConstRef<'a>>,
    pub enums: Vec<EnumRef<'a>>,
    pub structs: Vec<StructRef<'a>>,
    pub unions: Vec<UnionRef<'a>>,
    pub exceptions: Vec<ExceptionRef<'a>>,
    pub services: Vec<ServiceRef<'a>>,
}

impl<'a> Parser<'a> for DocumentRef<'a> {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        let mut target = Self::default();
        let includes = &mut target.includes;
        let cpp_includes = &mut target.cpp_includes;
        let namespaces = &mut target.namespaces;
        let typedefs = &mut target.typedefs;
        let consts = &mut target.consts;
        let enums = &mut target.enums;
        let structs = &mut target.structs;
        let unions = &mut target.unions;
        let exceptions = &mut target.exceptions;
        let services = &mut target.services;

        let (remains, _) = many0(delimited(
            opt(Separator::parse),
            alt((
                map(IncludeRef::parse, |i| includes.push(i)),
                map(CppIncludeRef::parse, |i| cpp_includes.push(i)),
                map(NamespaceRef::parse, |i| namespaces.push(i)),
                map(TypedefRef::parse, |i| typedefs.push(i)),
                map(ConstRef::parse, |i| consts.push(i)),
                map(EnumRef::parse, |i| enums.push(i)),
                map(StructRef::parse, |i| structs.push(i)),
                map(UnionRef::parse, |i| unions.push(i)),
                map(ExceptionRef::parse, |i| exceptions.push(i)),
                map(ServiceRef::parse, |i| services.push(i)),
            )),
            opt(Separator::parse),
        ))(input)?;
        Ok((remains, target))
    }
}

#[derive(PartialEq, Debug, Clone, Default)]
pub struct Document {
    pub includes: Vec<Include>,
    pub cpp_includes: Vec<CppInclude>,
    pub namespaces: Vec<Namespace>,
    pub typedefs: Vec<Typedef>,
    pub consts: Vec<Const>,
    pub enums: Vec<Enum>,
    pub structs: Vec<Struct>,
    pub unions: Vec<Union>,
    pub exceptions: Vec<Exception>,
    pub services: Vec<Service>,
}

impl<'a> From<DocumentRef<'a>> for Document {
    fn from(r: DocumentRef<'a>) -> Self {
        Self {
            includes: r.includes.into_iter().map(Into::into).collect(),
            cpp_includes: r.cpp_includes.into_iter().map(Into::into).collect(),
            namespaces: r.namespaces.into_iter().map(Into::into).collect(),
            typedefs: r.typedefs.into_iter().map(Into::into).collect(),
            consts: r.consts.into_iter().map(Into::into).collect(),
            enums: r.enums.into_iter().map(Into::into).collect(),
            structs: r.structs.into_iter().map(Into::into).collect(),
            unions: r.unions.into_iter().map(Into::into).collect(),
            exceptions: r.exceptions.into_iter().map(Into::into).collect(),
            services: r.services.into_iter().map(Into::into).collect(),
        }
    }
}

impl<'a> Parser<'a> for Document {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        DocumentRef::parse(input).map(|(remains, parsed)| (remains, parsed.into()))
    }
}

#[cfg(test)]
mod tests {
    use crate::basic::LiteralRef;

    use super::*;

    #[test]
    fn test_document() {
        let expected = DocumentRef {
            includes: vec![IncludeRef::from(LiteralRef::from("another.thrift"))],
            ..Default::default()
        };
        assert_eq!(
            DocumentRef::parse("include 'another.thrift'").unwrap().1,
            expected
        );
    }
}
