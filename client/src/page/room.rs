use std::time::Duration;

use iced::alignment::Horizontal;
use iced::widget::scrollable::Properties;
use iced::widget::{column, container, horizontal_space, row, scrollable, text, Column};
use iced::{theme, Alignment, Color, Length};
use trivia::managers::login::LoggedUser;
use trivia::managers::room::RoomID;
use trivia::messages::{Request, Response};

use crate::action::Action;
use crate::consts;
use crate::message::Message;

use super::Page;

// NOTE: my temporary solution is to consider the first user in the users list as the admin, not
// sure how great of a solution that is but ig it will work

#[derive(Debug, Clone)]
pub enum Msg {
    UpdatePlayers,
}

pub struct RoomPage {
    id: RoomID,
    players: Vec<LoggedUser>,
    is_admin: bool, // true when the current user is the admin
}

impl Page for RoomPage {
    fn update(&mut self, message: Message) -> Action {
        if let Message::Response(response) = message {
            match response.as_ref() {
                Response::PlayersInRoom(players) => {
                    self.players = players.clone();
                    eprintln!("players in room {}: {:?}", self.id, self.players);
                }

                _ => eprintln!("response ignored: {:?}", response),
            }

            return Action::none();
        }

        let Message::Room(msg) = message else {
            return Action::none();
        };

        match msg {
            Msg::UpdatePlayers => Action::request(Request::PlayersInRoom(self.id)),
        }
    }

    fn view(&self) -> iced::Element<Message> {
        let title = text(format!("Room {}", self.id))
            .size(consts::TITLE_SIZE)
            .width(Length::Fill)
            .horizontal_alignment(Horizontal::Center);

        let players_col = Column::from_vec(self.players.iter().map(player_element).collect())
            .align_items(Alignment::Center)
            .padding(2)
            .spacing(20)
            .width(Length::Fill);

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
            rooms.height(Length::FillPortion(4))
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
    pub fn new(id: RoomID, is_admin: bool) -> Self {
        Self {
            id,
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
