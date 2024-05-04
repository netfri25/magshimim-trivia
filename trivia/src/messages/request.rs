use std::borrow::Cow;
use std::io::{Read, Write};
use std::time::{Duration, Instant};

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::db::question::QuestionData;
use crate::managers::room::RoomID;

use super::{Address, Error};

pub const DATE_FORMAT: &str = "%d/%m/%Y";

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum Request<'a> {
    Login {
        #[serde(borrow)]
        username: Cow<'a, str>,
        #[serde(borrow)]
        password: Cow<'a, str>,
    },
    Signup {
        #[serde(borrow)]
        username: Cow<'a, str>,
        #[serde(borrow)]
        password: Cow<'a, str>,
        #[serde(borrow)]
        email: Cow<'a, str>,
        #[serde(borrow)]
        phone: Cow<'a, str>,
        address: Address,
        birth_date: NaiveDate,
    },
    JoinRoom(RoomID),
    CreateRoom {
        #[serde(borrow)]
        name: Cow<'a, str>,
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
    LeaveGame,
    Question,
    SubmitAnswer(Cow<'a, str>),
    GameResult,
    CreateQuestion(QuestionData),
}

impl<'buf> Request<'buf> {
    pub fn read_from(buf: &'buf mut Vec<u8>, reader: &mut impl Read) -> Result<Self, Error> {
        let mut buf_data_len = [0; 4];
        reader.read_exact(&mut buf_data_len)?;
        let data_len = u32::from_le_bytes(buf_data_len);
        let data_len = data_len as usize;

        buf.clear();
        buf.resize(data_len, 0);
        reader.read_exact(buf)?;

        let request = serde_json::from_slice(buf)?;
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

#[derive(Debug)]
pub struct RequestInfo<'a> {
    pub data: Request<'a>,
    pub time: Instant,
}

impl<'a> RequestInfo<'a> {
    pub fn new(data: Request<'a>, time: Instant) -> Self {
        Self { data, time }
    }

    pub fn new_now(kind: Request<'a>) -> Self {
        Self::new(kind, Instant::now())
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use crate::messages::{Address, DATE_FORMAT};

    #[test]
    fn serde() {
        use super::Request;
        use std::io::Cursor;

        let to_test = [
            Request::Signup {
                username: "user1234".into(),
                password: "Pass@123".into(),
                email: "example@mail.com".into(),
                phone: "052-1122333".into(),
                address: Address::new("Netanya", "Alonim", 69),
                birth_date: NaiveDate::parse_from_str("22/04/2038", DATE_FORMAT).unwrap(),
            },
            Request::Login {
                username: "user1234".into(),
                password: "Pass@123".into(),
            },
        ];

        for original_response in to_test {
            let mut buf = Vec::new();
            original_response.write_to(&mut buf).unwrap();
            let mut reader = Cursor::new(buf);
            let mut output = Vec::new();
            let parsed_response = Request::read_from(&mut output, &mut reader).unwrap();
            assert_eq!(original_response, parsed_response);
        }
    }
}
