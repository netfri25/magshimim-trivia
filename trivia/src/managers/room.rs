use std::time::Duration;
use std::collections::HashMap;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

use super::login::LoggedUser;

pub type RoomID = i64;

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

    pub fn delete_room(&mut self, id: RoomID) -> Option<Room> {
        self.rooms.remove(&id)
    }

    pub fn room_state(&self, id: RoomID) -> Option<RoomState> {
        self.rooms.get(&id).map(|room| room.room_data().state)
    }

    pub fn set_state(&mut self, id: RoomID, state: RoomState) -> bool {
        self.rooms.get_mut(&id).map(|room| room.data.state = state).is_some()
    }

    pub fn room(&self, id: RoomID) -> Option<&Room> {
        self.rooms.get(&id)
    }

    pub fn room_mut(&mut self, id: RoomID) -> Option<&mut Room> {
        self.rooms.get_mut(&id)
    }

    pub fn rooms(&self) -> impl Iterator<Item = &Room> {
        self.rooms.values()
    }
}

#[derive(Default, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
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

    pub fn room_data(&self) -> &RoomData {
        &self.data
    }

    pub fn is_full(&self) -> bool {
        self.users().len() >= self.room_data().max_players
    }

    pub fn is_empty(&self) -> bool {
        self.users().is_empty()
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum RoomState {
    #[default]
    Waiting,
    InGame,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct RoomData {
    pub room_id: RoomID,
    pub name: String,
    pub max_players: usize,
    pub questions_count: usize,
    pub time_per_question: Duration,
    pub state: RoomState
}

impl RoomData {
    pub fn new(
        name: impl Into<String>,
        max_players: usize,
        questions_count: usize,
        time_per_question: Duration
    ) -> Self {
        static ROOM_ID_COUNTER: Mutex<RoomID> = Mutex::new(0);
        let room_id;
        {
            let mut id_lock = ROOM_ID_COUNTER.lock().unwrap();
            room_id = *id_lock;
            *id_lock += 1;
        }

        let name = name.into();

        Self {
            room_id,
            name,
            max_players,
            questions_count,
            time_per_question,
            state: RoomState::default(),
        }
    }
}
