use std::collections::HashMap;
use std::net::{SocketAddr, TcpListener, TcpStream, ToSocketAddrs};
use std::sync::{Arc, Mutex};

use crate::handler::{Handler, LoginRequestHandler};
use crate::messages::{Request, RequestResult, RequestInfo};

type Clients = HashMap<SocketAddr, Box<dyn Handler>>;

pub struct Communicator {
    socket: TcpListener,
    clients: Arc<Mutex<Clients>>,
}

impl Communicator {
    pub fn build(addr: impl ToSocketAddrs) -> anyhow::Result<Self> {
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
        clients: Arc<Mutex<Clients>>,
    ) -> anyhow::Result<()> {
        let addr = client.peer_addr()?;
        loop {
            // using little-endian for the data length
            let request = Request::read_from(&mut client)?;
            let request_info = RequestInfo::new_now(request);

            let RequestResult { response, new_handler } = {
                let mut clients_mx = clients.lock().unwrap();
                let handler = clients_mx.get_mut(&addr).expect("client must have an handler");
                handler.handle(request_info)?
            };

            response.write_to(&mut client)?;

            if let Some(handler) = new_handler {
                let mut clients_mx = clients.lock().unwrap();
                clients_mx.insert(addr, handler);
            }
        }
    }
}
