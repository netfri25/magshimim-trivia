use iced::{
    alignment::Horizontal, theme, widget::{button, column, container, horizontal_space, row, text, text_input}, Alignment, Length
};

use crate::message::Message;
use crate::{action::Action, consts};

use super::{LoginPage, Page};

#[derive(Debug, Clone)]
pub enum Msg {
    UsernameInput(String),
    PasswordInput(String),
    EmailInput(String),
    Register,
    Login,
}

#[derive(Default)]
pub struct RegisterPage {
    username: String,
    password: String,
    email: String,
}

impl Page for RegisterPage {
    fn update(&mut self, message: Message) -> Action {
        let Message::Register(msg) = message else {
            return Action::Nothing;
        };

        match msg {
            Msg::UsernameInput(username) => self.username = username,
            Msg::PasswordInput(password) => self.password = password,
            Msg::EmailInput(email) => self.email = email,
            Msg::Register => {
                return Action::MakeRequest(trivia::messages::Request::Signup {
                    username: self.username.clone(),
                    password: self.password.clone(),
                    email: self.email.clone(),
                })
            }
            Msg::Login => return Action::GoTo(Box::<LoginPage>::default()),
        }

        Action::Nothing
    }

    fn view(&self) -> iced::Element<Message> {
        let title = text("Register")
            .size(consts::TITLE_SIZE)
            .width(Length::Fill)
            .horizontal_alignment(Horizontal::Center);
        let subtitle = text("Register a new Trivia Account")
            .size(consts::SUBTITLE_SIZE)
            .width(Length::Fill)
            .horizontal_alignment(Horizontal::Center)
            .style(consts::SUBTITLE_COLOR);

        let register_button = button("Register").on_press(Msg::Register.into());
        let already_have_an_account_text = text("already have an account?")
            .style(consts::ALREADY_HAVE_AN_ACCOUNT_COLOR)
            .size(consts::ALREADY_HAVE_AN_ACCOUNT_SIZE);
        let already_have_an_account_button = button(already_have_an_account_text)
            .on_press(Msg::Login.into())
            .style(theme::Button::Text);
        let buttons = row![
            already_have_an_account_button,
            horizontal_space(),
            register_button
        ].align_items(Alignment::Center);

        let input_fields = column![
            text_input("username:", &self.username)
                .on_input(|input| Msg::UsernameInput(input).into()),
            text_input("password:", &self.password)
                .secure(true)
                .on_input(|input| Msg::PasswordInput(input).into()),
            text_input("email:", &self.email).on_input(|input| Msg::EmailInput(input).into()),
            container(buttons).padding(consts::BUTTONS_PADDING).center_y()
        ]
        .spacing(consts::INPUT_FIELDS_SPACING)
        .padding(consts::INPUT_FIELDS_PADDING)
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
                .center_y()
        ];

        container(body)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}
