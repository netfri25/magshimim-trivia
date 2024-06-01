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

// some handlers hold a reference to the db
// for that reason I need to "save" the db lifetime in the trait and the RequestResult
// handlers may also hold the RequestHandlerFactory, and therefore they also need a 'factory
// lifetime, but because the factory lives as long as db ('factory: 'db) therefore I don't need to
// hold a lifetime to it, because if it can live long enough to hold a reference to the db then it
// surely lives long enough to hold a reference to the factory
pub trait Handler<'db> {
    fn relevant(&self, request_info: &RequestInfo) -> bool;
    fn handle(&mut self, request_info: RequestInfo) -> Result<RequestResult<'db>, Error>;
}
