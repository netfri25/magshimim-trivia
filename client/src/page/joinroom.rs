use std::time::Duration;

use iced::alignment::Horizontal;
use iced::widget::scrollable::Properties;
use iced::widget::{button, column, container, horizontal_space, row, scrollable, text, Column};
use iced::{Alignment, Length};

use crate::action::Action;
use crate::consts;
use crate::message::Message;

use trivia::managers::room::{Room, RoomData, RoomID};
use trivia::messages::{Request, Response};

use super::Page;

#[derive(Debug, Clone)]
pub enum Msg {
    EnterRoom(RoomID),
    UpdateRooms,
}

#[derive(Default)]
pub struct JoinRoomPage {
    rooms: Vec<Room>,
}

impl Page for JoinRoomPage {
    fn update(&mut self, message: Message) -> Action {
        if let Message::Response(response) = message {
            match response.as_ref() {
                Response::RoomList(rooms) => {
                    println!("rooms have been set!");
                    self.rooms = rooms.clone();
                }

                _ => eprintln!("response ignored: {:?}", response),
            }

            return Action::none();
        }

        let Message::JoinRoom(msg) = message else {
            return Action::none();
        };

        // TODO: enter a room
        match msg {
            Msg::EnterRoom(id) => eprintln!("enter room {:?}", id),
            Msg::UpdateRooms => return Action::request(Request::RoomList),
        }

        Action::none()
    }

    fn view(&self) -> iced::Element<Message> {
        let title = text("Join Room")
            .size(consts::TITLE_SIZE)
            .width(Length::Fill)
            .horizontal_alignment(Horizontal::Center);

        let rooms_col = Column::from_vec(
            self.rooms
                .iter()
                .map(room_element)
                .collect(),
        )
        .align_items(Alignment::Center)
        .padding(2)
        .width(Length::Fill);

        let rooms = container(row![
            horizontal_space().width(Length::FillPortion(1)),
            scrollable(rooms_col)
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
        iced::time::every(Duration::from_secs(3)).map(|_| Msg::UpdateRooms.into())
    }
}

impl JoinRoomPage {
    pub fn new() -> (Self, Request) {
        (Self::default(), Request::RoomList)
    }
}

// TODO: add more info for each room
pub fn room_element<'a>(room: &Room) -> iced::Element<'a, Message, iced::Theme> {
    let RoomData {
        room_id,
        name,
        max_players,
        questions_count,
        time_per_question,
        state,
    } = room.room_data();

    // the first user is the admin
    let users = room.users();
    let admin = users.first().map(|u| u.username()).unwrap_or_default();

    let name = text(name).size(20);
    let players = text(format!("players: {}/{}", users.len(), max_players)).size(10);

    let room = column![name, players].align_items(Alignment::Center);

    let room_container = container(room)
        .style(iced::theme::Container::Custom(Box::new(style::RoomStyle)))
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y();

    button(room_container)
        .style(iced::theme::Button::Text)
        .on_press(Msg::EnterRoom(*room_id).into())
        .height(100)
        .padding(5)
        .into()
}

mod style {
    use iced::widget::container;

    pub struct RoomStyle;

    impl container::StyleSheet for RoomStyle {
        type Style = iced::Theme;

        fn appearance(&self, style: &Self::Style) -> container::Appearance {
            container::Appearance {
                border: iced::Border {
                    color: style.extended_palette().secondary.weak.color,
                    width: 2.,
                    radius: 5.into(),
                },
                ..Default::default()
            }
        }
    }
}
