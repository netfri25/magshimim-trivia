use std::io::{Read, Write};

use serde::{Deserialize, Serialize};

use crate::handler::Handler;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
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

    pub fn new_error(msg: impl ToString) -> Self {
        let msg = msg.to_string();
        Self::Error { msg }
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
        Self::without_handler(Response::new_error(msg))
    }
}

mod tests {
    #[test]
    fn serde() {
        use std::io::Cursor;
        use super::Response;

        let to_test = [
            Response::Error { msg: "some error".into() },
            Response::Login { status: 3 },
            Response::Signup { status: 12 },
        ];

        for original_response in to_test {
            let mut buf = Vec::new();
            original_response.write_to(&mut buf).unwrap();
            let mut reader = Cursor::new(buf);
            let parsed_response = Response::read_from(&mut reader).unwrap();
            assert_eq!(original_response, parsed_response);
        }
    }
}
