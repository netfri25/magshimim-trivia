use std::sync::{Arc, Mutex};
use std::net::{TcpStream, ToSocketAddrs};

use trivia::messages::{Request, Response};

#[derive(Debug, Default, Clone)]
pub struct Connection {
    stream: Arc<Mutex<Option<TcpStream>>>
}

impl Connection {
    pub fn connect(&self, addr: impl ToSocketAddrs) -> Result<(), Error> {
        let stream = TcpStream::connect(addr)?;
        *self.stream.lock().unwrap() = Some(stream);
        Ok(())
    }

    pub fn send(&self, msg: Request) -> Result<(), Error> {
        let mut stream_lock = self.stream.lock().unwrap();
        let Some(ref mut stream) = *stream_lock else {
            return Err(Error::NotConnected)
        };

        msg.write_to(stream)?;

        Ok(())
    }

    pub fn recv(&self) -> Result<Response, Error> {
        let mut stream_lock = self.stream.lock().unwrap();
        let Some(ref mut stream) = *stream_lock else {
            return Err(Error::NotConnected)
        };

        let response = Response::read_from(stream)?;

        Ok(response)
    }

    pub async fn send_recv(&self, msg: Request) -> Result<Response, Error> {
        async { self.send(msg) }.await?;
        async { self.recv() }.await
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    TriviaMsg(#[from] trivia::messages::Error),

    #[error("Client isn't connected to the server")]
    NotConnected,

    #[error("{0}")]
    ResponseError(String),
}
