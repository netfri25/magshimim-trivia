use std::sync::Arc;

use crate::messages::{Request, RequestResult, RequestInfo};

use super::{Handler, RequestHandlerFactory};

pub struct LoginRequestHandler {
    factory: Arc<RequestHandlerFactory>,
}

impl LoginRequestHandler {
    pub fn new(factory: Arc<RequestHandlerFactory>) -> Self {
        Self { factory }
    }
}

impl Handler for LoginRequestHandler {
    fn relevant(&self, request_info: &RequestInfo) -> bool {
        request_info.data.is_login()
    }

    fn handle(&mut self, request: RequestInfo) -> anyhow::Result<RequestResult> {
        let Request::Login { username, password } = request.data else {
            return Ok(RequestResult::new_error("Invalid request"));
        };

        let login_manager = self.factory.get_login_manager();
        if let Some(response) = login_manager.lock().unwrap().login(username, &password)? {
            return Ok(RequestResult::new_error(response));
        }

        todo!("move to the next manager")
    }
}
