use std::time::Duration;

use iced::alignment::Horizontal;
use iced::widget::scrollable::Properties;
use iced::widget::{
    button, column, container, horizontal_space, row, scrollable, text, tooltip, Column,
};
use iced::{theme, Alignment, Length};

use crate::action::Action;
use crate::consts;
use crate::message::Message;

use trivia::managers::room::{Room, RoomData, RoomID, RoomState};
use trivia::messages::{Request, Response};

use super::room::RoomPage;
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
                    self.rooms = rooms
                        .iter()
                        .filter(|r| r.room_data().state == RoomState::Waiting)
                        .cloned()
                        .collect()
                }

                &Response::JoinRoom => {
                    let page = RoomPage::new(false);
                    let req = Request::RoomState;
                    return Action::switch_and_request(page, req);
                }

                _ => eprintln!("response ignored: {:?}", response),
            }

            return Action::none();
        }

        let Message::JoinRoom(msg) = message else {
            return Action::none();
        };

        match msg {
            Msg::UpdateRooms => Action::request(Request::RoomList),
            Msg::EnterRoom(id) => Action::request(Request::JoinRoom(id)),
        }
    }

    fn view(&self) -> iced::Element<Message> {
        let title = text("Join Room")
            .size(consts::TITLE_SIZE)
            .width(Length::Fill)
            .horizontal_alignment(Horizontal::Center);

        let rooms_col = Column::from_vec(self.rooms.iter().map(room_element).collect())
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
#[allow(unused_variables)]
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
    let admin = users.first().map(|u| u.as_ref()).unwrap_or_default();

    let name = text(name).size(20);
    let players = text(format!("players: {}/{}", users.len(), max_players)).size(10);

    let room = column![name, players].align_items(Alignment::Center);

    let room_container = container(room)
        .style(iced::theme::Container::Custom(Box::new(style::RoomStyle)))
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y();

    let room = button(room_container)
        .style(iced::theme::Button::Text)
        .on_press(Msg::EnterRoom(*room_id).into())
        .height(100)
        .padding(5);

    let users = Column::from_vec(
        users
            .iter()
            .map(|u| text(u.as_ref()).size(10).into())
            .collect(),
    );

    tooltip(room, users, tooltip::Position::FollowCursor)
        .style(theme::Container::Box)
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
