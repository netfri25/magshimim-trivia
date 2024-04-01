use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream, ToSocketAddrs};
use std::sync::{Arc, Mutex};

use crate::handler::{Handler, LoginRequestHandler};

type Clients = HashMap<SocketAddr, Box<dyn Handler>>;

pub struct Communicator {
    socket: TcpListener,
    clients: Arc<Mutex<Clients>>,
}

impl Communicator {
    pub fn build(addr: impl ToSocketAddrs) -> std::io::Result<Self> {
        let socket = TcpListener::bind(addr)?;
        let clients = Default::default();
        Ok(Self { socket, clients })
    }

    pub fn start_handle_requests(&mut self) {
        self.listen()
    }

    fn listen(&mut self) {
        for client in self.socket.incoming() {
            let Ok(client) = client else {
                eprintln!("[ERROR] connection error: {:?}", client);
                continue;
            };

            eprintln!("[LOG] connected {:?}", client);

            let handler = Box::new(LoginRequestHandler);
            let addr = client.peer_addr().unwrap();
            self.clients.lock().unwrap().insert(addr, handler);
            let clients = self.clients.clone();
            std::thread::spawn(move || {
                if let Err(err) = Self::handle_new_client(client, clients) {
                    eprintln!("[ERROR] communication error: {err}");
                }
            });
        }
    }

    fn handle_new_client(
        mut client: TcpStream,
        _clients: Arc<Mutex<Clients>>,
    ) -> std::io::Result<()> {
        let mut buf = [0; 5];
        client.read_exact(&mut buf)?;
        let text = String::from_utf8_lossy(&buf);
        eprintln!("[LOG] from client: {}", text);
        write!(&mut client, "{}", text)?;
        Ok(())
    }
}
