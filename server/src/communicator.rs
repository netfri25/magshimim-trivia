use std::cell::{Cell, RefCell};
use std::io;
use std::net::{TcpListener, TcpStream, ToSocketAddrs};

use trivia::db::Database;
use trivia::handler::{self, Handler, RequestHandlerFactory};
use trivia::messages::{self, Request, RequestInfo, RequestResult, Response};

use crate::defer::Defer;

pub struct Communicator<'db, DB: ?Sized> {
    socket: TcpListener,
    factory: RequestHandlerFactory<'db, DB>,
}

impl<'db, 'me: 'db, DB> Communicator<'db, DB>
where
    DB: Database + Sync + ?Sized,
{
    pub fn build(
        addr: impl ToSocketAddrs,
        factory: RequestHandlerFactory<'db, DB>,
    ) -> Result<Self, Error> {
        let socket = TcpListener::bind(addr)?;
        Ok(Self { socket, factory })
    }

    pub fn start_handle_requests(&'me self) {
        self.listen()
    }

    fn listen(&'me self) {
        std::thread::scope(|scope| {
            eprintln!("[INFO] listening: {:?}", self.socket);
            for client in self.socket.incoming() {
                let Ok(client) = client else {
                    eprintln!("[ERROR] connection error: {:?}", client);
                    continue;
                };

                eprintln!("[LOG] connected {:?}", client);

                let handler = self.factory.create_login_request_handler();
                scope.spawn(|| {
                    if let Err(err) = self.handle_new_client(client, handler) {
                        eprintln!("[ERROR] communication error: {err}");
                    }
                });
            }
        })
    }

    // returns the username, if the user has connected
    fn handle_new_client(
        &'me self,
        mut client: TcpStream,
        handler: impl Handler<'db> + 'db,
    ) -> Result<(), Error> {
        let addr = client.peer_addr()?;
        let login_username = Cell::new(None);
        let handler = RefCell::new(Box::new(handler) as Box<dyn Handler>);

        let _defer = Defer(|| {
            if let Some(ref username) = login_username.take() {
                eprintln!("[LOG] {:?} disconnected", username);
                self.factory
                    .login_manager()
                    .write()
                    .unwrap()
                    .logout(username);
            }

            let req = Request::Logout;
            handler.borrow_mut().handle(RequestInfo::new_now(req)).ok();
        });

        let mut buf = Vec::new();
        loop {
            let request = Request::read_from(&mut buf, &mut client);
            eprintln!("[REQ]:  {:?}", request);

            let request = match request {
                Ok(request) => request,
                Err(messages::Error::Json(err)) => {
                    return Response::new_error(err)
                        .write_to(&mut client)
                        .map_err(Into::into)
                }
                err => err?,
            };
            eprint!("[RESP]: ");

            // save the username, so it can be removed at the end of communication
            if let Request::Login { ref username, .. } = request {
                login_username.set(username.parse().ok());
            }

            let request_info = RequestInfo::new_now(request);
            if !handler.borrow().relevant(&request_info) {
                eprintln!("[INFO] Irrelevant request ({}): {:?}", addr, request_info);
                return Response::new_error("Irrelevant request")
                    .write_to(&mut client)
                    .map_err(Into::into);
            }

            let RequestResult {
                response,
                new_handler,
            } = handler.borrow_mut().handle(request_info)?;

            eprintln!("{:?}", response);
            response.write_to(&mut client)?;

            if let Some(new_handler) = new_handler {
                *handler.borrow_mut() = new_handler;
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
