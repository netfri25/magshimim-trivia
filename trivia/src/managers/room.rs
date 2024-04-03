use std::{collections::HashMap, time::Duration};

use super::login::LoggedUser;

pub type RoomID = i32;

#[derive(Default)]
pub struct RoomManager {
    rooms: HashMap<RoomID, Room>,
}

impl RoomManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_room(&mut self, user: LoggedUser, data: RoomData) {
        let mut room = Room::new(data);
        room.add_user(user);
        self.rooms.insert(room.data.room_id, room);
    }

    pub fn delete_room(&mut self, id: RoomID) {
        self.rooms.remove(&id);
    }

    pub fn is_room_active(&self, id: RoomID) -> bool {
        self.rooms.get(&id).map(|room| room.data.active).unwrap_or(false)
    }

    pub fn room(&self, id: RoomID) -> Option<&Room> {
        self.rooms.get(&id)
    }

    pub fn room_mut(&mut self, id: RoomID) -> Option<&mut Room> {
        self.rooms.get_mut(&id)
    }
}

#[derive(Default)]
pub struct Room {
    data: RoomData,
    users: Vec<LoggedUser>,
}

impl Room {
    pub fn new(data: RoomData) -> Self {
        let users = Vec::new();
        Self { data, users }
    }

    pub fn add_user(&mut self, user: LoggedUser) {
        self.users.push(user)
    }

    pub fn remove_user(&mut self, user: &LoggedUser) {
        let Some(index) = self.users.iter().position(|u| u == user) else {
            return;
        };

        self.users.swap_remove(index);
    }

    pub fn users(&self) -> &[LoggedUser] {
        &self.users
    }
}

#[derive(Default)]
pub struct RoomData {
    pub room_id: i32,
    pub name: String,
    pub max_players: usize,
    pub questions_count: usize,
    pub time_per_quersion: Duration,
    pub active: bool
}
