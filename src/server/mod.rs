use std::net::ToSocketAddrs;
use std::io::Write;

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
        std::thread::spawn(move || {
            self.comm.start_handle_requests()
        });

        let mut line = String::new();
        loop {
            print!("> ");
            std::io::stdout().flush().ok();
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
