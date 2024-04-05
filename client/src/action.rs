use iced::Command;
use trivia::messages::Request;

use crate::page::Page;
use crate::message::Message;

pub enum Action {
    Switch(Box<dyn Page>, Command<Message>),
    MakeRequest(Request),
    Command(Command<Message>), // focus a text input
    Nothing,
}

impl Action {
    pub fn switch(page: impl Page + 'static) -> Self {
        Self::Switch(Box::new(page), Command::none())
    }

    pub fn switch_cmd(page: impl Page + 'static, cmd: Command<Message>) -> Self {
        Self::Switch(Box::new(page), cmd)
    }

    pub fn request(request: Request) -> Self {
        Self::MakeRequest(request)
    }

    pub fn cmd(cmd: Command<Message>) -> Self {
        Self::Command(cmd)
    }

    pub fn none() -> Self {
        Self::Nothing
    }
}
