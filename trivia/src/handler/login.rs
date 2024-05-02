use crate::managers::login::LoggedUser;
use crate::messages::{Request, RequestInfo, RequestResult, Response};

use super::{Error, Handler, RequestHandlerFactory};

pub struct LoginRequestHandler<'db, 'factory> {
    factory: &'factory RequestHandlerFactory<'db>,
}

impl<'db, 'factory> LoginRequestHandler<'db, 'factory> {
    pub fn new(factory: &'factory RequestHandlerFactory<'db>) -> Self {
        Self { factory }
    }
}

impl<'db, 'factory: 'db> Handler<'db> for LoginRequestHandler<'db, 'factory> {
    fn relevant(&self, request_info: &RequestInfo) -> bool {
        use Request::*;
        matches!(request_info.data, Login { .. } | Signup { .. })
    }

    fn handle(&mut self, request: RequestInfo) -> Result<RequestResult<'db>, Error> {
        let login_manager = self.factory.get_login_manager();

        let result = match request.data {
            Request::Login { username, password } => {
                if let Some(err) = login_manager
                    .write()
                    .unwrap()
                    .login(username.clone(), &password)?
                {
                    return Ok(RequestResult::new_error(err));
                }

                let logged_user = LoggedUser::new(username);
                let response = Response::Login;
                RequestResult::new(
                    response,
                    self.factory.create_menu_request_handler(logged_user),
                )
            }

            Request::Signup {
                username,
                password,
                email,
                phone,
                address,
                birth_date,
            } => {
                if let Some(err) = login_manager
                    .write()
                    .unwrap()
                    .signup(username, &password, &email, phone, address, birth_date)?
                {
                    return Ok(RequestResult::new_error(err));
                }

                let response = Response::Signup;
                RequestResult::without_handler(response) // no need to switch an handler
            }

            _ => RequestResult::new_error("Invalid request"),
        };

        Ok(result)
    }
}
