use std::time::Duration;

use iced::alignment::{Horizontal, Vertical};
use iced::widget::{
    button, column, container, horizontal_space, row, text, vertical_space, Column,
};
use iced::{theme, Alignment, Length};
use trivia::db::question::QuestionData;
use trivia::messages::{Request, Response};

use crate::action::Action;
use crate::consts;
use crate::message::Message;

use super::{Page, ResultsPage};

#[derive(Debug, Clone)]
pub enum Msg {
    NextQuestion,
    SelectAnswer(String),
    Tick(Duration),
}

pub struct GamePage {
    question: Option<QuestionData>,
    selected_answer: Option<String>,
    correct_answer: Option<String>,
    time_per_question: Duration,
    time_left: Duration,
}

impl GamePage {
    pub fn new(time_per_question: Duration) -> Self {
        Self {
            time_per_question,
            question: None,
            selected_answer: None,
            correct_answer: None,
            time_left: Duration::default(),
        }
    }
}

impl Page for GamePage {
    fn update(&mut self, message: Message) -> Action {
        if let Message::Response(response) = message {
            match response.as_ref() {
                Response::CorrectAnswer(correct_answer_index) => {
                    self.correct_answer = Some(correct_answer_index.clone())
                }

                Response::Question(question) => {
                    if let Some(question) = question {
                        self.question = Some(question.clone());
                        self.time_left = self.time_per_question;
                        self.selected_answer = None;
                        self.correct_answer = None;
                    } else {
                        return Action::switch_and_request(
                            ResultsPage::default(),
                            Request::GameResult,
                        );
                    }
                }

                _ => eprintln!("response ignored: {:?}", response),
            }

            return Action::none();
        }

        let Message::Game(msg) = message else {
            return Action::none();
        };

        match msg {
            Msg::NextQuestion => Action::request(Request::Question),
            Msg::SelectAnswer(answer) => {
                self.selected_answer = Some(answer.clone());
                Action::request(Request::SubmitAnswer(answer))
            }

            Msg::Tick(tick) => {
                self.time_left = self.time_left.saturating_sub(tick);
                if self.time_left.is_zero() {
                    self.question = None;
                    Action::request(Request::Question)
                } else {
                    Action::none()
                }
            }
        }
    }

    fn view(&self) -> iced::Element<Message> {
        let Some(ref question) = self.question else {
            return container("Waiting for question...")
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x()
                .center_y()
                .into();
        };

        let question_content = text(&question.question)
            .size(consts::TITLE_SIZE / 2)
            .width(Length::Fill)
            .horizontal_alignment(Horizontal::Center);

        let answers = Column::from_vec(
            question
                .answers
                .iter()
                .flat_map(|answer| {
                    [
                        vertical_space().into(),
                        answer_elem(
                            answer,
                            self.correct_answer.as_deref().and_then(|correct| {
                                self.selected_answer
                                    .as_deref()
                                    .map(|selected| (correct, selected))
                            }),
                        ),
                    ]
                })
                .skip(1)
                .collect(),
        )
        .height(Length::Fill)
        .align_items(Alignment::Center);

        let next_button_col =
            Column::new()
                .push(vertical_space())
                .push_maybe(self.correct_answer.as_ref().map(|_| {
                    button(text("Next Question").size(30)).on_press(Msg::NextQuestion.into())
                }))
                .width(Length::Fill)
                .height(Length::Fill)
                .align_items(Alignment::Center)
                .padding(10)
                .spacing(20);

        let timer = text(format!("time left: {:.01}s", self.time_left.as_secs_f32()))
            .size(20)
            .width(Length::Fill)
            .horizontal_alignment(Horizontal::Center);

        container(
            column![
                question_content,
                timer,
                row![
                    horizontal_space().width(Length::FillPortion(3)),
                    answers.width(Length::FillPortion(5)),
                    next_button_col.width(Length::FillPortion(3))
                ]
            ]
            .padding(10)
            .spacing(30),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .into()
    }

    fn subscription(&self) -> iced::Subscription<Message> {
        const TICK: Duration = Duration::from_millis(100);

        // only tick when the user hasn't selected an answer yet
        if self.selected_answer.is_none() {
            iced::time::every(TICK).map(|_| Msg::Tick(TICK).into())
        } else {
            iced::Subscription::none()
        }
    }
}

fn answer_elem<'a>(
    answer: &str,
    correct_and_selected: Option<(&str, &str)>,
) -> iced::Element<'a, Message> {
    let content = text(answer)
        .width(Length::Fill)
        .height(Length::Fill)
        .horizontal_alignment(Horizontal::Center)
        .vertical_alignment(Vertical::Center);

    let mut but = button(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .on_press_maybe(
            correct_and_selected
                .is_none()
                .then(|| Msg::SelectAnswer(answer.to_string()).into()),
        );

    if let Some((correct, selected)) = correct_and_selected {
        if answer == correct {
            but = but.style(theme::Button::Positive)
        } else if answer == selected {
            but = but.style(theme::Button::Destructive)
        } else {
            but = but.style(theme::Button::Secondary)
        }
    };

    but.into()
}
