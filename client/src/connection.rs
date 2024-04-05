use std::sync::{Arc, Mutex};
use std::net::{TcpStream, ToSocketAddrs};

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

    pub fn send(&self, msg: trivia::messages::Request) -> Result<(), Error> {
        let mut stream_lock = self.stream.lock().unwrap();
        let Some(ref mut stream) = *stream_lock else {
            return Err(Error::NotConnected)
        };

        msg.write_to(stream)?;

        Ok(())
    }

    pub fn recv(&self) -> Result<trivia::messages::Response, Error> {
        let mut stream_lock = self.stream.lock().unwrap();
        let Some(ref mut stream) = *stream_lock else {
            return Err(Error::NotConnected)
        };

        let response = trivia::messages::Response::read_from(stream)?;

        Ok(response)
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
