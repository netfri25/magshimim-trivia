use std::cmp::Ordering;

use iced::alignment::Horizontal;
use iced::widget::scrollable::Properties;
use iced::widget::{
    button, column, container, horizontal_space, radio, row, scrollable, text, text_input,
    vertical_space, Column,
};
use iced::{theme, Alignment, Color, Length};
use trivia::db::question::QuestionData;
use trivia::messages::{Request, Response};

use crate::action::Action;
use crate::consts;
use crate::message::Message;

use super::{MainMenuPage, Page};

#[derive(Debug, Clone)]
pub enum Msg {
    UpdateQuestion(String),
    UpdateAnswer(String, usize),
    MarkCorrect(usize),
    AddAnswer,
    RemoveAnswer(usize),
    SubmitAnswer(usize),
    SubmitQuestion,
    Create,
}

#[derive(Default)]
pub struct CreateQuestionPage {
    question: String,
    answers: Vec<String>,
    correct_answer_index: Option<usize>,
}

impl Page for CreateQuestionPage {
    fn update(&mut self, message: Message) -> Result<Action, String> {
        if let Message::Response(response) = message {
            match response.as_ref() {
                Response::CreateQuestion(res) => {
                    return if let Err(err) = res {
                        Err(err.to_string())
                    } else {
                        Ok(Action::switch(MainMenuPage))
                    }
                }
                _ => eprintln!("response ignored: {:?}", response),
            }

            return Ok(Action::none());
        }

        let Message::CreateQuestion(msg) = message else {
            return Ok(Action::none());
        };

        match msg {
            Msg::UpdateQuestion(question) => self.question = question,
            Msg::UpdateAnswer(answer, index) => self.answers[index] = answer,
            Msg::MarkCorrect(index) => self.correct_answer_index = Some(index),
            Msg::AddAnswer => self.answers.push(String::default()),
            Msg::RemoveAnswer(index) => {
                self.answers.remove(index);
                if let Some(correct_index) = self.correct_answer_index {
                    self.correct_answer_index = match index.cmp(&correct_index) {
                        Ordering::Less => Some(correct_index - 1),
                        Ordering::Equal => None,
                        Ordering::Greater => Some(correct_index),
                    };
                }
            }

            Msg::SubmitAnswer(index) => {
                let focus_index = index + 1;
                if self.answers.len() == focus_index {
                    self.answers.push(String::default());
                }

                return Ok(Action::cmd(text_input::focus(answer_id(focus_index))));
            }

            Msg::SubmitQuestion => {
                if self.answers.is_empty() {
                    self.answers.push(String::default())
                }

                return Ok(Action::cmd(text_input::focus(answer_id(0))));
            }

            Msg::Create => {
                // I can safely unwrap here because I make sure that the Submit button is only
                // pressable when `self.correct_answer_index.is_some()`
                let question = QuestionData::new(
                    self.question.clone(),
                    self.answers.clone(),
                    self.correct_answer_index.unwrap(),
                );
                return Ok(Action::request(Request::CreateQuestion(question)));
            }
        }

        Ok(Action::none())
    }

    fn view(&self) -> iced::Element<Message> {
        let title = text("Create Question")
            .size(consts::TITLE_SIZE)
            .width(Length::Fill)
            .horizontal_alignment(Horizontal::Center);

        let question = text_input("question:", &self.question)
            .on_input(|s| Msg::UpdateQuestion(s).into())
            .on_submit(Msg::SubmitQuestion.into())
            .width(Length::Fill);

        let answers_col = Column::from_vec(
            self.answers
                .iter()
                .enumerate()
                .map(|(i, ans)| answer(i, ans, self.correct_answer_index))
                .collect(),
        )
        .spacing(20)
        .padding(4)
        .align_items(Alignment::Start)
        .width(Length::Fill);

        let answers = scrollable(answers_col)
            .direction(scrollable::Direction::Vertical(
                Properties::new().width(3).scroller_width(1.5),
            ))
            .height(Length::Fill);

        let submit_text = text("Submit")
            .size(30)
            .width(Length::Fill)
            .horizontal_alignment(Horizontal::Center);
        let allowed_to_submit = self.correct_answer_index.is_some()
            && self.answers.iter().all(|s| !s.is_empty())
            && self.answers.len() >= 2;
        let submit_button =
            button(submit_text).on_press_maybe(allowed_to_submit.then_some(Msg::Create.into()));

        let add_answer_text = text("Add Answer")
            .size(20)
            .width(Length::Fill)
            .horizontal_alignment(Horizontal::Center);
        let add_answer_button = button(add_answer_text).on_press(Msg::AddAnswer.into());

        let page = column![
            title,
            row![
                horizontal_space().width(Length::FillPortion(3)),
                column![
                    question,
                    row![
                        horizontal_space().width(Length::FillPortion(1)),
                        answers.width(Length::FillPortion(10)),
                        horizontal_space().width(Length::FillPortion(1)),
                    ],
                    row![
                        horizontal_space().width(Length::FillPortion(1)),
                        add_answer_button.width(Length::FillPortion(11)),
                        horizontal_space().width(Length::FillPortion(1)),
                    ],
                ]
                .spacing(10)
                .width(Length::FillPortion(7))
                .height(Length::Fill)
                .align_items(Alignment::Center),
                column![vertical_space(), submit_button]
                    .padding(10)
                    .width(Length::FillPortion(3))
                    .height(Length::Fill)
                    .align_items(Alignment::Center),
            ]
            .width(Length::Fill)
            .height(Length::Fill)
            .align_items(Alignment::Center)
        ]
        .spacing(10)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_items(Alignment::Center);

        container(page)
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

fn answer(
    index: usize,
    answer: &str,
    correct_answer_index: Option<usize>,
) -> iced::Element<Message> {
    let input = text_input(&format!("answer {}:", index + 1), answer)
        .on_input(move |s| Msg::UpdateAnswer(s, index).into())
        .on_submit(Msg::SubmitAnswer(index).into())
        .id(answer_id(index))
        .width(Length::Fill);

    let select_correct = radio(String::new(), index, correct_answer_index, |i| {
        Msg::MarkCorrect(i).into()
    });

    let remove_answer_text = text("x").style(theme::Text::Color(Color::from_rgb(0.8, 0.2, 0.1)));
    let remove_answer_button = button(remove_answer_text)
        .style(theme::Button::Text)
        .on_press(Msg::RemoveAnswer(index).into());

    row![select_correct, input, remove_answer_button]
        .spacing(5)
        .into()
}

fn answer_id(index: usize) -> text_input::Id {
    text_input::Id::new(format!("answer{}", index))
}
