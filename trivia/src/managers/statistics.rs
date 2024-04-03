use std::sync::{Arc, Mutex};

use crate::db::Database;


pub struct StatisticsManager {
    db: Arc<Mutex<dyn Database>>
}

impl StatisticsManager {
    pub fn new(db: Arc<Mutex<dyn Database>>) -> Self {
        Self { db }
    }

    pub fn get_high_scores(&self) -> Vec<String> {
        todo!()
    }

    pub fn get_user_statistics(&self, username: impl AsRef<str>) -> Vec<String> {
        let username = username.as_ref();
        todo!()
    }
}
