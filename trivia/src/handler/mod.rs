pub mod login;
pub use login::LoginRequestHandler;

pub mod menu;
pub use menu::MenuRequestHandler;

pub mod factory;
pub use factory::RequestHandlerFactory;

pub mod room_user;
pub use room_user::RoomUserRequestHandler;

pub mod game;
pub use game::GameRequestHandler;

use crate::db;
use crate::messages::{RequestInfo, RequestResult};

pub use db::Error;

pub trait Handler<'db>: Send {
    fn relevant(&self, request_info: &RequestInfo) -> bool;
    fn handle(&mut self, request_info: RequestInfo) -> Result<RequestResult<'db>, Error>;
}
