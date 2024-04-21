use std::cell::Cell;
use std::collections::HashMap;
use std::io;
use std::net::{SocketAddr, TcpListener, TcpStream, ToSocketAddrs};
use std::sync::mpsc::{self, SyncSender};
use std::sync::{Arc, Mutex};

use crate::defer::Defer;
use crate::handler::{self, Handler, RequestHandlerFactory};
use crate::messages::{self, Request, RequestInfo, RequestResult};

type Clients = HashMap<SocketAddr, Box<dyn Handler>>;
pub type Channels = HashMap<String, SyncSender<RequestResult>>;

pub struct Communicator {
    socket: TcpListener,
    clients: Mutex<Clients>,
    channels: Arc<Mutex<Channels>>,
    factory: Arc<RequestHandlerFactory>,
}

impl Communicator {
    pub fn build(addr: impl ToSocketAddrs, factory: Arc<RequestHandlerFactory>) -> Result<Self, Error> {
        let socket = TcpListener::bind(addr)?;
        let clients = Default::default();
        let channels = factory.channels();
        Ok(Self { socket, clients, channels, factory })
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
    ) -> Result<(), Error> {
        let addr = client.peer_addr()?;
        let login_username: Cell<Option<String>> = Cell::new(None);

        let (sender, receiver) = mpsc::sync_channel(1);

        let _defer = Defer(|| {
            if let Some(ref username) = login_username.take() {
                eprintln!("[LOG] {:?} disconnected", username);
                self.factory.get_login_manager().lock().unwrap().logut(username);
                let req = Request::Logout;
                let mut clients_mx = self.clients.lock().unwrap();
                clients_mx.get_mut(&addr).map(|handler| handler.handle(RequestInfo::new_now(req)));
            }
        });

        loop {
            let result = if let Ok(result) = receiver.try_recv() {
                eprint!("[CHANNEL]: ");
                result
            } else {
                let request = Request::read_from(&mut client)?;
                eprintln!("[REQ]:     {:?}", request);
                eprint!("[RESP]:    ");

                // save the username, so it can be removed at the end of communication
                if let Request::Login { ref username, .. } = request {
                    if let Some(old_username) = login_username.replace(Some(String::from(username))) {
                        self.channels.lock().unwrap().remove(&old_username);
                    }

                    self.channels.lock().unwrap().insert(username.to_string(), sender.clone());
                }

                let request_info = RequestInfo::new_now(request);

                let mut clients_mx = self.clients.lock().unwrap();
                let handler = clients_mx.get_mut(&addr).expect("client must have a handler");
                handler.handle(request_info)?
            };

            eprintln!("{:?}", result.response);
            let RequestResult { response, new_handler } = result;

            response.write_to(&mut client)?;

            if let Some(handler) = new_handler {
                let mut clients_mx = self.clients.lock().unwrap();
                clients_mx.insert(addr, handler);
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] io::Error),

    #[error(transparent)]
    Messages(#[from] messages::Error),

    #[error(transparent)]
    Handler(#[from] handler::Error),
}
