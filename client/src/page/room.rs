use std::time::Duration;

use iced::alignment::Horizontal;
use iced::widget::scrollable::Properties;
use iced::widget::{
    button, column, container, horizontal_space, row, scrollable, text, vertical_space, Column,
};
use iced::{theme, Alignment, Length};
use trivia::managers::login::LoggedUser;
use trivia::messages::{Request, Response};

use crate::action::Action;
use crate::consts;
use crate::message::Message;

use super::{MainMenuPage, Page};

// NOTE: my temporary solution is to consider the first user in the users list as the admin, not
// sure how great of a solution that is but ig it will work

#[derive(Debug, Clone)]
pub enum Msg {
    UpdatePlayers,
    StartGame,
    CloseRoom,
    LeaveRoom,
}

pub struct RoomPage {
    room_name: String,
    players: Vec<LoggedUser>,
    is_admin: bool, // true when the current user is the admin
}

impl Page for RoomPage {
    fn update(&mut self, message: Message) -> Action {
        if let Message::Response(response) = message {
            match response.as_ref() {
                Response::RoomState {
                    state,
                    name,
                    players,
                    question_count,
                    time_per_question,
                } => {
                    self.room_name = name.clone();
                    self.players = players.clone()
                },

                Response::StartGame => todo!("switch to the StartGame page"),

                _ => eprintln!("response ignored: {:?}", response),
            }

            return Action::none();
        }

        let Message::Room(msg) = message else {
            return Action::none();
        };

        match msg {
            Msg::UpdatePlayers => Action::request(Request::RoomState),
            Msg::StartGame => Action::request(Request::StartGame), // TODO: switch to the game page
            Msg::CloseRoom => Action::switch_and_request(MainMenuPage, Request::CloseRoom),
            Msg::LeaveRoom => Action::switch_and_request(MainMenuPage, Request::LeaveRoom),
        }
    }

    fn view(&self) -> iced::Element<Message> {
        let title = text(format!("Room {}", self.room_name))
            .size(consts::TITLE_SIZE)
            .width(Length::Fill)
            .horizontal_alignment(Horizontal::Center);

        let players_col = Column::from_vec(self.players.iter().map(player_element).collect())
            .align_items(Alignment::Center)
            .padding(2)
            .spacing(20)
            .width(Length::Fill);

        let buttons = if self.is_admin {
            column![
                vertical_space(),
                button(text("Start Game").size(30)).on_press(Msg::StartGame.into()),
                button(text("Close Room").size(30)).on_press(Msg::CloseRoom.into()),
            ]
        } else {
            column![
                vertical_space(),
                button(text("Leave Room").size(30)).on_press(Msg::LeaveRoom.into()),
            ]
        }
        .width(Length::Fill)
        .height(Length::Fill)
        .align_items(Alignment::Center)
        .padding(10)
        .spacing(20);

        let rooms = container(row![
            horizontal_space().width(Length::FillPortion(1)),
            scrollable(players_col)
                .direction(scrollable::Direction::Vertical(
                    Properties::new().width(3).scroller_width(1.5),
                ))
                .width(Length::FillPortion(3))
                .height(Length::Fill),
            horizontal_space().width(Length::FillPortion(1)),
        ]);

        container(column![
            container(title)
                .height(Length::FillPortion(1))
                .padding(consts::TITLES_PADDING),
            row![rooms, buttons].height(Length::FillPortion(4))
        ])
        .height(Length::Fill)
        .width(Length::Fill)
        .center_x()
        .center_y()
        .into()
    }

    fn subscription(&self) -> iced::Subscription<Message> {
        iced::time::every(Duration::from_secs(3)).map(|_| Msg::UpdatePlayers.into())
    }
}

impl RoomPage {
    pub fn new(is_admin: bool) -> Self {
        Self {
            room_name: String::new(),
            players: vec![],
            is_admin,
        }
    }
}

fn player_element(user: &LoggedUser) -> iced::Element<Message> {
    let user = text(user.username()).size(30);

    container(column![user].align_items(Alignment::Center))
        .style(theme::Container::Box)
        .height(40)
        .width(200)
        .center_x()
        .into()
}
