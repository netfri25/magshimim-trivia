pub mod login;
pub use login::LoginRequestHandler;

pub mod menu;
pub use menu::MenuRequestHandler;

pub mod factory;
pub use factory::RequestHandlerFactory;

pub mod room_admin;
pub use room_admin::RoomAdminRequestHandler;

pub mod room_member;
pub use room_member::RoomMemberRequestHandler;

pub mod game;
pub use game::GameRequestHandler;

use crate::messages::{RequestResult, RequestInfo};
use crate::db;

pub trait Handler: Send {
    fn relevant(&self, request_info: &RequestInfo) -> bool;
    fn handle(&mut self, request_info: RequestInfo) -> Result<RequestResult, Error>;
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    DBError(#[from] db::Error),

    #[error(transparent)]
    SystemTime(#[from] std::time::SystemTimeError),
}
