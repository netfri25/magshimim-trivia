use iced::alignment::Horizontal;
use iced::widget::{button, column, container, horizontal_space, row, text, text_input};
use iced::{theme, Alignment, Length};
use trivia::messages::{Request, Response};

use crate::action::Action;
use crate::consts;
use crate::message::Message;

use super::{MainMenuPage, Page, RegisterPage};

#[derive(Debug, Clone)]
pub enum Msg {
    UsernameInput(String),
    UsernameSubmit,
    PasswordInput(String),
    PasswordSubmit,
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
        if let Message::Response(response) = message {
            match response.as_ref() {
                Response::Login => {
                    return Action::switch(MainMenuPage);
                }
                _ => eprintln!("response ignored: {:?}", response),
            }

            return Action::none();
        }

        let Message::Login(msg) = message else {
            return Action::none();
        };

        match msg {
            Msg::UsernameInput(username) => self.username = username,
            Msg::UsernameSubmit => {
                return Action::cmd(text_input::focus(text_input::Id::new("password")))
            }

            Msg::PasswordInput(password) => self.password = password,
            Msg::PasswordSubmit | Msg::Login => {
                return Action::request(Request::Login {
                    username: self.username.clone().into(),
                    password: self.password.clone().into(),
                });
            }

            Msg::Register => return Action::switch(RegisterPage::default()),
        }

        Action::none()
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
                .id(text_input::Id::new("username"))
                .on_submit(Msg::UsernameSubmit.into())
                .on_input(|input| Msg::UsernameInput(input).into()),
            text_input("password:", &self.password)
                .id(text_input::Id::new("password"))
                .secure(true)
                .on_submit(Msg::PasswordSubmit.into())
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

    fn quit(&mut self) -> Action {
        std::process::exit(0)
    }
}

impl LoginPage {
    pub fn new(username: String, password: String) -> Self {
        Self { username, password }
    }
}
