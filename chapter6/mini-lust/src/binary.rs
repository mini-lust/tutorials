use std::convert::TryFrom;

use byteorder::{BigEndian, ByteOrder};

use crate::protocol::{
    field_type_from_u8, TFieldIdentifier, TInputProtocol, TListIdentifier, TMapIdentifier,
    TMessageIdentifier, TMessageType, TOutputProtocol, TSetIdentifier, TStructIdentifier, TType,
};
use crate::{new_protocol_error, ProtocolError, ProtocolErrorKind};

const BINARY_PROTOCOL_VERSION_1: u32 = 0x8001_0000;

// Mini-lust note: warp a BufMut to reuse buffer and do buffered packet parsing.
#[derive(Debug)]
pub struct TBinaryOutputProtocol<T> {
    buf: T,
    strict: bool,
}

impl<T> TBinaryOutputProtocol<T> {
    #[allow(dead_code)]
    pub fn new(buf: T, strict: bool) -> TBinaryOutputProtocol<T> {
        TBinaryOutputProtocol { buf, strict }
    }
}

// Mini-lust note: Mainly copied from official thrift code
impl<T> TOutputProtocol for TBinaryOutputProtocol<T>
where
    T: bytes::BufMut,
{
    #[inline]
    fn write_message_begin(&mut self, identifier: &TMessageIdentifier) -> crate::Result<()> {
        if self.strict {
            let message_type: u8 = identifier.message_type.into();
            let header = BINARY_PROTOCOL_VERSION_1 | (message_type as u32);
            self.buf.put_u32(header);
            self.write_string(&identifier.name)?;
            self.write_i32(identifier.sequence_number)
        } else {
            self.write_string(&identifier.name)?;
            self.write_byte(identifier.message_type.into())?;
            self.write_i32(identifier.sequence_number)
        }
    }

    #[inline]
    fn write_message_end(&mut self) -> crate::Result<()> {
        Ok(())
    }

    #[inline]
    fn write_struct_begin(&mut self, _: &TStructIdentifier) -> crate::Result<()> {
        Ok(())
    }

    #[inline]
    fn write_struct_end(&mut self) -> crate::Result<()> {
        Ok(())
    }

    #[inline]
    fn write_field_begin(&mut self, identifier: &TFieldIdentifier) -> crate::Result<()> {
        if identifier.id.is_none() && identifier.field_type != TType::Stop {
            return Err(crate::Error::Protocol(ProtocolError {
                kind: ProtocolErrorKind::Unknown,
                message: format!(
                    "cannot write identifier {:?} without sequence number",
                    &identifier
                ),
            }));
        }

        self.write_byte(identifier.field_type.into())?;
        if let Some(id) = identifier.id {
            self.write_i16(id)
        } else {
            Ok(())
        }
    }

    #[inline]
    fn write_field_end(&mut self) -> crate::Result<()> {
        Ok(())
    }

    #[inline]
    fn write_field_stop(&mut self) -> crate::Result<()> {
        self.write_byte(TType::Stop.into())
    }

    #[inline]
    fn write_bool(&mut self, b: bool) -> crate::Result<()> {
        if b {
            self.write_i8(1)
        } else {
            self.write_i8(0)
        }
    }

    #[inline]
    fn write_bytes(&mut self, b: &[u8]) -> crate::Result<()> {
        self.write_i32(b.len() as i32)?;
        self.buf.put_slice(b);
        Ok(())
    }

    #[inline]
    fn write_i8(&mut self, i: i8) -> crate::Result<()> {
        self.buf.put_i8(i);
        Ok(())
    }

    #[inline]
    fn write_i16(&mut self, i: i16) -> crate::Result<()> {
        self.buf.put_i16(i);
        Ok(())
    }

    #[inline]
    fn write_i32(&mut self, i: i32) -> crate::Result<()> {
        self.buf.put_i32(i);
        Ok(())
    }

    #[inline]
    fn write_i64(&mut self, i: i64) -> crate::Result<()> {
        self.buf.put_i64(i);
        Ok(())
    }

    #[inline]
    fn write_double(&mut self, d: f64) -> crate::Result<()> {
        self.buf.put_f64(d);
        Ok(())
    }

    #[inline]
    fn write_string(&mut self, s: &str) -> crate::Result<()> {
        self.write_bytes(s.as_bytes())?;
        Ok(())
    }

    #[inline]
    fn write_list_begin(&mut self, identifier: &TListIdentifier) -> crate::Result<()> {
        self.write_byte(identifier.element_type.into())?;
        self.write_i32(identifier.size)
    }

    #[inline]
    fn write_list_end(&mut self) -> crate::Result<()> {
        Ok(())
    }

    #[inline]
    fn write_set_begin(&mut self, identifier: &TSetIdentifier) -> crate::Result<()> {
        self.write_byte(identifier.element_type.into())?;
        self.write_i32(identifier.size)
    }

    #[inline]
    fn write_set_end(&mut self) -> crate::Result<()> {
        Ok(())
    }

    #[inline]
    fn write_map_begin(&mut self, identifier: &TMapIdentifier) -> crate::Result<()> {
        let key_type = identifier
            .key_type
            .expect("map identifier to write should contain key type");
        self.write_byte(key_type.into())?;
        let val_type = identifier
            .value_type
            .expect("map identifier to write should contain value type");
        self.write_byte(val_type.into())?;
        self.write_i32(identifier.size)
    }

    #[inline]
    fn write_map_end(&mut self) -> crate::Result<()> {
        Ok(())
    }

    #[inline]
    fn flush(&mut self) -> crate::Result<()> {
        Ok(())
    }

    // utility
    //

    #[inline]
    fn write_byte(&mut self, b: u8) -> crate::Result<()> {
        self.buf.put_u8(b);
        Ok(())
    }
}

// Mini-lust note: warp a BufMut to reuse buffer and do buffered packet parsing.
#[derive(Debug)]
pub struct TBinaryInputProtocol<T> {
    buf: T,
    strict: bool,
}

// Mini-lust note: Mainly copied from official thrift code
impl<T> TBinaryInputProtocol<T> {
    #[allow(unused)]
    pub fn new(buf: T, strict: bool) -> TBinaryInputProtocol<T> {
        TBinaryInputProtocol { buf, strict }
    }
}

impl<T> TInputProtocol for TBinaryInputProtocol<T>
where
    T: bytes::Buf,
{
    #[allow(clippy::collapsible_if)]
    #[inline]
    fn read_message_begin(&mut self) -> crate::Result<TMessageIdentifier> {
        let first_bytes = self.buf.copy_to_bytes(4);

        // the thrift version header is intentionally negative
        // so the first check we'll do is see if the sign bit is set
        // and if so - assume it's the protocol-version header
        if first_bytes[0] >= 0x80 {
            // apparently we got a protocol-version header - check
            // it, and if it matches, read the rest of the fields
            if first_bytes[0..2] != [0x80, 0x01] {
                Err(crate::Error::Protocol(ProtocolError {
                    kind: ProtocolErrorKind::BadVersion,
                    message: format!("received bad version: {:?}", &first_bytes[0..2]),
                }))
            } else {
                let message_type: TMessageType = TryFrom::try_from(first_bytes[3])?;
                let name = self.read_string()?;
                let sequence_number = self.read_i32()?;
                Ok(TMessageIdentifier::new(name, message_type, sequence_number))
            }
        } else {
            // apparently we didn't get a protocol-version header,
            // which happens if the sender is not using the strict protocol
            if self.strict {
                // we're in strict mode however, and that always
                // requires the protocol-version header to be written first
                Err(crate::Error::Protocol(ProtocolError {
                    kind: ProtocolErrorKind::BadVersion,
                    message: format!("received bad version: {:?}", &first_bytes[0..2]),
                }))
            } else {
                // in the non-strict version the first message field
                // is the message name. strings (byte arrays) are length-prefixed,
                // so we've just read the length in the first 4 bytes
                let name_size = BigEndian::read_i32(&first_bytes) as usize;
                let mut name_buf: Vec<u8> = vec![0; name_size];
                self.buf.copy_to_slice(&mut name_buf);
                let name = String::from_utf8(name_buf)?;

                // read the rest of the fields
                let message_type: TMessageType = self.read_byte().and_then(TryFrom::try_from)?;
                let sequence_number = self.read_i32()?;
                Ok(TMessageIdentifier::new(name, message_type, sequence_number))
            }
        }
    }

    #[inline]
    fn read_message_end(&mut self) -> crate::Result<()> {
        Ok(())
    }

    #[inline]
    fn read_struct_begin(&mut self) -> crate::Result<Option<TStructIdentifier>> {
        Ok(None)
    }

    #[inline]
    fn read_struct_end(&mut self) -> crate::Result<()> {
        Ok(())
    }

    #[inline]
    fn read_field_begin(&mut self) -> crate::Result<TFieldIdentifier> {
        let field_type_byte = self.read_byte()?;
        let field_type: TType = field_type_from_u8(field_type_byte)?;
        let id = match field_type {
            TType::Stop => Ok(0),
            _ => self.read_i16(),
        }?;
        Ok(TFieldIdentifier::new::<Option<String>, String, i16>(
            None, field_type, id,
        ))
    }

    #[inline]
    fn read_field_end(&mut self) -> crate::Result<()> {
        Ok(())
    }

    #[inline]
    fn read_bool(&mut self) -> crate::Result<bool> {
        let b = self.read_i8()?;
        match b {
            0 => Ok(false),
            _ => Ok(true),
        }
    }

    #[inline]
    fn read_bytes(&mut self) -> crate::Result<Vec<u8>> {
        protocol_len_check(&self.buf, 4)?;
        let num_bytes = self.buf.get_i32() as usize;
        let mut output = vec![0; num_bytes];
        protocol_len_check(&self.buf, num_bytes)?;
        self.buf.copy_to_slice(&mut output);
        Ok(output)
    }

    #[inline]
    fn read_i8(&mut self) -> crate::Result<i8> {
        protocol_len_check(&self.buf, 1)?;
        Ok(self.buf.get_i8())
    }

    #[inline]
    fn read_i16(&mut self) -> crate::Result<i16> {
        protocol_len_check(&self.buf, 2)?;
        Ok(self.buf.get_i16())
    }

    #[inline]
    fn read_i32(&mut self) -> crate::Result<i32> {
        protocol_len_check(&self.buf, 4)?;
        Ok(self.buf.get_i32())
    }

    #[inline]
    fn read_i64(&mut self) -> crate::Result<i64> {
        protocol_len_check(&self.buf, 8)?;
        Ok(self.buf.get_i64())
    }

    #[inline]
    fn read_double(&mut self) -> crate::Result<f64> {
        protocol_len_check(&self.buf, 8)?;
        Ok(self.buf.get_f64())
    }

    #[inline]
    fn read_string(&mut self) -> crate::Result<String> {
        let bytes = self.read_bytes()?;
        String::from_utf8(bytes).map_err(From::from)
    }

    #[inline]
    fn read_list_begin(&mut self) -> crate::Result<TListIdentifier> {
        let element_type: TType = self.read_byte().and_then(field_type_from_u8)?;
        let size = self.read_i32()?;
        Ok(TListIdentifier::new(element_type, size))
    }

    #[inline]
    fn read_list_end(&mut self) -> crate::Result<()> {
        Ok(())
    }

    #[inline]
    fn read_set_begin(&mut self) -> crate::Result<TSetIdentifier> {
        let element_type: TType = self.read_byte().and_then(field_type_from_u8)?;
        let size = self.read_i32()?;
        Ok(TSetIdentifier::new(element_type, size))
    }

    #[inline]
    fn read_set_end(&mut self) -> crate::Result<()> {
        Ok(())
    }

    #[inline]
    fn read_map_begin(&mut self) -> crate::Result<TMapIdentifier> {
        let key_type: TType = self.read_byte().and_then(field_type_from_u8)?;
        let value_type: TType = self.read_byte().and_then(field_type_from_u8)?;
        let size = self.read_i32()?;
        Ok(TMapIdentifier::new(key_type, value_type, size))
    }

    #[inline]
    fn read_map_end(&mut self) -> crate::Result<()> {
        Ok(())
    }

    // utility
    //

    #[inline]
    fn read_byte(&mut self) -> crate::Result<u8> {
        protocol_len_check(&self.buf, 1)?;
        Ok(self.buf.get_u8())
    }
}

#[inline]
fn protocol_len_check<T>(buf: &T, required_len: usize) -> crate::Result<()>
where
    T: bytes::Buf,
{
    #[cfg(not(feature = "unstable"))]
    if buf.remaining() >= required_len {
        return Ok(());
    }
    #[cfg(feature = "unstable")]
    if std::intrinsics::likely(buf.remaining() >= required_len) {
        return Ok(());
    }
    Err(new_protocol_error(
        ProtocolErrorKind::InvalidData,
        "unexpected data length",
    ))
}
