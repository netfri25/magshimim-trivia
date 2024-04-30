use iced::alignment::Horizontal;
use iced::widget::{column, container, horizontal_space, row, text, Column};
use iced::{Alignment, Length};

use crate::action::Action;
use crate::consts;
use crate::message::Message;

use super::Page;

use trivia::managers::game::Score;
use trivia::managers::statistics::Highscores;

pub struct HighScoresPage {
    scores: Highscores,
}

impl Page for HighScoresPage {
    fn update(&mut self, _message: Message) -> Action {
        Action::none()
    }

    fn view(&self) -> iced::Element<Message> {
        let title = text("High Scores")
            .size(consts::TITLE_SIZE)
            .width(Length::Fill)
            .horizontal_alignment(Horizontal::Center);

        let users_col = Column::from_vec(
            self.scores
                .iter()
                .flatten()
                .map(|(username, score)| user_score(username, *score))
                .collect(),
        )
        .height(Length::Fill)
        .spacing(10)
        .align_items(Alignment::Center);

        let users = row![
            horizontal_space().width(Length::FillPortion(1)),
            users_col.width(Length::FillPortion(2)),
            horizontal_space().width(Length::FillPortion(1)),
        ]
        .width(Length::Fill)
        .align_items(Alignment::Center);

        container(column![
            container(title)
                .height(Length::FillPortion(1))
                .padding(consts::TITLES_PADDING * 2),
            users.height(Length::FillPortion(3)),
        ])
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .into()
    }
}

impl HighScoresPage {
    pub fn new(scores: Highscores) -> Self {
        Self { scores }
    }
}

fn user_score(username: &str, score: Score) -> iced::Element<Message> {
    container(text(format!("{} - {:.2}", username, score)).size(25))
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .into()
}
