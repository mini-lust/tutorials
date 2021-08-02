use darling::FromDeriveInput;
use proc_macro2::TokenStream;

use crate::receiver::{enum_to_message, fields_to_message, StructReceiver};

mod types;
mod receiver;
mod fields;

#[proc_macro_derive(Message, attributes(mini_lust))]
pub fn message(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let parsed = syn::parse_macro_input!(input as syn::DeriveInput);
    let receiver = StructReceiver::from_derive_input(&parsed).unwrap();

    let name = receiver.ident;
    let generics = receiver.generics;

    let (tok_enc, tok_dec) = if receiver.data.is_struct() {
        let struct_stream = receiver.data.take_struct().unwrap();
        fields_to_message(name.clone(), struct_stream.fields)
    } else if receiver.data.is_enum() {
        let enum_stream = receiver.data.take_enum().unwrap();
        enum_to_message(name.clone(), enum_stream, receiver.dispatch_only)
    } else {
        (TokenStream::new(), TokenStream::new())
    };

    let ts2 = quote::quote! {
        impl ::mini_lust_chap6::Message for #name #generics {
            fn encode<T: ::mini_lust_chap6::TOutputProtocol>(&self, cx: &::mini_lust_chap6::MsgContext, protocol: &mut T) -> ::mini_lust_chap6::Result<()> {
                #tok_enc
            }

            fn decode<T: ::mini_lust_chap6::TInputProtocol>(cx: &mut ::mini_lust_chap6::MsgContext, protocol: &mut T) -> ::mini_lust_chap6::Result<Self> {
                #tok_dec
            }
        }
    };
    proc_macro::TokenStream::from(ts2)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test() {}
}
