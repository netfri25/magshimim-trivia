use crate::action::Action;
use crate::message::Message;

pub mod login;
pub use login::LoginPage;

pub mod register;
pub use register::RegisterPage;

pub mod main_menu;
pub use main_menu::MainMenuPage;

pub mod create_room;
pub use create_room::CreateRoomPage;

pub mod join_room;
pub use join_room::JoinRoomPage;

pub mod room;
pub use room::RoomPage;

pub mod statistics;
pub use statistics::StatisticsPage;

pub mod personal_stats;
pub use personal_stats::PersonalStatsPage;

pub mod highscores;
pub use highscores::HighScoresPage;

pub mod game;
pub use game::GamePage;

pub mod results;
pub use results::ResultsPage;

pub mod create_question;
pub use create_question::CreateQuestionPage;

pub trait Page {
    fn update(&mut self, message: Message) -> Result<Action, String>;
    fn view(&self) -> iced::Element<Message>;

    fn subscription(&self) -> iced::Subscription<Message> {
        iced::Subscription::none()
    }

    fn quit(&mut self) -> Action;
}
