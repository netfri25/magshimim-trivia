use std::io::{Read, Write};

use serde::{Deserialize, Serialize};

use crate::db::Score;
use crate::managers::statistics::Statistics;
use crate::managers::room::{Room, RoomID};
use crate::handler::Handler;

use super::StatusCode;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum Response {
    Error { msg: String },
    Login { status: StatusCode },
    Signup { status: StatusCode },
    Logout,
    Rooms(Vec<Room>),
    PlayersInRoom(Vec<String>),
    JoinRoom,
    CreateRoom(RoomID),
    HighScore {
        user_statistics: Statistics,
        high_scores: [Score; 5],
    }
}

impl Response {
    pub fn read_from(reader: &mut impl Read) -> anyhow::Result<Self> {
        let mut buf_data_len = [0; 4];
        reader.read_exact(&mut buf_data_len)?;
        let data_len = u32::from_le_bytes(buf_data_len);
        let data_len = data_len as usize;

        let mut buf = vec![0; data_len];
        reader.read_exact(&mut buf)?;

        let response = serde_json::from_slice(&buf)?;
        Ok(response)
    }

    pub fn write_to(&self, writer: &mut impl Write) -> anyhow::Result<()> {
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
    pub fn new(response: Response, new_handler: Box<dyn Handler>) -> Self {
        let new_handler = Some(new_handler);
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

#[cfg(test)]
mod tests {
    use crate::messages::StatusCode;

    #[test]
    fn serde() {
        use std::io::Cursor;
        use super::Response;

        let to_test = [
            Response::Error { msg: "some error".into() },
            Response::Login { status: StatusCode::ResponseOk },
            Response::Signup { status: StatusCode::ResponseOk },
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
