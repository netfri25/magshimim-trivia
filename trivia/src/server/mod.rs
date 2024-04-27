use std::net::ToSocketAddrs;
use std::sync::{Arc, Mutex};

pub mod communicator;
use communicator::Communicator;

use crate::db::{self, Database};
use crate::handler::RequestHandlerFactory;

pub struct TriviaServer {
    comm: Communicator,
}

impl TriviaServer {
    pub fn build(addr: impl ToSocketAddrs, mut db: impl Database + 'static) -> Result<Self, Error> {
        db.open()?;
        let db = Arc::new(Mutex::new(db));
        let factory = Arc::new(RequestHandlerFactory::new(db.clone()));
        let comm = Communicator::build(addr, factory.clone())?;
        Ok(Self { comm })
    }

    pub fn run(self) {
        std::thread::spawn(move || self.comm.start_handle_requests());

        let mut line = String::new();
        loop {
            line.clear();
            if let Err(err) = std::io::stdin().read_line(&mut line) {
                eprintln!("[ERROR] stdin error: {}", err);
                continue;
            }

            let cmd = line.trim().to_lowercase();

            match cmd.as_str() {
                "exit" => break,
                _ => eprintln!("Unknown command: {:?}", cmd),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::net::TcpStream;
    use std::sync::OnceLock;

    use crate::db::SqliteDatabase;
    use crate::messages::{Request, Response};

    use super::*;

    const ADDR: &str = "127.0.0.1:6969";

    static START_SERVER: OnceLock<()> = OnceLock::new();

    fn start_server() {
        START_SERVER.get_or_init(|| {
            let Ok(db) = SqliteDatabase::connect(":memory:") else {
                return;
            };

            let Ok(server) = TriviaServer::build(ADDR, db) else {
                return;
            };

            std::thread::spawn(move || server.run());
        });
    }

    #[test]
    fn login_without_signup() {
        start_server();

        let mut client = TcpStream::connect(ADDR).unwrap();
        let username = "user1234".to_string();
        let password = "pass1234".to_string();
        let request = Request::Login { username, password };

        request.write_to(&mut client).unwrap();
        let response = Response::read_from(&mut client).unwrap();
        let expected = Response::new_error("user doesn't exist");

        assert_eq!(response, expected);
    }

    #[test]
    fn signup_login() {
        start_server();

        let mut client = TcpStream::connect(ADDR).unwrap();

        let request = Request::Signup {
            username: "signup1234".to_string(),
            password: "pass1234".to_string(),
            email: "email@example.com".to_string(),
        };
        request.write_to(&mut client).unwrap();
        let response = Response::read_from(&mut client).unwrap();
        let expected = Response::Signup;
        assert_eq!(response, expected);

        let request = Request::Login {
            username: "signup1234".to_string(),
            password: "pass1234".to_string(),
        };
        request.write_to(&mut client).unwrap();
        let response = Response::read_from(&mut client).unwrap();
        let expected = Response::Login;
        assert_eq!(response, expected);
    }

    #[test]
    fn signup_signup() {
        start_server();

        let mut client = TcpStream::connect(ADDR).unwrap();

        let request = Request::Signup {
            username: "double".to_string(),
            password: "pass1234".to_string(),
            email: "email@example.com".to_string(),
        };
        request.write_to(&mut client).unwrap();
        let response = Response::read_from(&mut client).unwrap();
        let expected = Response::Signup;
        assert_eq!(response, expected);

        let request = Request::Signup {
            username: "double".to_string(),
            password: "pass1234".to_string(),
            email: "email@example.com".to_string(),
        };
        request.write_to(&mut client).unwrap();
        let response = Response::read_from(&mut client).unwrap();
        let expected = Response::Error("username already exists".into());
        assert_eq!(response, expected);
    }

    #[test]
    fn signup_login_logut_login() {
        start_server();

        let mut client = TcpStream::connect(ADDR).unwrap();

        let request = Request::Signup {
            username: "multiple-login".to_string(),
            password: "pass1234".to_string(),
            email: "email@example.com".to_string(),
        };
        request.write_to(&mut client).unwrap();
        let response = Response::read_from(&mut client).unwrap();
        let expected = Response::Signup;
        assert_eq!(response, expected);

        let request = Request::Login {
            username: "multiple-login".to_string(),
            password: "pass1234".to_string(),
        };
        request.write_to(&mut client).unwrap();
        let response = Response::read_from(&mut client).unwrap();
        let expected = Response::Login;
        assert_eq!(response, expected);

        // disconnect from the server
        drop(client);

        let mut client = TcpStream::connect(ADDR).unwrap();

        let request = Request::Login {
            username: "multiple-login".to_string(),
            password: "pass1234".to_string(),
        };
        request.write_to(&mut client).unwrap();
        let response = Response::read_from(&mut client).unwrap();
        let expected = Response::Login;
        assert_eq!(response, expected);
    }

    #[test]
    fn signup_login_login() {
        start_server();

        let mut client = TcpStream::connect(ADDR).unwrap();

        let request = Request::Signup {
            username: "login-abuser".to_string(),
            password: "pass1234".to_string(),
            email: "email@example.com".to_string(),
        };
        request.write_to(&mut client).unwrap();
        let response = Response::read_from(&mut client).unwrap();
        let expected = Response::Signup;
        assert_eq!(response, expected);

        let request = Request::Login {
            username: "login-abuser".to_string(),
            password: "pass1234".to_string(),
        };
        request.write_to(&mut client).unwrap();
        let response = Response::read_from(&mut client).unwrap();
        let expected = Response::Login;
        assert_eq!(response, expected);

        let mut client = TcpStream::connect(ADDR).unwrap();

        let request = Request::Login {
            username: "login-abuser".to_string(),
            password: "pass1234".to_string(),
        };
        request.write_to(&mut client).unwrap();
        let response = Response::read_from(&mut client).unwrap();
        let expected = Response::Error("user already connected".to_string());
        assert_eq!(response, expected);
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Communicator(#[from] communicator::Error),

    #[error(transparent)]
    DBError(#[from] db::Error),
}
