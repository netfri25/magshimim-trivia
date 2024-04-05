use iced::{
    alignment::Horizontal, theme, widget::{button, column, container, horizontal_space, row, text, text_input}, Alignment, Length
};

use crate::message::Message;
use crate::{action::Action, consts};

use super::{register::RegisterPage, Page};

#[derive(Debug, Clone)]
pub enum Msg {
    UsernameInput(String),
    PasswordInput(String),
    Login,
    Register,
}

#[derive(Default)]
pub struct LoginPage {
    username: String,
    password: String,
}

impl Page for LoginPage {
    fn update(&mut self, message: Message) -> Action {
        let Message::Login(msg) = message else {
            return Action::Nothing;
        };

        match msg {
            Msg::UsernameInput(username) => self.username = username,
            Msg::PasswordInput(password) => self.password = password,
            Msg::Login => {
                return Action::MakeRequest(trivia::messages::Request::Login {
                    username: self.username.clone(),
                    password: self.password.clone(),
                })
            }
            Msg::Register => return Action::GoTo(Box::<RegisterPage>::default()),
        }

        Action::Nothing
    }

    fn view(&self) -> iced::Element<Message> {
        let title = text("Login")
            .size(consts::TITLE_SIZE)
            .width(Length::Fill)
            .horizontal_alignment(Horizontal::Center);
        let subtitle = text("Login with your Trivia Account")
            .size(consts::SUBTITLE_SIZE)
            .width(Length::Fill)
            .horizontal_alignment(Horizontal::Center)
            .style(consts::SUBTITLE_COLOR);

        let login_button = button("Login").on_press(Msg::Login.into());
        let register_button = button("Register")
            .on_press(Msg::Register.into())
            .style(theme::Button::Secondary);
        let buttons =
            row![register_button, horizontal_space(), login_button].align_items(Alignment::Center);

        let input_fields = column![
            text_input("username:", &self.username)
                .on_input(|input| Msg::UsernameInput(input).into()),
            text_input("password:", &self.password)
                .secure(true)
                .on_input(|input| Msg::PasswordInput(input).into()),
            container(buttons).padding(2.).center_y(),
        ]
        .padding(consts::INPUT_FIELDS_PADDING)
        .spacing(consts::INPUT_FIELDS_SPACING)
        .max_width(consts::INPUT_FIELDS_MAX_WIDTH);

        let body = column![
            container(
                column![title, subtitle]
                    .padding(consts::TITLES_PADDING)
                    .spacing(consts::TITLES_SPACING)
            )
            .height(consts::TITLES_PORTION)
            .center_y(),
            container(input_fields)
                .width(Length::Fill)
                .height(consts::INPUT_FIELDS_PORTION)
                .center_x()
                .center_y(),
        ];

        container(body)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}
