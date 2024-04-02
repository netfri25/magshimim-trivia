use std::sync::{Arc, Mutex};
use std::net::ToSocketAddrs;

mod communicator;
use communicator::Communicator;

use crate::db::Database;
use crate::handler::RequestHandlerFactory;

pub struct Server {
    comm: Communicator,
    db: Arc<Mutex<dyn Database>>,
    factory: RequestHandlerFactory,
}

impl Server {
    pub fn build(addr: impl ToSocketAddrs, db: impl Database + 'static) -> anyhow::Result<Self> {
        let comm = Communicator::build(addr)?;
        let db = Arc::new(Mutex::new(db));
        let factory = RequestHandlerFactory::new(db.clone());
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

mod tests {

    #[test]
    fn try_login() {
        use crate::messages::{Request, Response};
        use crate::server::Server;
        use crate::db::SqliteDatabase;
        use std::net::TcpStream;

        const ADDR: &str = "127.0.0.1:6969";

        let server = Server::build(ADDR, SqliteDatabase::connect(":memory:").unwrap()).unwrap();
        std::thread::spawn(move || server.run());

        let mut client = TcpStream::connect(ADDR).unwrap();
        let username = "user1234".to_string();
        let password = "pass1234".to_string();
        let request = Request::Login { username, password };

        request.write_to(&mut client).unwrap();
        let response = Response::read_from(&mut client).unwrap();
        let expected = Response::new_error("not yet implemented");

        assert_eq!(response, expected);
    }
}
