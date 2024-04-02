use std::cell::Cell;
use std::collections::HashMap;
use std::net::{SocketAddr, TcpListener, TcpStream, ToSocketAddrs};
use std::sync::{Arc, Mutex};

use crate::defer::Defer;
use crate::handler::{Handler, LoginRequestHandler, RequestHandlerFactory};
use crate::messages::{Request, RequestResult, RequestInfo};

type Clients = HashMap<SocketAddr, Box<dyn Handler>>;

pub struct Communicator {
    socket: TcpListener,
    clients: Mutex<Clients>,
    factory: Arc<RequestHandlerFactory>,
}

impl Communicator {
    pub fn build(addr: impl ToSocketAddrs, factory: Arc<RequestHandlerFactory>) -> anyhow::Result<Self> {
        let socket = TcpListener::bind(addr)?;
        let clients = Default::default();
        Ok(Self { socket, clients, factory })
    }

    pub fn start_handle_requests(self) {
        Arc::new(self).listen()
    }

    fn listen(self: Arc<Self>) {
        for client in self.socket.incoming() {
            let Ok(client) = client else {
                eprintln!("[ERROR] connection error: {:?}", client);
                continue;
            };

            eprintln!("[LOG] connected {:?}", client);

            let handler = self.factory.create_login_request_handler();
            let addr = client.peer_addr().unwrap();
            self.clients.lock().unwrap().insert(addr, handler);
            let me = self.clone();
            std::thread::spawn(move || {
                if let Err(err) = me.clone().handle_new_client(client) {
                    eprintln!("[ERROR] communication error: {err}");
                }
            });
        }
    }

    // returns the username, if the user has connected
    fn handle_new_client(
        self: Arc<Self>,
        mut client: TcpStream,
    ) -> anyhow::Result<()> {
        let addr = client.peer_addr()?;
        let login_username: Cell<Option<String>> = Cell::new(None);

        let _defer = Defer(|| {
            if let Some(ref username) = login_username.take() {
                self.factory.get_login_manager().lock().unwrap().logut(username)
            }
        });

        loop {
            // using little-endian for the data length
            let request = Request::read_from(&mut client)?;

            // save the username, so it can be removed at the end of communication
            if let Request::Login { ref username, .. } = request {
                login_username.set(Some(String::from(username)));
            }

            let request_info = RequestInfo::new_now(request);

            let RequestResult { response, new_handler } = {
                let mut clients_mx = self.clients.lock().unwrap();
                let handler = clients_mx.get_mut(&addr).expect("client must have an handler");
                handler.handle(request_info)?
            };

            response.write_to(&mut client)?;

            if let Some(handler) = new_handler {
                let mut clients_mx = self.clients.lock().unwrap();
                clients_mx.insert(addr, handler);
            }
        }
    }
}
