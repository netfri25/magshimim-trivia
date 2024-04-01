pub mod login;
pub use login::LoginRequestHandler;

use crate::messages::{RequestResult, RequestInfo};

pub trait Handler: Send {
    fn relevant(&self, request_info: &RequestInfo) -> bool;
    fn handle(&mut self, request_info: RequestInfo) -> std::io::Result<RequestResult>;
}
