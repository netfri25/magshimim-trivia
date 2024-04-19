use iced::Command;
use trivia::messages::Request;

use crate::page::Page;
use crate::message::Message;

pub enum Action {
    Switch(Box<dyn Page>, Option<Request>),
    MakeRequest(Request),
    Command(Command<Message>), // focus a text input
    Quit,
    Nothing,
}

impl Action {
    pub fn switch(page: impl Page + 'static) -> Self {
        Self::Switch(Box::new(page), None)
    }

    pub fn switch_and_request(page: impl Page + 'static, req: Request) -> Self {
        Self::Switch(Box::new(page), Some(req))
    }

    pub fn request(request: Request) -> Self {
        Self::MakeRequest(request)
    }

    pub fn cmd(cmd: Command<Message>) -> Self {
        Self::Command(cmd)
    }

    pub fn quit() -> Self {
        Self::Quit
    }

    pub fn none() -> Self {
        Self::Nothing
    }
}
