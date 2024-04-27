use std::io;

use serde_repr::{Deserialize_repr, Serialize_repr};

pub mod request;
pub use request::*; // re-export

pub mod response;
pub use response::*; // re-export

#[derive(Debug, Serialize_repr, Deserialize_repr, PartialEq, Eq)]
#[repr(u64)]
pub enum StatusCode {
    ResponseOk = 0,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] io::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),
}
