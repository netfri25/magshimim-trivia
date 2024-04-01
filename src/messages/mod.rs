use std::time::Instant;

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
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Response {
    Error { msg: String },

    Login { status: u64 },

    Signup { status: u64 },
}

pub struct RequestResult {
    response: Response,
    new_handler: Option<Box<dyn Handler>>,
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
