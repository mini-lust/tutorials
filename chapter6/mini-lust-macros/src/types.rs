use proc_macro2::{Ident, TokenStream};
use syn::spanned::Spanned;
use syn::{Meta, NestedMeta};
use quote::ToTokens;

#[derive(Debug)]
pub(crate) enum FieldType {
    String,
    Bool,
    I8,
    I16,
    I32,
    I64,
    Double,
    Byte,
    Ident(Ident),
    List(Box<FieldType>),
    Map(Box<FieldType>, Box<FieldType>),
    Set(Box<FieldType>),
    Void,
    Binary,
}

impl ToTokens for FieldType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(self.to_token_stream());
    }
}

impl FieldType {
    pub fn to_token_stream(&self) -> TokenStream {
        match self {
            FieldType::String => {
                quote::quote! { ::mini_lust_chap6::TType::String }
            }
            FieldType::Bool => {
                quote::quote! { ::mini_lust_chap6::TType::Bool }
            }
            FieldType::I8 => {
                quote::quote! { ::mini_lust_chap6::TType::I8 }
            }
            FieldType::I16 => {
                quote::quote! { ::mini_lust_chap6::TType::I16 }
            }
            FieldType::I32 => {
                quote::quote! { ::mini_lust_chap6::TType::I32 }
            }
            FieldType::I64 => {
                quote::quote! { ::mini_lust_chap6::TType::I64 }
            }
            FieldType::Double => {
                quote::quote! { ::mini_lust_chap6::TType::Double }
            }
            FieldType::Byte => {
                quote::quote! { ::mini_lust_chap6::TType::Byte }
            }
            FieldType::Ident(ident) => {
                quote::quote! { { use mini_lust_chap6::OrigType; #ident::orig_type() }}
            }
            FieldType::List(_) => {
                quote::quote! { ::mini_lust_chap6::TType::List }
            }
            FieldType::Map(_, _) => {
                quote::quote! { ::mini_lust_chap6::TType::Map }
            }
            FieldType::Set(_) => {
                quote::quote! { ::mini_lust_chap6::TType::Set }
            }
            FieldType::Void => {
                quote::quote! { ::mini_lust_chap6::TType::Void }
            }
            FieldType::Binary => {
                quote::quote! { ::mini_lust_chap6::TType::String }
            }
        }
    }

    pub fn parse(s: &str) -> Result<Self, syn::Error> {
        let t: Meta = syn::parse_str(s)?;
        Self::parse_meta(&t)
    }

    fn parse_meta(m: &Meta) -> Result<Self, syn::Error> {
        let span = m.span().to_owned();
        match m {
            Meta::Path(p) => match p.segments.first() {
                Some(seg) if seg.ident == "string" => Ok(Self::String),
                Some(seg) if seg.ident == "bool" => Ok(Self::Bool),
                Some(seg) if seg.ident == "i8" => Ok(Self::I8),
                Some(seg) if seg.ident == "i16" => Ok(Self::I16),
                Some(seg) if seg.ident == "i32" => Ok(Self::I32),
                Some(seg) if seg.ident == "i64" => Ok(Self::I64),
                Some(seg) if seg.ident == "double" => Ok(Self::Double),
                Some(seg) if seg.ident == "byte" => Ok(Self::Byte),
                Some(seg) if seg.ident == "void" => Ok(Self::Void),
                _ => {
                    return Err(syn::Error::new(span, ""));
                }
            },
            Meta::List(l) => match l.path.segments.first() {
                Some(seg) if seg.ident == "ident" => match l.nested.first() {
                    Some(NestedMeta::Meta(Meta::Path(p))) => {
                        if let Some(inner) = p.segments.first() {
                            Ok(Self::Ident(inner.ident.clone()))
                        } else {
                            return Err(syn::Error::new(span, ""));
                        }
                    },
                    _ => {
                        return Err(syn::Error::new(span, ""));
                    }
                },
                Some(seg) if seg.ident == "list" => match l.nested.first() {
                    Some(NestedMeta::Meta(m)) => {
                        let inner = Self::parse_meta(m)?;
                        Ok(Self::List(Box::new(inner)))
                    }
                    _ => {
                        return Err(syn::Error::new(span, ""));
                    }
                },
                Some(seg) if seg.ident == "set" => match l.nested.first() {
                    Some(NestedMeta::Meta(m)) => {
                        let inner = Self::parse_meta(m)?;
                        Ok(Self::Set(Box::new(inner)))
                    }
                    _ => {
                        return Err(syn::Error::new(span, ""));
                    }
                },
                Some(seg) if seg.ident == "map" => {
                    match (l.nested.first(), l.nested.iter().nth(1)) {
                        (Some(NestedMeta::Meta(k)), Some(NestedMeta::Meta(v))) => {
                            let key = Self::parse_meta(k)?;
                            let value = Self::parse_meta(v)?;
                            Ok(Self::Map(Box::new(key), Box::new(value)))
                        }
                        _ => {
                            return Err(syn::Error::new(span, ""));
                        }
                    }
                }
                _ => Err(syn::Error::new(span, "")),
            },
            Meta::NameValue(_) => {
                return Err(syn::Error::new(span, ""));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let t: Meta = syn::parse_str("a(b(c, d))").unwrap();
        println!("{:?}", t);
        let t: Meta = syn::parse_str("a(\"p\")").unwrap();
        println!("{:?}", t);
    }

    #[test]
    fn test_parse_field_type() {
        println!("{:?}", FieldType::parse("string").unwrap());
        println!("{:?}", FieldType::parse("ident(my_name)").unwrap());
        println!("{:?}", FieldType::parse("map(string, byte)").unwrap());
        println!("{:?}", FieldType::parse("list(i32)").unwrap());
        println!(
            "{:?}",
            FieldType::parse("list(map(i8, map(i16, i32)))").unwrap()
        );
    }
}
