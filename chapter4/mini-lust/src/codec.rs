use std::marker::PhantomData;

use bytes::{Buf, BufMut, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

use crate::binary::{TBinaryInputProtocol, TBinaryOutputProtocol};
use crate::context::MsgContext;
use crate::message::Message;
use crate::protocol::{TInputProtocol, TMessageType, TOutputProtocol};
use crate::{new_protocol_error, ApplicationError, ProtocolErrorKind};

pub struct FramedCodec<C>(C);

impl<C> FramedCodec<C> {
    #[allow(dead_code)]
    pub fn new(c: C) -> Self {
        FramedCodec(c)
    }
}

// The max message size is limited to 16M.
const MAX_MESSAGE_SIZE: usize = 16 * 1024 * 1024;

impl<C, T> Encoder<T> for FramedCodec<C>
where
    C: Encoder<T>,
    crate::Error: From<C::Error>,
{
    type Error = crate::Error;

    fn encode(&mut self, item: T, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let zero_index = dst.len();
        // Alloc 4-byte space
        dst.reserve(4);
        unsafe {
            dst.advance_mut(4);
        }
        // Call inner encoder
        self.0.encode(item, dst).map_err(Into::into)?;
        let written = dst.len() - 4 - zero_index;
        if written > MAX_MESSAGE_SIZE {
            return Err(new_protocol_error(
                ProtocolErrorKind::SizeLimit,
                format!("Frame of length {} is too large.", written),
            ));
        }
        let mut buf = &mut dst[zero_index..zero_index + 4];
        buf.put_u32(written as u32);
        Ok(())
    }
}

impl<C> Decoder for FramedCodec<C>
where
    C: Decoder,
    crate::Error: From<C::Error>,
{
    type Item = C::Item;
    type Error = crate::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < 4 {
            // Not enough data to read length marker.
            return Ok(None);
        }

        // Read length marker.
        let mut length_bytes = [0u8; 4];
        length_bytes.copy_from_slice(&src[..4]);
        let length = u32::from_be_bytes(length_bytes) as usize;

        // Check that the length is not too large to avoid a denial of
        // service attack where the server runs out of memory.
        if length > MAX_MESSAGE_SIZE {
            return Err(new_protocol_error(
                ProtocolErrorKind::SizeLimit,
                format!("Frame of length {} is too large.", length),
            ));
        }

        if src.len() < 4 + length {
            // The full string has not yet arrived.
            //
            // We reserve more space in the buffer. This is not strictly
            // necessary, but is a good idea performance-wise.
            src.reserve(4 + length - src.len());

            // We inform the Framed that we need more bytes to form the next
            // frame.
            return Ok(None);
        }

        // Skip the 4-byte length.
        src.advance(4);
        let decoded = self.0.decode(src)?;
        match decoded {
            None => Err(new_protocol_error(
                ProtocolErrorKind::InvalidData,
                "unable to decode message which the data size is enough for decoding",
            )),
            Some(inner) => Ok(Some(inner)),
        }
    }
}

#[derive(Debug)]
pub struct MessageCodec<E, D> {
    strict: bool,
    _phantom: PhantomData<fn(E, D)>,
}

impl<E, D> MessageCodec<E, D> {
    #[allow(unused)]
    pub fn new(strict: bool) -> Self {
        Self {
            strict,
            _phantom: PhantomData,
        }
    }
}

impl<E, D> Encoder<(MsgContext, Result<E, ApplicationError>)> for MessageCodec<E, D>
where
    E: Message,
{
    type Error = crate::Error;

    fn encode(
        &mut self,
        item: (MsgContext, Result<E, ApplicationError>),
        dst: &mut BytesMut,
    ) -> Result<(), Self::Error> {
        // We hard code BinaryProtocol for now. TODO: fix it
        let mut protocol = TBinaryOutputProtocol::new(dst, self.strict);
        let (mut cx, item) = item;
        match item {
            Ok(it) => {
                protocol.write_message_begin(&cx.identifier)?;
                it.encode(&cx, &mut protocol)?;
            }
            Err(err) => {
                cx.identifier.message_type = TMessageType::Exception;
                protocol.write_message_begin(&cx.identifier)?;
                err.encode(&cx, &mut protocol)?;
            }
        }
        protocol.write_message_end()?;
        Ok(())
    }
}

impl<E, D> Decoder for MessageCodec<E, D>
where
    D: Message,
{
    type Item = (MsgContext, Result<D, ApplicationError>);
    type Error = crate::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // We hard code BinaryProtocol for now. TODO: fix it
        let mut protocol = TBinaryInputProtocol::new(src, self.strict);
        let mut cx = MsgContext::default();

        cx.identifier = protocol.read_message_begin()?;
        if cx.identifier.message_type == TMessageType::Exception {
            // TODO: decode to Exception
            let exception = ApplicationError::decode(&mut cx, &mut protocol)?;
            protocol.read_message_end()?;
            return Ok(Some((cx, Err(exception))));
        }
        let item = D::decode(&mut cx, &mut protocol)?;
        protocol.read_message_end()?;
        Ok(Some((cx, Ok(item))))
    }
}

#[cfg(test)]
mod tests {
    use bytes::BufMut;
    use futures_util::{SinkExt, StreamExt};
    use tokio_util::codec::{Decoder, Encoder};

    use crate::codec::MessageCodec;
    use crate::context::MsgContext;
    use crate::message::Message;
    use crate::protocol::{
        TFieldIdentifier, TInputProtocol, TMessageIdentifier, TMessageType, TOutputProtocol,
        TStructIdentifier, TType,
    };
    use crate::{ApplicationError, ApplicationErrorKind};

    struct IdentityCodec;
    impl Encoder<bytes::Bytes> for IdentityCodec {
        type Error = crate::Error;

        fn encode(
            &mut self,
            item: bytes::Bytes,
            dst: &mut bytes::BytesMut,
        ) -> Result<(), Self::Error> {
            dst.extend(item);
            Ok(())
        }
    }
    impl Decoder for IdentityCodec {
        type Item = bytes::Bytes;
        type Error = crate::Error;

        fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
            Ok(Some(src.split().freeze()))
        }
    }

    /// test_framed_read read from mocked io with FramedCodec + IdentityCodec.
    #[tokio::test]
    async fn test_framed_read() {
        let mut data = Vec::<u8>::new();
        data.put_u32(7);
        for _ in 0..7 {
            data.put_u8(3);
        }

        let io = tokio_test::io::Builder::default()
            .read(data.as_ref())
            .build();
        let mut framed = tokio_util::codec::Framed::new(io, super::FramedCodec::new(IdentityCodec));
        let r = framed.next().await;
        assert!(
            r.is_some() && r.as_ref().unwrap().is_ok(),
            "there must be some data"
        );

        let buf = r.unwrap().unwrap();
        assert_eq!(buf.len(), 7, "the data length must matches");
        assert!(buf.iter().all(|x| *x == 3), "the data byte must matches");
    }

    /// test_framed_write write to mocked io with FramedCodec + IdentityCodec.
    #[tokio::test]
    async fn test_framed_write() {
        let mut data = Vec::<u8>::new();
        data.put_u32(7);
        for i in 0..7 {
            data.put_u8(i);
        }

        let io = tokio_test::io::Builder::default()
            .write(data.as_ref())
            .build();
        let mut framed = tokio_util::codec::Framed::new(io, super::FramedCodec::new(IdentityCodec));
        let s = framed.send(bytes::Bytes::copy_from_slice(&data[4..])).await;
        assert!(s.is_ok(), "send must be success");
    }

    #[derive(Debug, Default, Clone, Eq, PartialEq)]
    struct MockMessage {
        id: i32,
        is_male: bool,
    }
    impl Message for MockMessage {
        fn encode<T: TOutputProtocol>(
            &self,
            cx: &MsgContext,
            protocol: &mut T,
        ) -> crate::Result<()> {
            protocol.write_struct_begin(&TStructIdentifier {
                name: "MockMessage".to_string(),
            })?;

            protocol.write_field_begin(&TFieldIdentifier {
                name: Some("id".to_string()),
                field_type: TType::I32,
                id: Some(1),
            })?;
            self.id.encode(cx, protocol)?;
            protocol.write_field_end()?;

            protocol.write_field_begin(&TFieldIdentifier {
                name: Some("is_male".to_string()),
                field_type: TType::Bool,
                id: Some(2),
            })?;
            self.is_male.encode(cx, protocol)?;
            protocol.write_field_end()?;

            protocol.write_field_stop()?;
            protocol.write_struct_end()?;
            Ok(())
        }

        fn decode<T: TInputProtocol>(
            _cx: &mut MsgContext,
            protocol: &mut T,
        ) -> crate::Result<Self> {
            protocol.read_struct_begin()?;
            let mut output = Self::default();

            loop {
                let ident = protocol.read_field_begin()?;
                if ident.field_type == TType::Stop {
                    break;
                }
                match ident.id {
                    Some(1) => {
                        // read id
                        output.id = protocol.read_i32()?;
                    }
                    Some(2) => {
                        // read is_male
                        output.is_male = protocol.read_bool()?;
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

    #[test]
    fn test_message_codec() {
        let mut codec = MessageCodec::new(true);
        let mut buf = bytes::BytesMut::new();

        let cx = MsgContext {
            identifier: TMessageIdentifier {
                name: "TestMethod".to_string(),
                message_type: TMessageType::Call,
                sequence_number: 1,
            },
            ..MsgContext::default()
        };
        let message = MockMessage {
            id: 1,
            is_male: true,
        };

        codec.encode((cx, Ok(message.clone())), &mut buf).unwrap();
        let (_decoded_cx, decoded_message) = codec.decode(&mut buf).unwrap().unwrap();
        assert_eq!(message, decoded_message.unwrap());
    }

    #[test]
    fn test_message_codec_error() {
        let mut codec: MessageCodec<MockMessage, MockMessage> = MessageCodec::new(true);
        let mut buf = bytes::BytesMut::new();

        let cx = MsgContext {
            identifier: TMessageIdentifier {
                name: "TestMethod".to_string(),
                message_type: TMessageType::Exception,
                sequence_number: 1,
            },
            ..MsgContext::default()
        };
        let error = ApplicationError::new(ApplicationErrorKind::InternalError, "the error");

        codec.encode((cx, Err(error.clone())), &mut buf).unwrap();
        let (_decoded_cx, decoded_message) = codec.decode(&mut buf).unwrap().unwrap();
        assert_eq!(error, decoded_message.unwrap_err());
    }
}
