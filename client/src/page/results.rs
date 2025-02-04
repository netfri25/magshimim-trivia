use std::time::Duration;

use iced::alignment::{Horizontal, Vertical};
use iced::widget::scrollable::{Direction, Properties};
use iced::widget::{
    button, column, container, horizontal_space, row, scrollable, text, vertical_space, Column,
};
use iced::{Alignment, Length, Pixels, Subscription};
use trivia::messages::{PlayerResults, Request, Response};

use crate::action::Action;
use crate::consts;
use crate::message::Message;

use super::{MainMenuPage, Page};

#[derive(Debug, Clone)]
pub enum Msg {
    GetResults,
    Cry,
}

#[derive(Default)]
pub struct ResultsPage {
    results: Box<[PlayerResults]>,
}

impl Page for ResultsPage {
    fn update(&mut self, message: Message) -> Result<Action, String> {
        if let Message::Response(response) = message {
            match response.as_ref() {
                // the results are sent sorted
                Response::GameResult(results) => self.results = results.clone().into_boxed_slice(),

                _ => eprintln!("response ignored: {:?}", response),
            }

            return Ok(Action::none());
        }

        let Message::Results(msg) = message else {
            return Ok(Action::none());
        };

        Ok(match msg {
            Msg::GetResults => Action::request(Request::GameResult),
            Msg::Cry => Action::switch(MainMenuPage),
        })
    }

    fn view(&self) -> iced::Element<Message> {
        let title = text("Results")
            .size(consts::TITLE_SIZE)
            .width(Length::Fill)
            .horizontal_alignment(Horizontal::Center);

        let results: iced::Element<_> = if self.results.is_empty() {
            text("Waiting for all players")
                .size(consts::TITLE_SIZE / 2)
                .width(Length::FillPortion(8))
                .height(Length::Fill)
                .horizontal_alignment(Horizontal::Center)
                .vertical_alignment(Vertical::Center)
                .into()
        } else {
            scrollable(
                Column::from_vec(self.results.iter().map(result_elem).collect())
                    .spacing(20)
                    .padding(5)
                    .width(Length::Fill),
            )
            .direction(Direction::Vertical(
                Properties::new().width(3).scroller_width(1.5),
            ))
            .width(Length::FillPortion(8))
            .height(Length::Fill)
            .into()
        };

        let cry_button = button(text("Cry").size(30)).on_press(Msg::Cry.into());

        container(
            column![
                title.height(Length::FillPortion(2)),
                row![
                    horizontal_space().width(Length::FillPortion(3)),
                    results,
                    column![vertical_space(), cry_button]
                        .align_items(Alignment::Center)
                        .width(Length::FillPortion(3))
                        .height(Length::Fill)
                        .padding(10)
                        .spacing(20),
                ]
                .height(Length::FillPortion(5))
            ]
            .spacing(consts::TITLES_SPACING),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        if self.results.is_empty() {
            iced::time::every(Duration::from_secs(3)).map(|_| Msg::GetResults.into())
        } else {
            Subscription::none()
        }
    }

    fn quit(&mut self) -> Action {
        Action::switch(MainMenuPage)
    }
}

pub fn result_elem(result: &PlayerResults) -> iced::Element<Message> {
    let PlayerResults {
        username,
        correct_answers,
        wrong_answers,
        avg_time,
        score,
    } = result;

    column![
        text(username.as_ref())
            .size(20)
            .width(Length::Fill)
            .horizontal_alignment(Horizontal::Center),
        result_field("score", format!("{:.2}", score), 15),
        result_field("correct", format!("{}", correct_answers), 15),
        result_field("wrong", format!("{}", wrong_answers), 15),
        result_field("average time", format!("{:.2?}", avg_time), 15),
    ]
    .width(Length::Fill)
    .height(150)
    .align_items(Alignment::Center)
    .padding(2)
    .spacing(2)
    .into()
}

fn result_field(
    name: &str,
    value: String,
    size: impl Into<Pixels> + Copy,
) -> iced::Element<Message> {
    row![
        text(format!("{}: ", name)).size(size),
        horizontal_space(),
        text(value).size(size)
    ]
    .width(Length::Fill)
    .into()
}
