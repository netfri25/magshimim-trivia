use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::db::Database;

use crate::managers::game::Score;

pub struct StatisticsManager<'db> {
    db: &'db (dyn Database + Sync),
}

impl<'db> StatisticsManager<'db> {
    pub fn new(db: &'db (dyn Database + Sync)) -> Self {
        Self { db }
    }

    pub fn get_high_scores(&self) -> Result<[Option<(String, Score)>; 5], crate::db::Error> {
        self.db.get_five_highscores()
    }

    pub fn get_user_statistics(&self, username: &str) -> Result<Statistics, crate::db::Error> {
        let correct_answers = self.db.get_correct_answers_count(username)?;
        let total_answers = self.db.get_total_answers_count(username)?;
        let average_answer_time = self.db.get_player_average_answer_time(username)?;
        let total_games = self.db.get_games_count(username)?;
        let score = self.db.get_score(username)?;

        Ok(Statistics {
            correct_answers,
            total_answers,
            average_answer_time,
            total_games,
            score,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Statistics {
    pub correct_answers: i64,
    pub total_answers: i64,
    pub average_answer_time: Duration,
    pub total_games: i64,
    pub score: Score,
}
