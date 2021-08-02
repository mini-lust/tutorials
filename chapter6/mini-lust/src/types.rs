use crate::TType;

pub trait OrigType {
    fn orig_type() -> crate::protocol::TType {
        TType::Struct
    }
}
