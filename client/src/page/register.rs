use iced::alignment::Horizontal;
use iced::widget::{button, column, container, horizontal_space, row, text, text_input};
use iced::{theme, Alignment, Length};
use iced_aw::date_picker;
use iced_aw::date_picker::Date;

use trivia::messages::{Address, Request, Response};
use trivia::NaiveDate;

use crate::message::Message;
use crate::{action::Action, consts};

use super::{LoginPage, Page};

#[derive(Debug, Clone)]
pub enum Msg {
    UsernameInput(String),
    UsernameSubmit,
    PasswordInput(String),
    PasswordSubmit,
    EmailInput(String),
    EmailSubmit,
    PhonePrefixInput(String),
    PhonePrefixSubmit,
    PhoneNumberInput(String),
    PhoneNumberSubmit,
    AddressCityInput(String),
    AddressCitySubmit,
    AddressStreetInput(String),
    AddressStreetSubmit,
    AddressApartmentInput(String),
    AddressApartmentSubmit,
    PickDate(Date),
    OpenPicker,
    ClosePicker,
    Register,
    Login,
}

pub struct RegisterPage {
    username: String,
    password: String,
    email: String,
    phone_prefix: String,
    phone_number: String,
    address_city: String,
    address_street: String,
    address_apartment: Option<u64>,
    birth_date: Date,
    choosing_date: bool,
}

impl Page for RegisterPage {
    fn update(&mut self, message: Message) -> Action {
        if let Message::Response(response) = message {
            match response.as_ref() {
                Response::Signup => {
                    return Action::switch(LoginPage::new(
                        self.username.clone(),
                        self.password.clone(),
                    ))
                }

                _ => eprintln!("response ignored: {:?}", response),
            }

            return Action::none();
        };

        let Message::Register(msg) = message else {
            return Action::none();
        };

        match msg {
            Msg::UsernameInput(username) => self.username = username,
            Msg::UsernameSubmit => {
                return Action::cmd(text_input::focus(text_input::Id::new("password")))
            }

            Msg::PasswordInput(password) => self.password = password,
            Msg::PasswordSubmit => {
                return Action::cmd(text_input::focus(text_input::Id::new("email")))
            }

            Msg::EmailInput(email) => self.email = email,
            Msg::EmailSubmit => {
                return Action::cmd(text_input::focus(text_input::Id::new("phone-prefix")))
            }

            Msg::PhonePrefixInput(phone_prefix) => self.phone_prefix = phone_prefix,
            Msg::PhonePrefixSubmit => {
                return Action::cmd(text_input::focus(text_input::Id::new("phone-number")))
            }

            Msg::PhoneNumberInput(phone_number) => self.phone_number = phone_number,
            Msg::PhoneNumberSubmit => {
                return Action::cmd(text_input::focus(text_input::Id::new("address-city")))
            }

            Msg::AddressCityInput(address_city) => self.address_city = address_city,
            Msg::AddressCitySubmit => {
                return Action::cmd(text_input::focus(text_input::Id::new("address-street")))
            }

            Msg::AddressStreetInput(address_street) => self.address_street = address_street,
            Msg::AddressStreetSubmit => {
                return Action::cmd(text_input::focus(text_input::Id::new("address-apartment")))
            }

            Msg::AddressApartmentInput(address_apartment) => {
                if address_apartment.is_empty() {
                    self.address_apartment = None;
                } else {
                    self.address_apartment = Some(address_apartment.parse().unwrap_or_default())
                }
            }
            Msg::AddressApartmentSubmit | Msg::Register => {
                return Action::request(Request::Signup {
                    username: self.username.clone(),
                    password: self.password.clone(),
                    email: self.email.clone(),
                    phone: format!("{}-{}", self.phone_prefix, self.phone_number),
                    address: Address::new(
                        self.address_city.clone(),
                        self.address_street.clone(),
                        self.address_apartment.unwrap_or_default(),
                    ),
                    birth_date: NaiveDate::from_ymd_opt(
                        self.birth_date.year,
                        self.birth_date.month,
                        self.birth_date.day,
                    )
                    .expect("date from date-picker is always valid"),
                });
            }

            Msg::PickDate(date) => {
                self.birth_date = date;
                self.choosing_date = false;
            }

            Msg::OpenPicker => self.choosing_date = true,
            Msg::ClosePicker => self.choosing_date = false,

            Msg::Login => return Action::switch(LoginPage::default()),
        }

        Action::none()
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
        ]
        .align_items(Alignment::Center);

        let birth_date_button = button(text(format!("Birth date: {}", self.birth_date)))
            .on_press(Msg::OpenPicker.into());
        let birth_date_picker = date_picker(
            self.choosing_date,
            self.birth_date,
            birth_date_button,
            Msg::ClosePicker.into(),
            |date| Msg::PickDate(date).into(),
        );

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
            text_input("email:", &self.email)
                .id(text_input::Id::new("email"))
                .on_submit(Msg::EmailSubmit.into())
                .on_input(|input| Msg::EmailInput(input).into()),
            row![
                text_input("phone prefix:", &self.phone_prefix)
                    .id(text_input::Id::new("phone-prefix"))
                    .on_submit(Msg::PhonePrefixSubmit.into())
                    .on_input(|input| Msg::PhonePrefixInput(input).into())
                    .width(Length::FillPortion(3)),
                text_input("phone number:", &self.phone_number)
                    .id(text_input::Id::new("phone-number"))
                    .on_submit(Msg::PhoneNumberSubmit.into())
                    .on_input(|input| Msg::PhoneNumberInput(input).into())
                    .width(Length::FillPortion(7)),
            ]
            .spacing(consts::INPUT_FIELDS_PADDING)
            .width(Length::Fill),
            row![
                text_input("city:", &self.address_city)
                    .id(text_input::Id::new("address-city"))
                    .on_submit(Msg::AddressCitySubmit.into())
                    .on_input(|input| Msg::AddressCityInput(input).into())
                    .width(Length::FillPortion(4)),
                text_input("street:", &self.address_street)
                    .id(text_input::Id::new("address-street"))
                    .on_submit(Msg::AddressStreetSubmit.into())
                    .on_input(|input| Msg::AddressStreetInput(input).into())
                    .width(Length::FillPortion(6)),
                text_input(
                    "apt:",
                    &self
                        .address_apartment
                        .map(|v| v.to_string())
                        .unwrap_or_default()
                )
                .id(text_input::Id::new("address-apartment"))
                .on_submit(Msg::AddressApartmentSubmit.into())
                .on_input(|input| Msg::AddressApartmentInput(
                    input.chars().filter(|c| c.is_ascii_digit()).collect()
                )
                .into())
                .width(Length::FillPortion(1)),
            ]
            .spacing(consts::INPUT_FIELDS_PADDING)
            .width(Length::Fill),
            birth_date_picker,
            container(buttons)
                .padding(consts::BUTTONS_PADDING)
                .center_y(),
        ]
        .spacing(consts::INPUT_FIELDS_SPACING)
        .padding(consts::INPUT_FIELDS_PADDING);

        let body = column![
            container(
                column![title, subtitle]
                    .padding(consts::TITLES_PADDING)
                    .spacing(consts::TITLES_SPACING)
            )
            .height(consts::TITLES_PORTION)
            .center_y(),
            container(
                row![
                    horizontal_space().width(Length::FillPortion(2)),
                    input_fields.width(Length::FillPortion(7)),
                    horizontal_space().width(Length::FillPortion(2)),
                ]
                .width(Length::Fill)
            )
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

impl Default for RegisterPage {
    fn default() -> Self {
        Self {
            username: String::new(),
            password: String::new(),
            email: String::new(),
            phone_prefix: String::new(),
            phone_number: String::new(),
            address_city: String::new(),
            address_street: String::new(),
            address_apartment: None,
            birth_date: Date::today(),
            choosing_date: false,
        }
    }
}
