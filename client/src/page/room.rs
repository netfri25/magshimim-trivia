use std::time::Duration;

use iced::alignment::Horizontal;
use iced::widget::scrollable::Properties;
use iced::widget::{
    button, column, container, horizontal_space, row, scrollable, text, vertical_space, Column,
};
use iced::{theme, Alignment, Length};
use trivia::messages::{Request, Response};
use trivia::username::Username;

use crate::action::Action;
use crate::consts;
use crate::message::Message;

use super::{GamePage, MainMenuPage, Page};

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
    players: Vec<Username>,
    time_per_question: Duration,
    is_admin: bool, // true when the current user is the admin
    question_count: usize,
}

impl Page for RoomPage {
    fn update(&mut self, message: Message) -> Result<Action, String> {
        if let Message::Response(response) = message {
            match response.as_ref() {
                Response::RoomState {
                    name,
                    players,
                    time_per_question,
                    question_count,
                    ..
                } => {
                    self.room_name.clone_from(name);
                    self.players.clone_from(players);
                    self.time_per_question = *time_per_question;
                    self.question_count = *question_count;
                }

                Response::StartGame(res) => {
                    return if let Err(err) = res {
                        Err(err.to_string())
                    } else {
                        Ok(Action::switch_and_request(
                            GamePage::new(self.time_per_question, self.question_count),
                            Request::Question,
                        ))
                    }
                }

                Response::LeaveRoom => return Ok(Action::switch(MainMenuPage)),

                _ => eprintln!("response ignored: {:?}", response),
            }

            return Ok(Action::none());
        }

        let Message::Room(msg) = message else {
            return Ok(Action::none());
        };

        Ok(match msg {
            Msg::UpdatePlayers => Action::request(Request::RoomState),
            Msg::StartGame => Action::request(Request::StartGame),
            Msg::CloseRoom => Action::switch_and_request(MainMenuPage, Request::CloseRoom),
            Msg::LeaveRoom => Action::switch_and_request(MainMenuPage, Request::LeaveRoom),
        })
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

    fn quit(&mut self) -> Action {
        let req = if self.is_admin {
            Request::CloseRoom
        } else {
            Request::LeaveRoom
        };
        Action::switch_and_request(MainMenuPage, req)
    }
}

impl RoomPage {
    pub fn new(is_admin: bool) -> Self {
        Self {
            room_name: String::new(),
            players: vec![],
            time_per_question: Default::default(),
            is_admin,
            question_count: 0,
        }
    }
}

fn player_element(user: &Username) -> iced::Element<Message> {
    let user = text(user.as_ref()).size(30);

    container(column![user].align_items(Alignment::Center))
        .style(theme::Container::Box)
        .height(40)
        .width(200)
        .center_x()
        .into()
}
