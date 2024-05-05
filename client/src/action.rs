use iced::Command;
use trivia::messages::Request;

use crate::message::Message;
use crate::page::Page;

pub enum Action {
    Switch(Box<dyn Page>, Option<Request<'static>>),
    MakeRequest(Request<'static>),
    Command(Command<Message>), // focus a text input
    Nothing,
}

impl Action {
    pub fn switch(page: impl Page + 'static) -> Self {
        Self::Switch(Box::new(page), None)
    }

    pub fn switch_and_request(page: impl Page + 'static, req: Request<'static>) -> Self {
        Self::Switch(Box::new(page), Some(req))
    }

    pub fn request(request: Request<'static>) -> Self {
        Self::MakeRequest(request)
    }

    pub fn cmd(cmd: Command<Message>) -> Self {
        Self::Command(cmd)
    }

    pub fn none() -> Self {
        Self::Nothing
    }
}
