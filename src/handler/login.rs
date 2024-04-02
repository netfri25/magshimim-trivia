use crate::messages::{Request, RequestResult, RequestInfo};

use super::Handler;

pub struct LoginRequestHandler;

impl Handler for LoginRequestHandler {
    fn relevant(&self, request_info: &RequestInfo) -> bool {
        request_info.data.is_login()
    }

    fn handle(&mut self, request: RequestInfo) -> anyhow::Result<RequestResult> {
        let Request::Login { username, password } = request.data else {
            return Ok(RequestResult::new_error("Invalid request"));
        };

        eprintln!("time since got request: {:?}", request.time.elapsed());
        eprintln!("username: {username}");
        eprintln!("password: {password}");

        Ok(RequestResult::new_error("not yet implemented"))
    }
}
