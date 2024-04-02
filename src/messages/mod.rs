use serde_repr::{Serialize_repr, Deserialize_repr};

pub mod request;
pub use request::*; // re-export

pub mod response;
pub use response::*; // re-export

#[derive(Debug, Serialize_repr, Deserialize_repr, PartialEq, Eq)]
#[repr(u64)]
pub enum StatusCode {
    ResponseOk = 0,
}
