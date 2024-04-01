use std::time::Instant;
use std::io::{Read, Write};

use serde::{Deserialize, Serialize};

use crate::handler::Handler;

#[derive(Debug, Serialize, Deserialize)]
pub enum RequestKind {
    Login {
        username: String,
        password: String,
    },

    Signup {
        username: String,
        password: String,
        email: String,
    },
}

impl RequestKind {
    #[must_use]
    pub fn is_signup(&self) -> bool {
        matches!(self, Self::Signup { .. })
    }

    #[must_use]
    pub fn is_login(&self) -> bool {
        matches!(self, Self::Login { .. })
    }
}

pub struct Request {
    pub kind: RequestKind,
    pub time: Instant,
}

impl Request {
    pub fn new(kind: RequestKind, time: Instant) -> Self {
        Self { kind, time }
    }

    pub fn new_now(kind: RequestKind) -> Self {
        Self::new(kind, Instant::now())
    }

    pub fn read_from(reader: &mut impl Read) -> std::io::Result<Self> {
        let mut buf_data_len = [0; 4];
        reader.read_exact(&mut buf_data_len)?;
        let data_len = u32::from_le_bytes(buf_data_len);
        let data_len = data_len as usize;

        let mut buf = vec![0; data_len];
        reader.read_exact(&mut buf)?;

        let request_kind = serde_json::from_slice(&buf)?;
        Ok(Self::new_now(request_kind))
    }

    pub fn write_to(&self, writer: &mut impl Write) -> std::io::Result<()> {
        let json = serde_json::to_vec(&self.kind)?;
        let len = json.len() as u32;
        let len_bytes = len.to_le_bytes();
        writer.write_all(&len_bytes)?;
        writer.write_all(&json)?;
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Response {
    Error { msg: String },

    Login { status: u64 },

    Signup { status: u64 },
}

impl Response {
    pub fn read_from(reader: &mut impl Read) -> std::io::Result<Self> {
        let mut buf_data_len = [0; 4];
        reader.read_exact(&mut buf_data_len)?;
        let data_len = u32::from_le_bytes(buf_data_len);
        let data_len = data_len as usize;

        let mut buf = vec![0; data_len];
        reader.read_exact(&mut buf)?;

        let response = serde_json::from_slice(&buf)?;
        Ok(response)
    }

    pub fn write_to(&self, writer: &mut impl Write) -> std::io::Result<()> {
        let json = serde_json::to_vec(self)?;
        let len = json.len() as u32;
        let len_bytes = len.to_le_bytes();
        writer.write_all(&len_bytes)?;
        writer.write_all(&json)?;
        Ok(())
    }
}

pub struct RequestResult {
    pub response: Response,
    pub new_handler: Option<Box<dyn Handler>>,
}

impl RequestResult {
    pub fn new(response: Response, new_handler: Option<impl Handler + 'static>) -> Self {
        let new_handler = new_handler.map(|h| Box::new(h) as Box<dyn Handler>);
        Self {
            response,
            new_handler,
        }
    }

    pub fn without_handler(response: Response) -> Self {
        Self {
            response,
            new_handler: None,
        }
    }

    pub fn new_error(msg: impl ToString) -> Self {
        let msg = msg.to_string();
        let response = Response::Error { msg };
        Self::without_handler(response)
    }
}
