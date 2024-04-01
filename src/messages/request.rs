use std::time::Instant;
use std::io::{Read, Write};

use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Request {
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

impl Request {
    #[must_use]
    pub fn is_signup(&self) -> bool {
        matches!(self, Self::Signup { .. })
    }

    #[must_use]
    pub fn is_login(&self) -> bool {
        matches!(self, Self::Login { .. })
    }

    pub fn read_from(reader: &mut impl Read) -> std::io::Result<Self> {
        let mut buf_data_len = [0; 4];
        reader.read_exact(&mut buf_data_len)?;
        let data_len = u32::from_le_bytes(buf_data_len);
        let data_len = data_len as usize;

        let mut buf = vec![0; data_len];
        reader.read_exact(&mut buf)?;

        let request = serde_json::from_slice(&buf)?;
        Ok(request)
    }

    pub fn write_to(&self, writer: &mut impl Write) -> std::io::Result<()> {
        let json = serde_json::to_vec(&self)?;
        let len = json.len() as u32;
        let len_bytes = len.to_le_bytes();
        writer.write_all(&len_bytes)?;
        writer.write_all(&json)?;
        Ok(())
    }
}

pub struct RequestInfo {
    pub data: Request,
    pub time: Instant,
}

impl RequestInfo {
    pub fn new(data: Request, time: Instant) -> Self {
        Self { data, time }
    }

    pub fn new_now(kind: Request) -> Self {
        Self::new(kind, Instant::now())
    }
}
