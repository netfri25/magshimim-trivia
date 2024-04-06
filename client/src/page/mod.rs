use crate::message::Message;
use crate::action::Action;

pub mod login;
pub use login::LoginPage;

pub mod register;
pub use register::RegisterPage;

pub mod mainmenu;
pub use mainmenu::MainMenuPage;

pub mod createroom;
pub use createroom::CreateRoomPage;

pub trait Page {
    fn update(&mut self, message: Message) -> Action;
    fn view(&self) -> iced::Element<Message>;
}
