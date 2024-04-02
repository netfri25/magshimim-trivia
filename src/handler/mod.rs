pub mod login;
pub use login::LoginRequestHandler;

pub mod menu;
pub use menu::MenuRequestHandler;

pub mod factory;
pub use factory::RequestHandlerFactory;

use crate::messages::{RequestResult, RequestInfo};

pub trait Handler: Send {
    fn relevant(&self, request_info: &RequestInfo) -> bool;
    fn handle(&mut self, request_info: RequestInfo) -> anyhow::Result<RequestResult>;
}
