use crate::{action::Action, message::Message};

pub mod login;

pub trait Page {
    fn update(&mut self, message: Message) -> Action;
    fn view(&self) -> iced::Element<Message>;
}
