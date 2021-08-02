use std::convert::TryInto;

use crate::context::MsgContext;
use crate::protocol::{
    TFieldIdentifier, TInputProtocol, TOutputProtocol, TStructIdentifier, TType,
};
use crate::{new_application_error, ApplicationError, ApplicationErrorKind, Error};

pub trait Message: Sized {
    fn encode<T: TOutputProtocol>(&self, cx: &MsgContext, protocol: &mut T) -> crate::Result<()>;
    fn decode<T: TInputProtocol>(cx: &mut MsgContext, protocol: &mut T) -> crate::Result<Self>;
}

/// ApplicationError defined as:
/// exception TApplicationException {
///   1: string message,
///   2: i32 type
/// }
///
impl Message for ApplicationError {
    fn encode<T: TOutputProtocol>(&self, _cx: &MsgContext, protocol: &mut T) -> crate::Result<()> {
        protocol.write_struct_begin(&TStructIdentifier {
            name: "ApplicationError".to_string(),
        })?;
        protocol.write_field_begin(&TFieldIdentifier {
            name: Some("message".to_string()),
            field_type: TType::Struct,
            id: Some(1),
        })?;
        protocol.write_string(self.message.as_str())?;
        protocol.write_field_end()?;

        protocol.write_field_begin(&TFieldIdentifier {
            name: Some("type".to_string()),
            field_type: TType::I32,
            id: Some(2),
        })?;
        protocol.write_i32(self.kind.into())?;
        protocol.write_field_end()?;
        protocol.write_field_stop()?;
        protocol.write_struct_end()?;
        Ok(())
    }

    fn decode<T: TInputProtocol>(cx: &mut MsgContext, protocol: &mut T) -> crate::Result<Self> {
        protocol.read_struct_begin()?;
        let mut output = Self::default();

        loop {
            let ident = protocol.read_field_begin()?;
            if ident.field_type == TType::Stop {
                break;
            }
            match ident.id {
                Some(1) => {
                    // read string
                    output.message = String::decode(cx, protocol)?;
                }
                Some(2) => {
                    // read i32
                    output.kind = protocol.read_i32()?.try_into()?;
                }
                _ => {
                    protocol.skip(ident.field_type)?;
                }
            }
            protocol.read_field_end()?;
        }
        protocol.read_struct_end()?;
        Ok(output)
    }
}

impl Message for Error {
    fn encode<T: TOutputProtocol>(&self, _cx: &MsgContext, _protocol: &mut T) -> crate::Result<()> {
        // TODO: encode error
        Ok(())
    }

    fn decode<T: TInputProtocol>(_cx: &mut MsgContext, _protocol: &mut T) -> crate::Result<Self> {
        // TODO: decode error
        Ok(new_application_error(ApplicationErrorKind::Unknown, "mock"))
    }
}

macro_rules! impl_message {
    ($e: ty, $r: ident, $w: ident) => {
        impl Message for $e {
            fn encode<T: TOutputProtocol>(
                &self,
                _cx: &MsgContext,
                protocol: &mut T,
            ) -> crate::Result<()> {
                protocol.$w(self)?;
                Ok(())
            }

            fn decode<T: TInputProtocol>(
                _cx: &mut MsgContext,
                protocol: &mut T,
            ) -> crate::Result<Self> {
                protocol.$r()
            }
        }
    };
}

macro_rules! impl_message_deref {
    ($e: ty, $r: ident, $w: ident) => {
        impl Message for $e {
            fn encode<T: TOutputProtocol>(
                &self,
                _cx: &MsgContext,
                protocol: &mut T,
            ) -> crate::Result<()> {
                protocol.$w(*self)?;
                Ok(())
            }

            fn decode<T: TInputProtocol>(
                _cx: &mut MsgContext,
                protocol: &mut T,
            ) -> crate::Result<Self> {
                protocol.$r()
            }
        }
    };
}

impl_message_deref!(bool, read_bool, write_bool);
impl_message_deref!(i8, read_i8, write_i8);
impl_message_deref!(i16, read_i16, write_i16);
impl_message_deref!(i32, read_i32, write_i32);
impl_message_deref!(i64, read_i64, write_i64);
impl_message!(String, read_string, write_string);
