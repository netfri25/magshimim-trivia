use std::sync::Arc;

use crate::messages::{Request, RequestInfo, RequestResult, Response};

use super::{Handler, MenuRequestHandler, RequestHandlerFactory};

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
        request_info.data.is_login() || request_info.data.is_signup()
    }

    fn handle(&mut self, request: RequestInfo) -> anyhow::Result<RequestResult> {
        let login_manager = self.factory.get_login_manager();

        let result = match request.data {
            Request::Login { username, password } => {
                if let Some(err) = login_manager.lock().unwrap().login(username, &password)? {
                    return Ok(RequestResult::new_error(err));
                }

                // TODO: proper status codes
                let response = Response::Login { status: 1 };
                RequestResult::new(response, MenuRequestHandler)
            },

            Request::Signup { username, password, email } => {
                if let Some(err) = login_manager.lock().unwrap().signup(username, &password, &email)? {
                    return Ok(RequestResult::new_error(err));
                }

                let response = Response::Signup { status: 1 };
                RequestResult::without_handler(response) // no need to switch an handler
            },

            _ => RequestResult::new_error("Invalid request"),
        };

        Ok(result)
    }
}
