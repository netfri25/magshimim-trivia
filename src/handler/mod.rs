pub mod login;
pub use login::LoginRequestHandler;

use crate::messages::{Request, RequestResult};

pub trait Handler: Send {
    fn relevant(&self, request: &Request) -> bool;
    fn handle(&mut self, request: Request) -> std::io::Result<RequestResult>;
}
