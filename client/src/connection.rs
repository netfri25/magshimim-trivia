use std::net::{TcpStream, ToSocketAddrs};

use trivia::messages::{Request, Response};

#[derive(Debug, Default)]
pub struct Connection {
    stream: Option<TcpStream>,
}

impl Connection {
    pub fn connect(addr: impl ToSocketAddrs) -> Result<Self, Error> {
        let stream = TcpStream::connect(addr)?;
        let stream = Some(stream);
        Ok(Self { stream })
    }

    pub fn is_connected(&self) -> bool {
        self.stream.is_some()
    }

    pub fn send(&self, msg: Request) -> Result<(), Error> {
        let Some(mut stream) = self.stream.as_ref() else {
            return Err(Error::NotConnected)
        };

        msg.write_to(&mut stream)?;

        Ok(())
    }

    pub fn recv(&self) -> Result<Response, Error> {
        let Some(mut stream) = self.stream.as_ref() else {
            return Err(Error::NotConnected)
        };

        let response = Response::read_from(&mut stream)?;

        Ok(response)
    }

    pub async fn send_and_recv(&self, msg: Request) -> Result<Response, Error> {
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
    ResponseErr(String),
}
