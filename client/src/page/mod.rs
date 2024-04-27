use crate::action::Action;
use crate::message::Message;

pub mod login;
pub use login::LoginPage;

pub mod register;
pub use register::RegisterPage;

pub mod mainmenu;
pub use mainmenu::MainMenuPage;

pub mod createroom;
pub use createroom::CreateRoomPage;

pub mod joinroom;
pub use joinroom::JoinRoomPage;

pub mod room;
pub use room::RoomPage;

pub mod statistics;
pub use statistics::StatisticsPage;

pub mod personalstats;
pub use personalstats::PersonalStatsPage;

pub mod highscores;
pub use highscores::HighScoresPage;

pub mod game;
pub use game::GamePage;

pub mod results;
pub use results::ResultsPage;

pub trait Page {
    fn update(&mut self, message: Message) -> Action;
    fn view(&self) -> iced::Element<Message>;

    fn subscription(&self) -> iced::Subscription<Message> {
        iced::Subscription::none()
    }
}
