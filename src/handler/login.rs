use crate::messages::{Request, RequestKind, RequestResult};

use super::Handler;

pub struct LoginRequestHandler;

impl Handler for LoginRequestHandler {
    fn relevant(&self, request: &Request) -> bool {
        request.kind.is_login()
    }

    fn handle(&mut self, request: Request) -> std::io::Result<RequestResult> {
        let RequestKind::Login { username, password } = request.kind else {
            return Ok(RequestResult::new_error("Invalid request"));
        };

        eprintln!("time since got request: {:?}", request.time.elapsed());
        eprintln!("username: {username}");
        eprintln!("password: {password}");

        Ok(RequestResult::new_error("not yet implemented"))
    }
}
