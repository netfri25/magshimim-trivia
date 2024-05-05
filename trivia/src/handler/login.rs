use crate::db::Database;
use crate::email::{self, Email};
use crate::messages::phone_number::PhoneNumber;
use crate::messages::{phone_number, Address, Request, RequestInfo, RequestResult, Response};
use crate::password::{self, Password};
use crate::username::{self, Username};

use super::{Error, Handler, RequestHandlerFactory};

pub struct LoginRequestHandler<'db, 'factory, DB: ?Sized> {
    factory: &'factory RequestHandlerFactory<'db, DB>,
}

impl<'db, 'factory: 'db, DB> Handler<'db> for LoginRequestHandler<'db, 'factory, DB>
where
    DB: Database + Sync + ?Sized,
{
    fn relevant(&self, request_info: &RequestInfo) -> bool {
        use Request::*;
        matches!(request_info.data, Login { .. } | Signup { .. })
    }

    fn handle(&mut self, request: RequestInfo) -> Result<RequestResult<'db>, Error> {
        match request.data {
            Request::Login { username, password } => {
                let (username, password) = match parse_login(&username, &password) {
                    Ok(tup) => tup,
                    Err(err) => return Ok(RequestResult::new_error(err)),
                };

                self.login(username, password)
            }

            Request::Signup {
                username,
                password,
                email,
                phone,
                address,
                birth_date,
            } => {
                let (username, password, email, phone) =
                    match parse_signup(&username, &password, &email, &phone) {
                        Ok(tup) => tup,
                        Err(err) => return Ok(RequestResult::new_error(err)),
                    };

                self.signup(username, password, email, phone, address, birth_date)
            }

            _ => Ok(RequestResult::new_error("Invalid request")),
        }
    }
}

impl<'db, 'factory: 'db, DB> LoginRequestHandler<'db, 'factory, DB>
where
    DB: Database + Sync + ?Sized,
{
    pub fn new(factory: &'factory RequestHandlerFactory<'db, DB>) -> Self {
        Self { factory }
    }

    fn login(&self, username: Username, password: Password) -> Result<RequestResult<'db>, Error> {
        let login_manager = self.factory.get_login_manager();
        if let Some(err) = login_manager
            .write()
            .unwrap()
            .login(username.clone(), password)?
        {
            Ok(RequestResult::new_error(err))
        } else {
            let response = Response::Login;
            Ok(RequestResult::new(
                response,
                self.factory.create_menu_request_handler(username),
            ))
        }
    }

    fn signup(
        &self,
        username: Username,
        password: Password,
        email: Email,
        phone: PhoneNumber,
        address: Address,
        birth_date: chrono::NaiveDate,
    ) -> Result<RequestResult<'db>, Error> {
        let login_manager = self.factory.get_login_manager();
        if let Some(err) = login_manager
            .write()
            .unwrap()
            .signup(username, password, email, phone, address, birth_date)?
        {
            Ok(RequestResult::new_error(err))
        } else {
            let response = Response::Signup;
            Ok(RequestResult::without_handler(response)) // no need to switch an handler
        }
    }
}

fn parse_login(username: &str, password: &str) -> Result<(Username, Password), ParseError> {
    let username = username.parse()?;
    let password = password.parse()?;
    Ok((username, password))
}

fn parse_signup(
    username: &str,
    password: &str,
    email: &str,
    phone: &str,
) -> Result<(Username, Password, Email, PhoneNumber), ParseError> {
    let (username, password) = parse_login(username, password)?;
    let email = email.parse()?;
    let phone = phone.parse()?;
    Ok((username, password, email, phone))
}

#[derive(Debug, thiserror::Error)]
enum ParseError {
    #[error("invalid username: {0}")]
    Username(#[from] username::Error),

    #[error("invalid password: {0}")]
    Password(#[from] password::Error),

    #[error("invalid email: {0}")]
    Email(#[from] email::Error),

    #[error("invalid phone number: {0}")]
    PhoneNumber(#[from] phone_number::Error),
}
