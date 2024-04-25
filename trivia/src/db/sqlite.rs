use std::path::Path;
use std::time::Duration;

use query::Iden;
use sea_query as query;
use sqlite::{Connection, ConnectionThreadSafe, State};

use crate::managers::game::{calc_score, GameData};

use super::question::QuestionData;
use super::{opentdb, Database, Error, Score};

pub struct SqliteDatabase {
    conn: ConnectionThreadSafe,
}

impl SqliteDatabase {
    pub fn connect(path: impl AsRef<Path>) -> Result<Self, Error> {
        let conn = Connection::open_thread_safe(path)?;
        Ok(Self { conn })
    }

    pub fn populate_questions(&mut self, amount: u8) -> Result<(), Error> {
        let questions = opentdb::get_questions(amount)?;

        for question in questions {
            let question_insert_query = query::Query::insert()
                .into_table(Question::Table)
                .columns([Question::Content])
                .values_panic([question.question().into()])
                .to_string(query::SqliteQueryBuilder);

            // when encountering a question that already exists on the db (19 => unique constraint
            // conflict) just skip it and go to the next question
            match self.conn.execute(question_insert_query) {
                Err(sqlite::Error { code: Some(19), .. }) => continue,
                res => res?,
            };

            let question_id = {
                let select_query = query::Query::select()
                    .columns([Question::Id])
                    .from(Question::Table)
                    .cond_where(query::Expr::col(Question::Content).is(question.question()))
                    .to_string(query::SqliteQueryBuilder);

                let mut statement = self.conn.prepare(&select_query)?;
                let State::Row = statement.next()? else {
                    return Err(anyhow::anyhow!("question doesn't exist after insertion").into());
                };

                statement.read::<i64, _>(Question::Id.to_string().as_str())?
            };

            let correct_answer_insert_query = query::Query::insert()
                .into_table(Answer::Table)
                .columns([Answer::Content, Answer::Correct, Answer::QuestionId])
                .values_panic([
                    question.correct_answer().into(),
                    true.into(),
                    question_id.into(),
                ])
                .to_string(query::SqliteQueryBuilder);
            self.conn.execute(correct_answer_insert_query)?;

            for incorrect_answer in question.incorrect_answers() {
                let incorrect_answer_insert_query = query::Query::insert()
                    .into_table(Answer::Table)
                    .columns([Answer::Content, Answer::Correct, Answer::QuestionId])
                    .values_panic([incorrect_answer.into(), false.into(), question_id.into()])
                    .to_string(query::SqliteQueryBuilder);
                self.conn.execute(incorrect_answer_insert_query)?;
            }
        }

        Ok(())
    }
}

impl Database for SqliteDatabase {
    fn open(&mut self) -> Result<(), Error> {
        // already opens the connection on creation
        // to design it like the cpp way where you have uninitialized variables (the
        // database connection) is unsafe, and I don't really want to deal with unsafe.

        let statements = [
            User::create_table_statement(),
            Question::create_table_statement(),
            Answer::create_table_statement(),
            Statistics::create_table_statement(),
        ];

        for statement in statements {
            self.conn
                .execute(statement.to_string(query::SqliteQueryBuilder))?;
        }

        match self.populate_questions(50) {
            Err(Error::OpenTDB(err)) => eprintln!("[INFO] can't get questions: {}", err),
            res => res?,
        };

        Ok(())
    }

    fn close(self) -> Result<(), Error> {
        Ok(()) // drop the connection and return without any errors
    }

    fn user_exists(&self, username: &str) -> Result<bool, Error> {
        let statement = query::Query::select()
            .column(User::Username)
            .from(User::Table)
            .and_where(query::Expr::col(User::Username).eq(username))
            .limit(1)
            .to_string(query::SqliteQueryBuilder);

        let mut exists = false;
        self.conn.iterate(statement, |_| {
            exists = true; // mark as found
            true // don't care if it will continue or not
        })?;

        Ok(exists)
    }

    fn password_matches(&self, username: &str, password: &str) -> Result<bool, Error> {
        let statement = query::Query::select()
            .columns([User::Username, User::Password])
            .from(User::Table)
            .and_where(query::Expr::col(User::Username).eq(username))
            .and_where(query::Expr::col(User::Password).eq(password))
            .limit(1)
            .to_string(query::SqliteQueryBuilder);

        let mut exists = false;
        self.conn.iterate(statement, |_| {
            exists = true; // mark as found
            true // don't care if it will continue or not
        })?;

        Ok(exists)
    }

    /// doesn't check whether the user exists or not
    fn add_user(&mut self, username: &str, password: &str, email: &str) -> Result<(), Error> {
        let statement = query::Query::insert()
            .into_table(User::Table)
            .columns([User::Username, User::Password, User::Email])
            .values_panic([username.into(), password.into(), email.into()])
            .to_string(query::SqliteQueryBuilder);
        self.conn.execute(statement)?;
        Ok(())
    }

    fn get_questions(&self, amount: usize) -> Result<Vec<QuestionData>, Error> {
        let select_question_query = query::Query::select()
            .columns([Question::Content, Question::Id])
            .from(Question::Table)
            .order_by_expr(query::Func::random().into(), query::Order::Asc)
            .limit(amount as u64)
            .to_string(query::SqliteQueryBuilder);

        let mut output = Vec::new();

        let mut questions_iter = self.conn.prepare(select_question_query)?;
        while let State::Row = questions_iter.next()? {
            let question_content =
                questions_iter.read::<String, _>(Question::Content.to_string().as_str())?;
            let question_id = questions_iter.read::<i64, _>(Question::Id.to_string().as_str())?;

            let get_answers_query = query::Query::select()
                .column(Answer::Content)
                .from(Answer::Table)
                .and_where(query::Expr::col(Answer::QuestionId).is(question_id))
                .to_string(query::SqliteQueryBuilder);

            let mut answers = Vec::new();
            let mut answers_iter = self.conn.prepare(get_answers_query)?;
            while let State::Row = answers_iter.next()? {
                let answer_content =
                    answers_iter.read::<String, _>(Answer::Content.to_string().as_str())?;

                answers.push(answer_content);
            }

            let correct_answer_query = query::Query::select()
                .column(Answer::Content)
                .from(Answer::Table)
                .and_where(query::Expr::col(Answer::QuestionId).is(question_id))
                .and_where(query::Expr::col(Answer::Correct).is(1))
                .to_string(query::SqliteQueryBuilder);

            let mut correct_answer_iter = self.conn.prepare(correct_answer_query)?;
            let State::Row = correct_answer_iter.next()? else {
                return Err(Error::NoCorrectAnswer {
                    question_id,
                    question_content,
                });
            };

            let correct_answer =
                correct_answer_iter.read::<String, _>(Answer::Content.to_string().as_str())?;

            // I can safely unwrap here because it's impossible to get the correct answer without
            // it being part of all of the answers
            let correct_answer_index = answers.iter().position(|s| *s == correct_answer).unwrap();

            let question = QuestionData::new(question_content, answers, correct_answer_index);
            output.push(question);
        }

        Ok(output)
    }

    // NOTE: LOTS of copy & paste, I can probably factor this out but it's not really necessary

    fn get_player_average_answer_time(&self, username: &str) -> Result<Duration, Error> {
        let statement = query::Query::select()
            .column(Statistics::AverageAnswerTime)
            .from(Statistics::Table)
            .inner_join(
                User::Table,
                query::Expr::col((Statistics::Table, Statistics::UserId))
                    .equals((User::Table, User::Id)),
            )
            .and_where(query::Expr::col((User::Table, User::Username)).eq(username))
            .to_string(query::SqliteQueryBuilder);

        let mut iter = self.conn.prepare(statement)?;
        if let State::Done = iter.next()? {
            return Err(Error::UserDoesntExist(username.to_string()));
        }

        let average_answer_time =
            iter.read::<f64, _>(Statistics::AverageAnswerTime.to_string().as_str())?;
        Ok(Duration::from_secs_f64(average_answer_time))
    }

    fn get_correct_answers_count(&self, username: &str) -> Result<i64, Error> {
        let statement = query::Query::select()
            .column(Statistics::CorrectAnswers)
            .from(Statistics::Table)
            .inner_join(
                User::Table,
                query::Expr::col((Statistics::Table, Statistics::UserId))
                    .equals((User::Table, User::Id)),
            )
            .and_where(query::Expr::col((User::Table, User::Username)).eq(username))
            .to_string(query::SqliteQueryBuilder);

        let mut iter = self.conn.prepare(statement)?;
        if let State::Done = iter.next()? {
            return Err(Error::UserDoesntExist(username.to_string()));
        }

        let correct_answers =
            iter.read::<i64, _>(Statistics::CorrectAnswers.to_string().as_str())?;
        Ok(correct_answers)
    }

    fn get_total_answers_count(&self, username: &str) -> Result<i64, Error> {
        let statement = query::Query::select()
            .column(Statistics::TotalAnswers)
            .from(Statistics::Table)
            .inner_join(
                User::Table,
                query::Expr::col((Statistics::Table, Statistics::UserId))
                    .equals((User::Table, User::Id)),
            )
            .and_where(query::Expr::col((User::Table, User::Username)).eq(username))
            .to_string(query::SqliteQueryBuilder);

        let mut iter = self.conn.prepare(statement)?;
        if let State::Done = iter.next()? {
            return Err(Error::UserDoesntExist(username.to_string()));
        }

        let total_answers = iter.read::<i64, _>(Statistics::TotalAnswers.to_string().as_str())?;
        Ok(total_answers)
    }

    fn get_games_count(&self, username: &str) -> Result<i64, Error> {
        let statement = query::Query::select()
            .column(Statistics::TotalGames)
            .from(Statistics::Table)
            .inner_join(
                User::Table,
                query::Expr::col((Statistics::Table, Statistics::UserId))
                    .equals((User::Table, User::Id)),
            )
            .and_where(query::Expr::col((User::Table, User::Username)).eq(username))
            .to_string(query::SqliteQueryBuilder);

        let mut iter = self.conn.prepare(statement)?;
        if let State::Done = iter.next()? {
            return Err(Error::UserDoesntExist(username.to_string()));
        }

        let total_games = iter.read::<i64, _>(Statistics::TotalGames.to_string().as_str())?;
        Ok(total_games)
    }

    fn get_score(&self, username: &str) -> Result<super::Score, Error> {
        let statement = query::Query::select()
            .column(Statistics::Score)
            .from(Statistics::Table)
            .inner_join(
                User::Table,
                query::Expr::col((Statistics::Table, Statistics::UserId))
                    .equals((User::Table, User::Id)),
            )
            .and_where(query::Expr::col((User::Table, User::Username)).eq(username))
            .to_string(query::SqliteQueryBuilder);

        let mut iter = self.conn.prepare(statement)?;
        if let State::Done = iter.next()? {
            return Err(Error::UserDoesntExist(username.to_string()));
        }

        let total_games = iter.read::<Score, _>(Statistics::Score.to_string().as_str())?;
        Ok(total_games)
    }

    fn get_five_highscores(&self) -> Result<[Option<(String, super::Score)>; 5], Error> {
        let statement = query::Query::select()
            .column(User::Username)
            .column(Statistics::Score)
            .from(Statistics::Table)
            .order_by(Statistics::Score, query::Order::Desc)
            .inner_join(
                User::Table,
                query::Expr::col((Statistics::Table, Statistics::UserId))
                    .equals((User::Table, User::Id)),
            )
            .limit(5)
            .to_string(query::SqliteQueryBuilder);

        let mut scores: [Option<(String, super::Score)>; 5] = Default::default();
        let mut index = 0;
        let mut iter = self.conn.prepare(statement)?;
        while let Ok(State::Row) = iter.next() {
            let username = iter.read::<String, _>(User::Username.to_string().as_str())?;
            let score = iter.read::<Score, _>(Statistics::Score.to_string().as_str())?;
            scores[index] = Some((username, score));
            index += 1;
        }

        Ok(scores)
    }

    // NOTE: not tested, hoping that this function works as expected
    fn submit_game_data(&mut self, username: &str, game_data: GameData) -> Result<(), Error> {
        let GameData {
            correct_answers,
            wrong_answers,
            avg_time,
            ..
        } = game_data;

        let user_id = {
            let statement = query::Query::select()
                .column(User::Id)
                .from(User::Table)
                .and_where(query::Expr::col(User::Username).eq(username))
                .to_string(query::SqliteQueryBuilder);

            let mut iter = self.conn.prepare(statement)?;
            let State::Row = iter.next()? else {
                return Err(Error::UserDoesntExist(username.to_string()));
            };

            iter.read::<i64, _>(User::Id.to_string().as_str())?
        };

        let old_total_answers = self.get_total_answers_count(username).unwrap_or_default();
        let total_answers = old_total_answers + wrong_answers as i64 + correct_answers as i64;
        let avg_time = {
            let old_total_time = self
                .get_player_average_answer_time(username)
                .unwrap_or_default()
                .as_secs_f64()
                * old_total_answers as f64;
            let new_total_time = avg_time.as_secs_f64() * (wrong_answers + correct_answers) as f64;
            (old_total_time + new_total_time) / total_answers as f64
        };

        let correct_answers =
            self.get_correct_answers_count(username).unwrap_or_default() + correct_answers as i64;

        let total_games = self.get_games_count(username).unwrap_or_default() + 1;

        let statement = query::Query::insert()
            .replace()
            .into_table(Statistics::Table)
            .columns([
                Statistics::UserId,
                Statistics::CorrectAnswers,
                Statistics::TotalAnswers,
                Statistics::AverageAnswerTime,
                Statistics::TotalAnswers,
                Statistics::TotalGames,
                Statistics::Score,
            ])
            .values_panic([
                user_id.into(),
                correct_answers.into(),
                total_answers.into(),
                avg_time.into(),
                total_answers.into(),
                total_games.into(),
                calc_score(
                    Duration::from_secs_f64(avg_time),
                    correct_answers,
                ).into(),
            ])
            .to_string(query::SqliteQueryBuilder);

        Ok(self.conn.execute(statement)?)
    }
}

// Users table definition
#[derive(query::Iden)]
enum User {
    Table,
    Id,
    Username,
    Password,
    Email,
}

impl User {
    fn create_table_statement() -> query::TableCreateStatement {
        query::Table::create()
            .table(User::Table)
            .if_not_exists()
            .col(
                query::ColumnDef::new(User::Id)
                    .integer()
                    .primary_key()
                    .not_null()
                    .auto_increment(),
            )
            .col(
                query::ColumnDef::new(User::Username)
                    .text()
                    .unique_key()
                    .not_null(),
            )
            .col(query::ColumnDef::new(User::Password).text().not_null())
            .col(query::ColumnDef::new(User::Email).text().not_null())
            .to_owned()
    }
}

#[derive(query::Iden)]
enum Question {
    Table,
    Id,
    Content,
}

impl Question {
    fn create_table_statement() -> query::TableCreateStatement {
        query::Table::create()
            .table(Question::Table)
            .if_not_exists()
            .col(
                query::ColumnDef::new(Question::Id)
                    .integer()
                    .primary_key()
                    .not_null()
                    .auto_increment(),
            )
            .col(
                query::ColumnDef::new(Question::Content)
                    .text()
                    .not_null()
                    .unique_key(),
            )
            .to_owned()
    }
}

#[derive(query::Iden)]
enum Answer {
    Table,
    Id,
    Content,
    Correct,
    QuestionId,
}

impl Answer {
    fn create_table_statement() -> query::TableCreateStatement {
        query::Table::create()
            .table(Answer::Table)
            .if_not_exists()
            .col(
                query::ColumnDef::new(Answer::Id)
                    .integer()
                    .primary_key()
                    .not_null()
                    .auto_increment(),
            )
            .col(query::ColumnDef::new(Answer::Content).text().not_null())
            .col(query::ColumnDef::new(Answer::Correct).boolean().not_null())
            .col(
                query::ColumnDef::new(Answer::QuestionId)
                    .integer()
                    .not_null(),
            )
            .foreign_key(
                query::ForeignKey::create()
                    .from(Answer::Table, Answer::QuestionId)
                    .to(Question::Table, Question::Id),
            )
            .to_owned()
    }
}

#[derive(query::Iden)]
enum Statistics {
    Table,
    Id,
    CorrectAnswers,
    TotalAnswers,
    AverageAnswerTime, // in seconds
    TotalGames,
    Score,
    UserId,
}

impl Statistics {
    fn create_table_statement() -> query::TableCreateStatement {
        query::Table::create()
            .table(Statistics::Table)
            .if_not_exists()
            .col(
                query::ColumnDef::new(Statistics::Id)
                    .integer()
                    .not_null()
                    .primary_key()
                    .auto_increment(),
            )
            .col(
                query::ColumnDef::new(Statistics::CorrectAnswers)
                    .integer()
                    .not_null(),
            )
            .col(
                query::ColumnDef::new(Statistics::TotalAnswers)
                    .integer()
                    .not_null(),
            )
            .col(
                query::ColumnDef::new(Statistics::AverageAnswerTime)
                    .double()
                    .not_null(),
            )
            .col(
                query::ColumnDef::new(Statistics::TotalGames)
                    .integer()
                    .not_null(),
            )
            .col(query::ColumnDef::new(Statistics::Score).double().not_null())
            .col(
                query::ColumnDef::new(Statistics::UserId)
                    .integer()
                    .not_null(),
            )
            .foreign_key(
                query::ForeignKey::create()
                    .from(Statistics::Table, Statistics::UserId)
                    .to(User::Table, User::Id),
            )
            .to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signup() {
        let mut db = SqliteDatabase::connect(":memory:").unwrap();
        db.open().unwrap();
        assert!(!db.user_exists("me").unwrap());
        db.add_user("me", "password1234", "main@example.com")
            .unwrap();
        assert!(db.user_exists("me").unwrap());
        assert!(db.password_matches("me", "password1234").unwrap());
        assert!(!db.password_matches("me", "jqwemnedfk").unwrap());
    }

    #[test]
    #[ignore = "prevent API spamming"]
    fn populate_questions() -> anyhow::Result<()> {
        let mut db = SqliteDatabase::connect(":memory:")?;
        db.open()?;
        db.populate_questions(100)?;
        db.get_questions(10)?;
        Ok(())
    }

    fn insert_stats(
        db: &mut SqliteDatabase,
        correct: i64,
        total_answers: i64,
        avg_time: f64,
        total_games: i64,
        user_id: i64,
    ) -> anyhow::Result<()> {
        let score = calc_score(Duration::from_secs_f64(avg_time), correct);
        let statement = query::Query::insert()
            .columns([
                Statistics::CorrectAnswers,
                Statistics::TotalAnswers,
                Statistics::AverageAnswerTime,
                Statistics::TotalGames,
                Statistics::Score,
                Statistics::UserId,
            ])
            .into_table(Statistics::Table)
            .values_panic([
                correct.into(),
                total_answers.into(),
                avg_time.into(),
                total_games.into(),
                score.into(),
                user_id.into(),
            ])
            .to_string(query::SqliteQueryBuilder);

        db.conn.execute(statement)?;
        Ok(())
    }

    #[test]
    fn top_four() -> anyhow::Result<()> {
        let mut db = SqliteDatabase::connect(":memory:")?;
        db.open()?;

        for user_id in 1..=4 {
            let username = format!("user{}", user_id);
            db.add_user(&username, "pass", "email@example.com")?;
        }

        let stats = [(10, 20, 2.2), (12, 20, 1.3), (15, 20, 2.6), (17, 20, 3.2)];

        for (user_id, stat) in stats.iter().enumerate() {
            let user_id = user_id + 1;
            insert_stats(&mut db, stat.0, stat.1, stat.2, 12, user_id as i64)?;
        }

        let scores = stats.map(|stat| calc_score(Duration::from_secs_f64(stat.2), stat.0));
        let highscores = db.get_five_highscores()?;

        assert_eq!(
            highscores,
            [
                Some(("user2".to_string(), scores[1])),
                Some(("user3".to_string(), scores[2])),
                Some(("user4".to_string(), scores[3])),
                Some(("user1".to_string(), scores[0])),
                None
            ]
        );

        Ok(())
    }
}
