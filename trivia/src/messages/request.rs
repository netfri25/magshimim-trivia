use std::time::{Duration, Instant};
use std::io::{Read, Write};

use serde::{Serialize, Deserialize};

use crate::managers::room::RoomID;

use super::Error;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
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
    JoinRoom(RoomID),
    CreateRoom {
        name: String,
        max_users: usize,
        questions: usize,
        answer_timeout: Duration,
    },
    Statistics,
    Logout,
    RoomList,
    CloseRoom,
    StartGame,
    RoomState,
    LeaveRoom,
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

    #[must_use]
    pub fn is_join_room(&self) -> bool {
        matches!(self, Self::JoinRoom(..))
    }

    #[must_use]
    pub fn is_create_room(&self) -> bool {
        matches!(self, Self::CreateRoom { .. })
    }

    #[must_use]
    pub fn is_statistics(&self) -> bool {
        matches!(self, Self::Statistics)
    }

    #[must_use]
    pub fn is_logout(&self) -> bool {
        matches!(self, Self::Logout)
    }

    #[must_use]
    pub fn is_room_list(&self) -> bool {
        matches!(self, Self::RoomList)
    }

    pub fn read_from(reader: &mut impl Read) -> Result<Self, Error> {
        let mut buf_data_len = [0; 4];
        reader.read_exact(&mut buf_data_len)?;
        let data_len = u32::from_le_bytes(buf_data_len);
        let data_len = data_len as usize;

        let mut buf = vec![0; data_len];
        reader.read_exact(&mut buf)?;

        let request = serde_json::from_slice(&buf)?;
        Ok(request)
    }

    pub fn write_to(&self, writer: &mut impl Write) -> Result<(), Error> {
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

#[cfg(test)]
mod tests {
    #[test]
    fn serde() {
        use std::io::Cursor;
        use super::Request;

        let to_test = [
            Request::Signup {
                username: "user1234".into(),
                password: "pass1234".into(),
                email: "example@mail.com".into(),
            },

            Request::Login {
                username: "user1234".into(),
                password: "pass1234".into(),
            },
        ];

        for original_response in to_test {
            let mut buf = Vec::new();
            original_response.write_to(&mut buf).unwrap();
            let mut reader = Cursor::new(buf);
            let parsed_response = Request::read_from(&mut reader).unwrap();
            assert_eq!(original_response, parsed_response);
        }
    }
}
