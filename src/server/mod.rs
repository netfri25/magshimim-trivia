use std::sync::{Arc, Mutex};
use std::net::ToSocketAddrs;

mod communicator;
use communicator::Communicator;

use crate::db::Database;
use crate::handler::RequestHandlerFactory;

pub struct Server {
    comm: Communicator,
    db: Arc<Mutex<dyn Database>>,
    factory: Arc<RequestHandlerFactory>,
}

impl Server {
    pub fn build(addr: impl ToSocketAddrs, mut db: impl Database + 'static) -> anyhow::Result<Self> {
        db.open()?;
        let db = Arc::new(Mutex::new(db));
        let factory = Arc::new(RequestHandlerFactory::new(db.clone()));
        let comm = Communicator::build(addr, factory.clone())?;
        Ok(Self { comm, db, factory })
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
    use std::sync::OnceLock;
    use std::net::TcpStream;

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

            let Ok(server) = Server::build(ADDR, db) else {
                return;
            };

            std::thread::spawn(move || server.run());
        });
    }

    #[test]
    fn try_login() {
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
    fn signup_and_login() {
        start_server();

        let mut client = TcpStream::connect(ADDR).unwrap();

        let request = Request::Signup {
            username: "signup1234".to_string(),
            password: "pass1234".to_string(),
            email: "email@example.com".to_string(),
        };
        request.write_to(&mut client).unwrap();
        let response = Response::read_from(&mut client).unwrap();
        let expected = Response::Signup { status: 1 };
        assert_eq!(response, expected);

        let request = Request::Login {
            username: "signup1234".to_string(),
            password: "pass1234".to_string(),
        };
        request.write_to(&mut client).unwrap();
        let response = Response::read_from(&mut client).unwrap();
        let expected = Response::Login { status: 1 };
        assert_eq!(response, expected);
    }

    #[test]
    fn double_signup() {
        start_server();

        let mut client = TcpStream::connect(ADDR).unwrap();

        let request = Request::Signup {
            username: "double".to_string(),
            password: "pass1234".to_string(),
            email: "email@example.com".to_string(),
        };
        request.write_to(&mut client).unwrap();
        let response = Response::read_from(&mut client).unwrap();
        let expected = Response::Signup { status: 1 };
        assert_eq!(response, expected);

        let request = Request::Signup {
            username: "double".to_string(),
            password: "pass1234".to_string(),
            email: "email@example.com".to_string(),
        };
        request.write_to(&mut client).unwrap();
        let response = Response::read_from(&mut client).unwrap();
        let expected = Response::Error { msg: "username already exists".into() };
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
        let expected = Response::Signup { status: 1 };
        assert_eq!(response, expected);

        let request = Request::Login {
            username: "multiple-login".to_string(),
            password: "pass1234".to_string(),
        };
        request.write_to(&mut client).unwrap();
        let response = Response::read_from(&mut client).unwrap();
        let expected = Response::Login { status: 1 };
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
        let expected = Response::Login { status: 1 };
        assert_eq!(response, expected);
    }

    #[test]
    fn signup_double_login() {
        start_server();

        let mut client = TcpStream::connect(ADDR).unwrap();

        let request = Request::Signup {
            username: "login-abuser".to_string(),
            password: "pass1234".to_string(),
            email: "email@example.com".to_string(),
        };
        request.write_to(&mut client).unwrap();
        let response = Response::read_from(&mut client).unwrap();
        let expected = Response::Signup { status: 1 };
        assert_eq!(response, expected);

        let request = Request::Login {
            username: "login-abuser".to_string(),
            password: "pass1234".to_string(),
        };
        request.write_to(&mut client).unwrap();
        let response = Response::read_from(&mut client).unwrap();
        let expected = Response::Login { status: 1 };
        assert_eq!(response, expected);

        let mut client = TcpStream::connect(ADDR).unwrap();

        let request = Request::Login {
            username: "login-abuser".to_string(),
            password: "pass1234".to_string(),
        };
        request.write_to(&mut client).unwrap();
        let response = Response::read_from(&mut client).unwrap();
        let expected = Response::Error { msg: "user already connected".to_string() };
        assert_eq!(response, expected);
    }
}
