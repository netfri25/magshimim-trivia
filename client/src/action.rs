use iced::Command;

use crate::page::Page;
use crate::message::Message;

pub enum Action {
    GoTo(Box<dyn Page>),
    MakeRequest(trivia::messages::Request),
    Command(Command<Message>), // focus a text input
    Nothing,
}
