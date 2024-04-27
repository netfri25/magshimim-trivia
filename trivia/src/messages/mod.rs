use std::io;

pub mod request;
pub use request::*; // re-export

pub mod response;
pub use response::*; // re-export

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] io::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),
}
