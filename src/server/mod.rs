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

    pub fn run(mut self) {
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
    const ADDR: &str = "127.0.0.1:6969";

    fn start_server() {
        use crate::server::Server;
        use crate::db::SqliteDatabase;

        let Ok(db) = SqliteDatabase::connect(":memory:") else {
            return;
        };

        let Ok(server) = Server::build(ADDR, db) else {
            return;
        };

        std::thread::spawn(move || server.run());
    }

    #[test]
    fn try_login() {
        use crate::messages::{Request, Response};
        use std::net::TcpStream;

        const ADDR: &str = "127.0.0.1:6969";

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
        use crate::messages::{Request, Response};
        use std::net::TcpStream;

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
}
