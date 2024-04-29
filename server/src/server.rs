use std::net::ToSocketAddrs;
use std::sync::Mutex;

use crate::communicator::{self, Communicator};

use trivia::db::{self, Database};
use trivia::handler::RequestHandlerFactory;

pub struct Server<'db> {
    comm: Communicator<'db>,
}

impl<'db, 'me: 'db> Server<'db> {
    pub fn build(
        addr: impl ToSocketAddrs,
        db: &'db Mutex<impl Database + 'static>,
    ) -> Result<Self, Error> {
        let factory = RequestHandlerFactory::new(db);
        let comm = Communicator::build(addr, factory)?;
        Ok(Self { comm })
    }

    pub fn run(&'me self) {
        std::thread::scope(|scope| {
            scope.spawn(|| self.comm.start_handle_requests());

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
        });
    }
}

#[cfg(test)]
mod tests {
    use std::net::TcpStream;
    use std::sync::{Arc, Condvar, OnceLock};

    use trivia::db::SqliteDatabase;
    use trivia::messages::{Request, Response};

    use super::*;

    const ADDR: &str = "127.0.0.1:1234";

    static START_SERVER: OnceLock<()> = OnceLock::new();

    fn start_server() {
        START_SERVER.get_or_init(|| {
            let cond = Arc::new(Condvar::new());
            let cond2 = cond.clone();

            std::thread::spawn(move || {
                let Ok(mut db) = SqliteDatabase::connect(":memory:") else {
                    return;
                };

                db.open().unwrap();

                let db = Mutex::new(db);

                let Ok(server) = Server::build(ADDR, &db) else {
                    return;
                };

                // notify that the server has started running
                cond2.notify_one();
                server.run();
            });

            let dummy = Mutex::new(());
            drop(cond.wait(dummy.lock().unwrap()));
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
