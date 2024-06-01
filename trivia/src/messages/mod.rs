use std::io;

pub mod request;
pub use request::*; // re-export

pub mod response;
pub use response::*; // re-export

pub mod phone_number;
pub use phone_number::PhoneNumber; // re-export

pub mod address;
pub use address::Address; // re-export

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] io::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),
}
