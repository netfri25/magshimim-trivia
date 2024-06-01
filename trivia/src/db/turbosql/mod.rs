use std::path::Path;
use std::str::FromStr;
use std::time::Duration;

use chrono::prelude::NaiveDate;
use turbosql::{execute, select, Turbosql};

use crate::email::Email;
use crate::managers::game::{calc_score, GameData, Score};
use crate::managers::statistics::Highscores;
use crate::messages::{Address, PhoneNumber, DATE_FORMAT};
use crate::password::Password;
use crate::username::Username;

use super::{Database, Error, QuestionData};

mod models;
use models::*;

pub struct TurboSqliteDatabase {}

impl TurboSqliteDatabase {
    pub fn connect(path: impl AsRef<Path>) -> Result<Self, Error> {
        turbosql::set_db_path(path.as_ref())?;
        Ok(Self {})
    }
}

impl Database for TurboSqliteDatabase {
    fn user_exists(&self, username: &Username) -> Result<bool, Error> {
        Ok(select!(Option<i64> "rowid FROM user WHERE username = ?", username.as_ref())?.is_some())
    }

    fn password_matches(&self, username: &Username, password: &Password) -> Result<bool, Error> {
        Ok(select!(Option<i64> "rowid FROM user WHERE username = ? AND password = ?", username.as_ref(), password.as_ref())?.is_some())
    }

    fn add_user(
        &self,
        username: Username,
        password: Password,
        email: Email,
        phone: PhoneNumber,
        address: Address,
        birth_date: NaiveDate,
    ) -> Result<(), Error> {
        let user = User {
            username: username.to_string(),
            password: password.to_string(),
            email: email.to_string(),
            phone: phone.to_string(),
            city: address.city().to_string(),
            street: address.street().to_string(),
            apartment: address.apartment(),
            birth_date: birth_date.format(DATE_FORMAT).to_string(),
            ..Default::default()
        };
        user.insert()?;
        turbosql::checkpoint()?;
        Ok(())
    }

    fn get_questions(&self, amount: usize) -> Result<Vec<QuestionData>, Error> {
        let questions = select!(Vec<Question> "ORDER BY RANDOM() LIMIT " amount)?;
        questions
            .into_iter()
            .map(|question| {
                let answers: Vec<Answer> =
                    select!(Vec<Answer> "WHERE question_id = " question.rowid)?;
                let Some(correct_index) = answers.iter().position(|a| a.correct) else {
                    return Err(Error::NoCorrectAnswer {
                        question_id: question.rowid.unwrap_or(-1),
                        question_content: question.content,
                    });
                };

                let answers = answers.into_iter().map(|a| a.content).collect();
                Ok(QuestionData::new(question.content, answers, correct_index))
            })
            .collect()
    }

    fn get_player_average_answer_time(&self, username: &Username) -> Result<Duration, Error> {
        let secs = select!(f64 "average_answer_time FROM statistics INNER JOIN user ON user.rowid = statistics.rowid WHERE user.username = ?", username.as_ref())?;
        Ok(Duration::from_secs_f64(secs))
    }

    fn get_correct_answers_count(&self, username: &Username) -> Result<i64, Error> {
        let res = select!(i64 "correct_answers FROM statistics INNER JOIN user ON user.rowid = statistics.rowid WHERE user.username = ?", username.as_ref())?;
        Ok(res)
    }

    fn get_total_answers_count(&self, username: &Username) -> Result<i64, Error> {
        let res = select!(i64 "total_answers FROM statistics INNER JOIN user ON user.rowid = statistics.rowid WHERE user.username = ?", username.as_ref())?;
        Ok(res)
    }

    fn get_games_count(&self, username: &Username) -> Result<i64, Error> {
        let res = select!(i64 "total_games FROM statistics INNER JOIN user ON user.rowid = statistics.rowid WHERE user.username = ?", username.as_ref())?;
        Ok(res)
    }

    fn get_score(&self, username: &Username) -> Result<Score, Error> {
        let res = select!(f64 "overall_score FROM statistics INNER JOIN user ON user.rowid = statistics.rowid WHERE user.username = ?", username.as_ref())?;
        Ok(res)
    }

    fn get_five_highscores(&self) -> Result<Highscores, Error> {
        let res = select!(Vec<HighscoreResult> "SELECT user.username, statistics.overall_score FROM statistics INNER JOIN user ON user.rowid = statistics.rowid ORDER BY statistics.overall_score DESC LIMIT 5")?;
        let res = res
            .into_iter()
            .map(|res| Ok((res.username.parse::<Username>()?, res.overall_score)))
            .collect::<Result<Highscores, <Username as FromStr>::Err>>()?;
        Ok(res)
    }

    fn submit_game_data(&self, username: &Username, game_data: GameData) -> Result<(), Error> {
        let GameData {
            correct_answers: current_correct_answers,
            wrong_answers: current_wrong_answers,
            avg_time: current_avg_time,
            ..
        } = game_data;

        let Some(user_id) =
            select!(Option<i64> "rowid FROM user WHERE username = ?", username.as_ref())?
        else {
            return Err(Error::UserDoesntExist(username.clone()));
        };

        let old_total_answers = self.get_total_answers_count(username).unwrap_or_default();
        let total_answers =
            old_total_answers + current_wrong_answers as i64 + current_correct_answers as i64;
        let avg_time = {
            let old_total_time = self
                .get_player_average_answer_time(username)
                .unwrap_or_default()
                .as_secs_f64()
                * old_total_answers as f64;
            let current_total_time = current_avg_time.as_secs_f64()
                * (current_wrong_answers + current_correct_answers) as f64;
            (old_total_time + current_total_time) / total_answers as f64
        };
        let correct_answers = self.get_correct_answers_count(username).unwrap_or_default()
            + current_correct_answers as i64;
        let total_games = self.get_games_count(username).unwrap_or_default() + 1;
        let overall_score = calc_score(Duration::from_secs_f64(avg_time), correct_answers);

        execute!(
            "REPLACE INTO statistics VALUES (?, ?, ?, ?, ?, ?)",
            user_id,
            correct_answers,
            total_answers,
            avg_time,
            total_games,
            overall_score
        )?;

        Ok(())
    }

    fn add_question(&self, data: &QuestionData) -> Result<bool, Error> {
        if select!(Option<i64> "rowid FROM question WHERE content = ?", data.question)?.is_some() {
            return Ok(false);
        }

        let question_id = Question {
            content: data.question.to_string(),
            ..Default::default()
        }
        .insert()?;

        for (i, answer) in data.answers.iter().enumerate() {
            Answer {
                question_id,
                content: answer.to_string(),
                correct: i == data.correct_answer_index,
                ..Default::default()
            }
            .insert()?;
        }
        turbosql::checkpoint()?;

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use crate::managers::game::{calc_score, GameData};
    use crate::messages::DATE_FORMAT;

    use super::*;

    #[test]
    fn signup() {
        let path = Path::new("test-db.sqlite");
        if path.exists() {
            std::fs::remove_file(path).ok();
            std::fs::remove_file(path.with_extension("sqlite-shm")).ok();
            std::fs::remove_file(path.with_extension("sqlite-wal")).ok();
        }
        let db = TurboSqliteDatabase::connect(path).unwrap();
        assert!(!db.user_exists(&"me".parse().unwrap()).unwrap());
        db.add_user(
            "me".parse().unwrap(),
            "Pass@123".parse().unwrap(),
            "main@example.com".parse().unwrap(),
            "052-1122333".parse().unwrap(),
            Address::new("Netanya", "Alonim", 69),
            NaiveDate::parse_from_str("22/04/2038", DATE_FORMAT).unwrap(),
        )
        .unwrap();
        assert!(db.user_exists(&"me".parse().unwrap()).unwrap());
        assert!(db
            .password_matches(&"me".parse().unwrap(), &"Pass@123".parse().unwrap())
            .unwrap());
        assert!(!db
            .password_matches(&"me".parse().unwrap(), &"NotPass@123".parse().unwrap())
            .unwrap());
    }

    #[test]
    #[ignore = "prevent API spamming"]
    fn populate_questions() -> anyhow::Result<()> {
        let db = TurboSqliteDatabase {};
        // db.populate_questions(100)?;
        db.get_questions(10)?;
        Ok(())
    }

    #[test]
    fn top_four() -> anyhow::Result<()> {
        let db = TurboSqliteDatabase {};

        for user_id in 1..=4 {
            let username = format!("user{}", user_id);
            db.add_user(
                username.parse().unwrap(),
                "Pass@123".parse().unwrap(),
                "email@example.com".parse().unwrap(),
                "052-1122333".parse().unwrap(),
                Address::new("Netanya", "Alonim", 69),
                NaiveDate::parse_from_str("22/04/2038", DATE_FORMAT).unwrap(),
            )?;
        }

        let stats = [(10, 20, 2.2), (12, 20, 1.3), (15, 20, 2.6), (17, 20, 3.2)];

        for (user_id, stat) in stats.iter().enumerate() {
            let user_id = user_id + 1;
            let username = format!("user{}", user_id).parse().unwrap();
            let data = GameData {
                correct_answers: stat.0,
                wrong_answers: stat.1,
                avg_time: Duration::from_secs_f64(stat.2),
                ..Default::default()
            };
            db.submit_game_data(&username, data)?;
        }

        let scores = stats.map(|stat| calc_score(Duration::from_secs_f64(stat.2), stat.0 as i64));
        let highscores = db.get_five_highscores()?;

        assert_eq!(
            highscores,
            [
                ("user2".parse().unwrap(), scores[1]),
                ("user3".parse().unwrap(), scores[2]),
                ("user4".parse().unwrap(), scores[3]),
                ("user1".parse().unwrap(), scores[0]),
            ]
        );

        Ok(())
    }
}
