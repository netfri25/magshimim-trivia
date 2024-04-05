use iced::{
    alignment::Horizontal,
    widget::{button, column, container, horizontal_rule, row, text, text_input},
    Color, Length,
};

use crate::action::Action;
use crate::message::Message;

#[derive(Debug, Clone)]
pub enum Msg {
    UsernameInput(String),
    PasswordInput(String),
    Login,
    Register,
}

#[derive(Default)]
pub struct Page {
    username: String,
    password: String,
}

impl super::Page for Page {
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
            Msg::Register => todo!(), // TODO: switch to the Register screen
        }

        Action::Nothing
    }

    fn view(&self) -> iced::Element<Message> {
        let title = text("Login")
            .size(70)
            .width(Length::Fill)
            .horizontal_alignment(Horizontal::Center);
        let subtitle = text("Login with your Trivia Account")
            .size(17.)
            .width(Length::Fill)
            .horizontal_alignment(Horizontal::Center)
            .style(Color::new(0.7, 0.7, 0.7, 0.7));

        // TODO: register button
        let login_button = button("Login").on_press(Msg::Login.into());

        let input_fields = column![
            text_input("username:", &self.username)
                .on_input(|input| Msg::UsernameInput(input).into()),
            text_input("password:", &self.password)
                .secure(true)
                .on_input(|input| Msg::PasswordInput(input).into()),
            container(login_button).padding(2.),
        ]
        .padding(10.)
        .spacing(10.)
        .max_width(300.);

        let body = column![
            container(column![title, subtitle,].padding(10.).spacing(10.))
                .height(Length::FillPortion(1))
                .center_y(),
            horizontal_rule(1),
            container(input_fields)
                .width(Length::Fill)
                .height(Length::FillPortion(2))
                .center_x()
                .center_y(),
            horizontal_rule(1),
        ];

        container(body)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}
