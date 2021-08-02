use std::collections::HashMap;

// Export the error
pub use errors::{Error, *};

use crate::context::MsgContext;
use crate::message::Message;
use crate::protocol::{
    TFieldIdentifier, TInputProtocol, TListIdentifier, TMapIdentifier, TOutputProtocol,
    TStructIdentifier, TType,
};

// Export the result alias
pub type Result<T> = std::result::Result<T, self::Error>;

mod binary;
mod context;
mod errors;
mod message;
mod protocol;

/// Generated(manually) User struct
#[derive(Debug, Clone, Default, PartialEq)]
pub struct User {
    pub user_id: i32,
    pub user_name: String,
    pub is_male: bool,

    pub extra: Option<HashMap<String, String>>,
}

/// Generated(manually) GetUserRequest struct
#[derive(Debug, Clone, Default, PartialEq)]
pub struct GetUserRequest {
    pub user_id: i32,
    pub user_name: String,
    pub is_male: bool,
}

/// Generated(manually) GetUserResponse struct
#[derive(Debug, Clone, Default, PartialEq)]
pub struct GetUserResponse {
    pub users: Vec<User>,
}

/// Generated(manually) impl Message for User
impl Message for User {
    fn encode<T: TOutputProtocol>(&self, _cx: &MsgContext, protocol: &mut T) -> Result<()> {
        protocol.write_struct_begin(&TStructIdentifier {
            name: "User".to_string(),
        })?;

        // user_id
        protocol.write_field_begin(&TFieldIdentifier {
            name: Some("user_id".to_string()),
            field_type: TType::I32,
            id: Some(1),
        })?;
        protocol.write_i32(self.user_id)?;
        protocol.write_field_end()?;

        // user_name
        protocol.write_field_begin(&TFieldIdentifier {
            name: Some("user_name".to_string()),
            field_type: TType::String,
            id: Some(2),
        })?;
        protocol.write_string(&self.user_name)?;
        protocol.write_field_end()?;

        // is_male
        protocol.write_field_begin(&TFieldIdentifier {
            name: Some("is_male".to_string()),
            field_type: TType::Bool,
            id: Some(3),
        })?;
        protocol.write_bool(self.is_male)?;
        protocol.write_field_end()?;

        // extra
        if let Some(extra) = self.extra.as_ref() {
            protocol.write_field_begin(&TFieldIdentifier {
                name: Some("extra".to_string()),
                field_type: TType::Map,
                id: Some(10),
            })?;
            protocol.write_map_begin(&TMapIdentifier {
                key_type: Some(TType::String),
                value_type: Some(TType::String),
                size: extra.len() as i32,
            })?;

            // key-values
            for (k, v) in extra.iter() {
                protocol.write_string(k)?;
                protocol.write_string(v)?;
            }

            protocol.write_map_end()?;
            protocol.write_field_end()?;
        }

        protocol.write_field_stop()?;
        protocol.write_struct_end()?;
        Ok(())
    }

    fn decode<T: TInputProtocol>(_cx: &mut MsgContext, protocol: &mut T) -> Result<Self> {
        let mut output = Self::default();
        protocol.read_struct_begin()?;

        loop {
            let ident = protocol.read_field_begin()?;
            if ident.field_type == TType::Stop {
                break;
            }
            match ident.id {
                Some(1) => {
                    ttype_comparing(ident.field_type, TType::I32)?;

                    // read i32
                    let content = protocol.read_i32()?;
                    output.user_id = content;
                }
                Some(2) => {
                    ttype_comparing(ident.field_type, TType::String)?;

                    // read string
                    let content = protocol.read_string()?;
                    output.user_name = content;
                }
                Some(3) if ident.field_type == TType::Bool => {
                    ttype_comparing(ident.field_type, TType::Bool)?;

                    // read bool
                    let content = protocol.read_bool()?;
                    output.is_male = content;
                }
                Some(10) if ident.field_type == TType::Map => {
                    ttype_comparing(ident.field_type, TType::Map)?;

                    // read optional map
                    let map_ident = protocol.read_map_begin()?;
                    let mut map = HashMap::new();
                    for _ in 0..map_ident.size {
                        let key = protocol.read_string()?;
                        let value = protocol.read_string()?;
                        map.insert(key, value);
                    }
                    output.extra = Some(map);
                    protocol.read_map_end()?;
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

/// Generated(manually) impl Message for GetUserRequest
impl Message for GetUserRequest {
    fn encode<T: TOutputProtocol>(&self, _cx: &MsgContext, protocol: &mut T) -> Result<()> {
        protocol.write_struct_begin(&TStructIdentifier {
            name: "GetUserRequest".to_string(),
        })?;

        // user_id
        protocol.write_field_begin(&TFieldIdentifier {
            name: Some("user_id".to_string()),
            field_type: TType::I32,
            id: Some(1),
        })?;
        protocol.write_i32(self.user_id)?;
        protocol.write_field_end()?;

        // user_name
        protocol.write_field_begin(&TFieldIdentifier {
            name: Some("user_name".to_string()),
            field_type: TType::String,
            id: Some(2),
        })?;
        protocol.write_string(&self.user_name)?;
        protocol.write_field_end()?;

        // is_male
        protocol.write_field_begin(&TFieldIdentifier {
            name: Some("is_male".to_string()),
            field_type: TType::Bool,
            id: Some(3),
        })?;
        protocol.write_bool(self.is_male)?;
        protocol.write_field_end()?;

        protocol.write_field_stop()?;
        protocol.write_struct_end()?;
        Ok(())
    }

    fn decode<T: TInputProtocol>(_cx: &mut MsgContext, protocol: &mut T) -> Result<Self> {
        let mut output = Self::default();
        protocol.read_struct_begin()?;

        loop {
            let ident = protocol.read_field_begin()?;
            if ident.field_type == TType::Stop {
                break;
            }
            match ident.id {
                Some(1) => {
                    ttype_comparing(ident.field_type, TType::I32)?;

                    // read i32
                    let content = protocol.read_i32()?;
                    output.user_id = content;
                }
                Some(2) => {
                    ttype_comparing(ident.field_type, TType::String)?;

                    // read string
                    let content = protocol.read_string()?;
                    output.user_name = content;
                }
                Some(3) => {
                    ttype_comparing(ident.field_type, TType::Bool)?;

                    // read bool
                    let content = protocol.read_bool()?;
                    output.is_male = content;
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

/// Generated(manually) impl Message for GetUserResponse
impl Message for GetUserResponse {
    fn encode<T: TOutputProtocol>(&self, cx: &MsgContext, protocol: &mut T) -> Result<()> {
        protocol.write_struct_begin(&TStructIdentifier {
            name: "GetUserResponse".to_string(),
        })?;

        // users
        protocol.write_field_begin(&TFieldIdentifier {
            name: Some("users".to_string()),
            field_type: TType::List,
            id: Some(1),
        })?;

        protocol.write_list_begin(&TListIdentifier {
            element_type: TType::Struct,
            size: self.users.len() as i32,
        })?;
        for user in self.users.iter() {
            user.encode(cx, protocol)?;
        }
        protocol.write_list_end()?;

        protocol.write_field_end()?;

        protocol.write_field_stop()?;
        protocol.write_struct_end()?;
        Ok(())
    }

    fn decode<T: TInputProtocol>(cx: &mut MsgContext, protocol: &mut T) -> Result<Self> {
        let mut output = Self::default();
        protocol.read_struct_begin()?;

        loop {
            let ident = protocol.read_field_begin()?;
            if ident.field_type == TType::Stop {
                break;
            }
            match ident.id {
                Some(1) => {
                    ttype_comparing(ident.field_type, TType::List)?;

                    // read list
                    let list_ident = protocol.read_list_begin()?;
                    output.users = Vec::with_capacity(list_ident.size as usize);
                    for _ in 0..list_ident.size {
                        let user = User::decode(cx, protocol)?;
                        output.users.push(user);
                    }
                    protocol.read_list_end()?;
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

#[inline(always)]
fn ttype_comparing(x: TType, y: TType) -> crate::Result<()> {
    if x != y {
        return Err(new_protocol_error(
            ProtocolErrorKind::InvalidData,
            format!("invalid ttype: {}, expect: {}", x, y),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::binary::{TBinaryInputProtocol, TBinaryOutputProtocol};
    use crate::message::Message;

    use super::User;

    #[test]
    fn test_user_encode_decode() {
        let mut extra = HashMap::new();
        extra.insert("test_key1".to_string(), "test_val1".to_string());
        extra.insert("test_key2".to_string(), "test_val2".to_string());
        let user = User {
            user_id: 7,
            user_name: "ihciah".to_string(),
            is_male: true,
            extra: Some(extra),
        };

        let mut buffer = bytes::BytesMut::new();

        // Encode
        let mut output_protocol = TBinaryOutputProtocol::new(&mut buffer, true);
        let context = Default::default();
        user.encode(&context, &mut output_protocol).unwrap();

        // Decode
        let mut input_protocol = TBinaryInputProtocol::new(buffer.as_ref(), true);
        let mut context = Default::default();
        let user_decoded = User::decode(&mut context, &mut input_protocol).unwrap();

        assert_eq!(user, user_decoded);
    }
}
