use std::time::Duration;
use std::sync::{Arc, Mutex};

use crate::db::{Database, Score};


pub struct StatisticsManager {
    db: Arc<Mutex<dyn Database>>
}

impl StatisticsManager {
    pub fn new(db: Arc<Mutex<dyn Database>>) -> Self {
        Self { db }
    }

    pub fn get_high_scores(&self) -> anyhow::Result<[Score; 5]> {
        self.db.lock().unwrap().get_five_highscores()
    }

    pub fn get_user_statistics(&self, username: &str) -> anyhow::Result<Statistics> {
        let correct_answers = self.db.lock().unwrap().get_correct_answers_count(username)?;
        let total_answers = self.db.lock().unwrap().get_total_answers_count(username)?;
        let average_answer_time = self.db.lock().unwrap().get_player_average_answer_time(username)?;
        let total_games = self.db.lock().unwrap().get_games_count(username)?;
        let score = self.db.lock().unwrap().get_score(username)?;

        Ok(Statistics {
            correct_answers,
            total_answers,
            average_answer_time,
            total_games,
            score,
        })
    }
}

pub struct Statistics {
    pub correct_answers: i64,
    pub total_answers: i64,
    pub average_answer_time: Duration,
    pub total_games: i64,
    pub score: Score,
}
