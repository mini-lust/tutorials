use crate::types::FieldType;
use proc_macro2::{Ident, TokenStream};

pub(crate) fn encode_content(type_: &FieldType, ident: &Ident) -> TokenStream {
    match type_ {
        FieldType::String => quote::quote! { protocol.write_string(#ident)?; },
        FieldType::Bool => quote::quote! { protocol.write_bool(*#ident)?; },
        FieldType::I8 => quote::quote! { protocol.write_i8(*#ident)?; },
        FieldType::I16 => quote::quote! { protocol.write_i16(*#ident)?; },
        FieldType::I32 => quote::quote! { protocol.write_i32(*#ident)?; },
        FieldType::I64 => quote::quote! { protocol.write_i64(*#ident)?; },
        FieldType::Double => quote::quote! { protocol.write_double(*#ident)?; },
        FieldType::Byte => quote::quote! { protocol.write_byte(*#ident)?; },
        FieldType::Ident(_) => quote::quote! { #ident.encode(cx, protocol)?; },
        FieldType::List(val) => {
            let inner = encode_content(val, &quote::format_ident!("val"));
            quote::quote! {
                protocol.write_list_begin(&::mini_lust_chap6::TListIdentifier {
                    element_type: #val,
                    size: #ident.len() as i32,
                })?;
                for val in #ident.iter() {
                    #inner
                }
                protocol.write_list_end()?;
            }
        }
        FieldType::Map(key, value) => {
            let key_inner = encode_content(key, &quote::format_ident!("key"));
            let value_inner = encode_content(value, &quote::format_ident!("value"));
            quote::quote! {
                protocol.write_map_begin(&::mini_lust_chap6::TMapIdentifier {
                    key_type: Some(#key),
                    value_type: Some(#value),
                    size: #ident.len() as i32,
                })?;
                for (key, value) in #ident.iter() {
                    #key_inner
                    #value_inner
                }
                protocol.write_map_end()?;
            }
        }
        FieldType::Set(val) => {
            let inner = encode_content(val, &quote::format_ident!("val"));
            quote::quote! {
                protocol.write_set_begin(&::mini_lust_chap6::TSetIdentifier {
                    element_type: #val,
                    size: #ident.len() as i32,
                })?;
                for val in #ident.iter() {
                    #inner
                }
                protocol.write_set_end()?;
            }
        }
        FieldType::Void => {
            quote::quote! { ().encode(cx, protocol)?; }
        }
        FieldType::Binary => {
            quote::quote! { protocol.write_bytes(&#ident)?; }
        }
    }
}

pub(crate) fn decode_content(type_: &FieldType) -> TokenStream {
    match type_ {
        FieldType::String => quote::quote! { protocol.read_string()? },
        FieldType::Bool => quote::quote! { protocol.read_bool()? },
        FieldType::I8 => quote::quote! { protocol.read_i8()? },
        FieldType::I16 => quote::quote! { protocol.read_i16()? },
        FieldType::I32 => quote::quote! { protocol.read_i32()? },
        FieldType::I64 => quote::quote! { protocol.read_i64()? },
        FieldType::Double => quote::quote! { protocol.read_double()? },
        FieldType::Byte => quote::quote! { protocol.read_byte()? },
        FieldType::Ident(ident) => quote::quote! { #ident::decode(cx, protocol)? },
        FieldType::List(val) => {
            let inner = decode_content(val);
            quote::quote! {
                {
                    let list = protocol.read_list_begin()?;
                    let mut val = ::std::vec::Vec::with_capacity(list.size as usize);
                    for i in 0..list.size {
                        let r_val = #inner;
                        val.push(r_val);
                    };
                    protocol.read_list_end()?;
                    val
                }
            }
        },
        FieldType::Map(key, value) => {
            let key = decode_content(key);
            let value = decode_content(value);
            quote::quote! {
                {
                    let map = protocol.read_map_begin()?;
                    let mut val = ::std::collections::BTreeMap::new();
                    for i in 0..map.size {
                        let r_key = #key;
                        let r_val = #value;
                        val.insert(r_key, r_val);
                    }
                    protocol.read_map_end()?;
                    val
                }
            }
        },
        FieldType::Set(val) => {
            let inner = decode_content(val);
            quote::quote! {
                {
                    let set = protocol.read_set_begin()?;
                    let mut val = ::std::collections::BTreeSet::new();
                    for i in 0..set.size {
                        let r_val = #inner;
                        val.push(r_val);
                    };
                    protocol.read_set_end()?;
                    val
                }
            }
        },
        FieldType::Void => {
            quote::quote! {
                {
                    let _: () = lust_thrift::message::Message::decode(cx, protocol)?;
                    ()
                }
            }
        },
        FieldType::Binary => quote::quote! { protocol.read_bytes()? }
    }
}