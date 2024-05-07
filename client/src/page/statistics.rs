use iced::alignment::{Horizontal, Vertical};
use iced::widget::{button, column, container, text};
use iced::{Alignment, Length};
use trivia::messages::{Request, Response};

use crate::action::Action;
use crate::consts;
use crate::message::Message;

use super::{HighScoresPage, MainMenuPage, Page, PersonalStatsPage};

#[derive(Debug, Clone)]
pub enum Msg {
    PersonalStats,
    Highscores,
}

pub struct StatisticsPage;

impl Page for StatisticsPage {
    fn update(&mut self, message: Message) -> Action {
        if let Message::Response(response) = message {
            match response.as_ref() {
                Response::PersonalStats(stats) => {
                    return Action::switch(PersonalStatsPage::new(stats.clone()))
                }

                Response::Highscores(highscores) => {
                    return Action::switch(HighScoresPage::new(highscores.clone()));
                }

                _ => eprintln!("response ignored: {:?}", response),
            }

            return Action::none();
        }

        let Message::Statistics(msg) = message else {
            return Action::none();
        };

        match msg {
            Msg::PersonalStats => Action::request(Request::PersonalStats),
            Msg::Highscores => Action::request(Request::Highscores),
        }
    }

    fn view(&self) -> iced::Element<Message> {
        let title = text("Statistics")
            .size(consts::TITLE_SIZE)
            .width(Length::Fill)
            .horizontal_alignment(Horizontal::Center);

        let personal_stats_button = menu_button("My Statistics", Msg::PersonalStats);
        let high_scores_button = menu_button("High Scores", Msg::Highscores);

        let buttons = container(
            column![personal_stats_button, high_scores_button,]
                .align_items(Alignment::Center)
                .width(Length::Fill)
                .spacing(35),
        )
        .center_y();

        container(column![
            container(title)
                .height(Length::FillPortion(1))
                .padding(consts::TITLES_PADDING * 2),
            buttons.height(Length::FillPortion(3)),
        ])
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .into()
    }

    fn quit(&mut self) -> Action {
        Action::switch(MainMenuPage)
    }
}

fn menu_button(button_text: &'static str, msg: Msg) -> iced::Element<Message> {
    let button_text = text(button_text)
        .size(30)
        .horizontal_alignment(Horizontal::Center)
        .vertical_alignment(Vertical::Center);

    button(button_text)
        .width(200)
        .height(80)
        .on_press(msg.into())
        .into()
}
