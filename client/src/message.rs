use std::sync::Arc;
use derive_more::From;
use trivia::messages::Response;

use crate::connection;
use crate::page::{login, register};

#[derive(From, Debug, Clone)]
#[non_exhaustive]
pub enum Message {
    Connected,
    Error(Arc<connection::Error>),
    Response(Arc<Response>),

    #[from]
    Login(login::Msg),

    #[from]
    Register(register::Msg),
}
