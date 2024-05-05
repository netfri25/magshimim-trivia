use std::ops::RangeInclusive;
use std::time::Duration;

use iced::alignment::{Horizontal, Vertical};
use iced::widget::{button, column, container, horizontal_space, row, slider, text, text_input};
use iced::{Alignment, Length};
use trivia::messages::{Request, Response};

use crate::action::Action;
use crate::consts;
use crate::message::Message;
use crate::page::RoomPage;

use super::{MainMenuPage, Page};

#[derive(Debug, Clone)]
pub enum Msg {
    NameInput(String),
    MaxUsersInput(u32),
    QuestionsInput(u32),
    AnswerTimeoutInput(u32),
    Submit,
}

pub struct CreateRoomPage {
    name: String,
    max_users: u32,
    questions: u32,
    answer_timeout: u32, // in seconds
}

impl Default for CreateRoomPage {
    fn default() -> Self {
        Self {
            name: Default::default(),
            max_users: 1,
            questions: 1,
            answer_timeout: 5,
        }
    }
}

impl Page for CreateRoomPage {
    fn update(&mut self, message: Message) -> Action {
        if let Message::Response(response) = message {
            match response.as_ref() {
                &Response::CreateRoom => {
                    let page = RoomPage::new(true);
                    let req = Request::RoomState;
                    return Action::switch_and_request(page, req);
                }

                _ => eprintln!("response ignored: {:?}", response),
            }

            return Action::none();
        };

        let Message::CreateRoom(msg) = message else {
            return Action::none();
        };

        match msg {
            Msg::NameInput(name) => self.name = name,
            Msg::MaxUsersInput(max_users) => self.max_users = max_users,
            Msg::QuestionsInput(questions) => self.questions = questions,
            Msg::AnswerTimeoutInput(answer_timeout) => self.answer_timeout = answer_timeout,
            Msg::Submit => {
                return Action::request(Request::CreateRoom {
                    name: self.name.clone().into(),
                    max_users: self.max_users as usize,
                    questions: self.questions as usize,
                    answer_timeout: Duration::from_secs(self.answer_timeout as u64),
                })
            }
        }

        Action::none()
    }

    fn view(&self) -> iced::Element<Message> {
        let title = container(
            text("Create Room")
                .size(consts::TITLE_SIZE)
                .width(Length::Fill)
                .horizontal_alignment(Horizontal::Center),
        )
        .padding(consts::TITLES_PADDING);

        let inputs = column![
            text_input("room name:", &self.name).on_input(|input| Msg::NameInput(input).into()),
            input_field("users count", 1..=20, self.max_users, 1, Msg::MaxUsersInput),
            input_field(
                "questions count",
                1..=30,
                self.questions,
                1,
                Msg::QuestionsInput
            ),
            input_field(
                "answer time (secs)",
                5..=120,
                self.answer_timeout,
                5,
                Msg::AnswerTimeoutInput
            ),
        ]
        .align_items(Alignment::Center)
        .padding(consts::INPUT_FIELDS_PADDING)
        .spacing(consts::INPUT_FIELDS_SPACING);

        let submit_button = container(
            button(
                text("Create")
                    .size(40)
                    .vertical_alignment(Vertical::Center)
                    .horizontal_alignment(Horizontal::Center),
            )
            .on_press(Msg::Submit.into()),
        )
        .width(Length::Fill)
        .center_x()
        .center_y();

        let page = column![
            title.height(consts::TITLES_PORTION),
            row![
                horizontal_space().width(Length::FillPortion(1)),
                inputs.width(Length::FillPortion(3)),
                horizontal_space().width(Length::FillPortion(1)),
            ]
            .height(consts::INPUT_FIELDS_PORTION),
            submit_button.height(Length::FillPortion(4)),
        ]
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

fn input_field<'a>(
    name: &str,
    range: RangeInclusive<u32>,
    value: u32,
    step: u32,
    msg: impl Fn(u32) -> Msg + 'a,
) -> iced::Element<'a, Message> {
    row![
        text(format!("{}: {}", name, value)),
        horizontal_space().width(Length::Fill),
        slider(range, value, move |v| msg(v).into())
            .width(250.)
            .step(step)
    ]
    .into()
}
