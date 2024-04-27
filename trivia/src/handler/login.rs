use std::sync::Arc;

use crate::{
    managers::login::LoggedUser,
    messages::{Request, RequestInfo, RequestResult, Response, StatusCode},
};

use super::{Error, Handler, RequestHandlerFactory};

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
        use Request::*;
        matches!(request_info.data, Login { .. } | Signup { .. })
    }

    fn handle(&mut self, request: RequestInfo) -> Result<RequestResult, Error> {
        let login_manager = self.factory.get_login_manager();

        let result = match request.data {
            Request::Login { username, password } => {
                if let Some(err) = login_manager
                    .lock()
                    .unwrap()
                    .login(username.clone(), &password)?
                {
                    return Ok(RequestResult::new_error(err));
                }

                let logged_user = LoggedUser::new(username);
                let response = Response::Login {
                    status: StatusCode::ResponseOk,
                };
                RequestResult::new(
                    response,
                    self.factory.create_menu_request_handler(logged_user),
                )
            }

            Request::Signup {
                username,
                password,
                email,
            } => {
                if let Some(err) = login_manager
                    .lock()
                    .unwrap()
                    .signup(username, &password, &email)?
                {
                    return Ok(RequestResult::new_error(err));
                }

                let response = Response::Signup {
                    status: StatusCode::ResponseOk,
                };
                RequestResult::without_handler(response) // no need to switch an handler
            }

            _ => RequestResult::new_error("Invalid request"),
        };

        Ok(result)
    }
}
