use crate::message::Message;
use crate::action::Action;

pub mod login;
pub mod register;

pub trait Page {
    fn update(&mut self, message: Message) -> Action;
    fn view(&self) -> iced::Element<Message>;
}
