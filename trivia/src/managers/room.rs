use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::username::Username;

pub type RoomID = usize;

#[derive(Default)]
pub struct RoomManager {
    rooms: HashMap<RoomID, Room>,
}

impl RoomManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_room(&mut self, user: Username, data: RoomData) {
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
        self.rooms
            .get_mut(&id)
            .map(|room| room.data.state = state)
            .is_some()
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
    users: Vec<Username>,
}

impl Room {
    pub fn new(data: RoomData) -> Self {
        let users = Vec::new();
        Self { data, users }
    }

    pub fn add_user(&mut self, user: Username) -> bool {
        if self.users.contains(&user) {
            false
        } else {
            self.users.push(user);
            true
        }
    }

    pub fn remove_user(&mut self, user: &Username) -> bool {
        let Some(index) = self.users.iter().position(|u| u == user) else {
            return false;
        };

        self.users.swap_remove(index);
        true
    }

    pub fn users(&self) -> &[Username] {
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
    pub state: RoomState,
}

impl RoomData {
    pub fn new(
        name: impl Into<String>,
        max_players: usize,
        questions_count: usize,
        time_per_question: Duration,
    ) -> Self {
        static ROOM_ID_COUNTER: AtomicUsize = AtomicUsize::new(0);
        let room_id = ROOM_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
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
