use std::io::Write;
use std::net::ToSocketAddrs;

mod communicator;
use communicator::Communicator;

pub struct Server {
    comm: Communicator,
}

impl Server {
    pub fn build(addr: impl ToSocketAddrs) -> std::io::Result<Self> {
        let comm = Communicator::build(addr)?;
        Ok(Self { comm })
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
    fn hello_hello() {
        use super::Server;
        use std::io::{Read, Write};
        use std::net::TcpStream;

        const ADDR: &str = "127.0.0.1:6969";

        let server = Server::build(ADDR).unwrap();
        std::thread::spawn(move || server.run());

        let mut client = TcpStream::connect(ADDR).unwrap();
        write!(&mut client, "Hello").unwrap();

        let mut buf = [0; 5];
        client.read_exact(&mut buf).unwrap();
        let text = String::from_utf8_lossy(&buf);
        assert_eq!(text, "Hello");
    }
}
