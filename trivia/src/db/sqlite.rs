use std::path::Path;
use std::time::Duration;

use chrono::NaiveDate;
use query::Iden;
use sea_query as query;
use sqlite::{Connection, ConnectionThreadSafe, State};

use crate::email::Email;
use crate::managers::game::{calc_score, GameData};
use crate::managers::statistics::Highscores;
use crate::messages::{Address, PhoneNumber, DATE_FORMAT};
use crate::password::Password;
use crate::username::Username;

use super::question::QuestionData;
use super::{opentdb, Database, Error, Score};

pub struct SqliteDatabase {
    conn: ConnectionThreadSafe,
}

impl SqliteDatabase {
    pub fn connect(path: impl AsRef<Path>) -> Result<Self, Error> {
        let conn = Connection::open_thread_safe(path)?;
        let statements = [
            User::create_table_statement(),
            Question::create_table_statement(),
            Answer::create_table_statement(),
            Statistics::create_table_statement(),
        ];

        for statement in statements {
            conn.execute(statement.to_string(query::SqliteQueryBuilder))?;
        }

        Ok(Self { conn })
    }

    pub fn populate_questions(&self, amount: u8) -> Result<(), Error> {
        opentdb::get_questions(amount)?
            .into_iter()
            .try_for_each(|question| self.add_question(&QuestionData::from(question)).map(drop))
    }

    fn get_stats<T: sqlite::ReadableWithIndex>(
        &self,
        username: &Username,
        col: Statistics,
    ) -> Result<T, Error> {
        let statement = query::Query::select()
            .column(col)
            .from(Statistics::Table)
            .inner_join(
                User::Table,
                query::Expr::col((Statistics::Table, Statistics::Id))
                    .equals((User::Table, User::Id)),
            )
            .and_where(query::Expr::col((User::Table, User::Username)).eq(username.as_ref()))
            .to_string(query::SqliteQueryBuilder);

        let mut iter = self.conn.prepare(statement)?;
        if let State::Done = iter.next()? {
            return Err(Error::UserDoesntExist(username.clone()));
        }

        Ok(iter.read::<T, _>(col.to_string().as_str())?)
    }
}

impl Database for SqliteDatabase {
    fn user_exists(&self, username: &Username) -> Result<bool, Error> {
        let statement = query::Query::select()
            .column(User::Username)
            .from(User::Table)
            .and_where(query::Expr::col(User::Username).eq(username.as_ref()))
            .limit(1)
            .to_string(query::SqliteQueryBuilder);

        Ok(self.conn.prepare(statement)?.next()? == State::Row)
    }

    fn password_matches(&self, username: &Username, password: &Password) -> Result<bool, Error> {
        let statement = query::Query::select()
            .columns([User::Username, User::Password])
            .from(User::Table)
            .and_where(query::Expr::col(User::Username).eq(username.as_ref()))
            .and_where(query::Expr::col(User::Password).eq(password.as_ref()))
            .limit(1)
            .to_string(query::SqliteQueryBuilder);

        Ok(self.conn.prepare(statement)?.next()? == State::Row)
    }

    /// doesn't check whether the user exists or not
    fn add_user(
        &self,
        username: Username,
        password: Password,
        email: Email,
        phone: PhoneNumber,
        address: Address,
        birth_date: NaiveDate,
    ) -> Result<(), Error> {
        let statement = query::Query::insert()
            .into_table(User::Table)
            .columns([
                User::Username,
                User::Password,
                User::Email,
                User::Phone,
                User::City,
                User::Street,
                User::Apartment,
                User::BirthDate,
            ])
            .values_panic([
                username.as_ref().into(),
                password.as_ref().into(),
                email.as_ref().into(),
                phone.to_string().into(),
                address.city().into(),
                address.street().into(),
                address.apartment().into(),
                birth_date.format(DATE_FORMAT).to_string().into(),
            ])
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

    fn get_player_average_answer_time(&self, username: &Username) -> Result<Duration, Error> {
        self.get_stats(username, Statistics::AverageAnswerTime)
            .map(Duration::from_secs_f64)
    }

    fn get_correct_answers_count(&self, username: &Username) -> Result<i64, Error> {
        self.get_stats(username, Statistics::CorrectAnswers)
    }

    fn get_total_answers_count(&self, username: &Username) -> Result<i64, Error> {
        self.get_stats(username, Statistics::TotalAnswers)
    }

    fn get_games_count(&self, username: &Username) -> Result<i64, Error> {
        self.get_stats(username, Statistics::TotalGames)
    }

    fn get_score(&self, username: &Username) -> Result<Score, Error> {
        self.get_stats(username, Statistics::OverallScore)
    }

    fn get_five_highscores(&self) -> Result<Highscores, Error> {
        let statement = query::Query::select()
            .column(User::Username)
            .column(Statistics::OverallScore)
            .from(Statistics::Table)
            .order_by(Statistics::OverallScore, query::Order::Desc)
            .inner_join(
                User::Table,
                query::Expr::col((Statistics::Table, Statistics::Id))
                    .equals((User::Table, User::Id)),
            )
            .limit(5)
            .to_string(query::SqliteQueryBuilder);

        let mut scores: Highscores = Default::default();
        let mut index = 0;
        let mut iter = self.conn.prepare(statement)?;
        while let Ok(State::Row) = iter.next() {
            let username = iter
                .read::<String, _>(User::Username.to_string().as_str())?
                .parse()?;
            let score = iter.read::<Score, _>(Statistics::OverallScore.to_string().as_str())?;
            scores[index] = Some((username, score));
            index += 1;
        }

        Ok(scores)
    }

    // NOTE: not tested, hoping that this function works as expected
    fn submit_game_data(&self, username: &Username, game_data: GameData) -> Result<(), Error> {
        let GameData {
            correct_answers: current_correct_answers,
            wrong_answers: current_wrong_answers,
            avg_time: current_avg_time,
            ..
        } = game_data;

        let user_id = {
            let statement = query::Query::select()
                .column(User::Id)
                .from(User::Table)
                .and_where(query::Expr::col(User::Username).eq(username.as_ref()))
                .to_string(query::SqliteQueryBuilder);

            let mut iter = self.conn.prepare(statement)?;
            let State::Row = iter.next()? else {
                return Err(Error::UserDoesntExist(username.clone()));
            };

            iter.read::<i64, _>(User::Id.to_string().as_str())?
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

        let statement = query::Query::insert()
            .replace()
            .into_table(Statistics::Table)
            .columns([
                Statistics::Id,
                Statistics::CorrectAnswers,
                Statistics::TotalAnswers,
                Statistics::AverageAnswerTime,
                Statistics::TotalAnswers,
                Statistics::TotalGames,
                Statistics::OverallScore,
            ])
            .values_panic([
                user_id.into(),
                correct_answers.into(),
                total_answers.into(),
                avg_time.into(),
                total_answers.into(),
                total_games.into(),
                calc_score(Duration::from_secs_f64(avg_time), correct_answers).into(),
            ])
            .to_string(query::SqliteQueryBuilder);

        Ok(self.conn.execute(statement)?)
    }

    fn add_question(&self, question: &QuestionData) -> Result<bool, Error> {
        let question_insert_query = query::Query::insert()
            .into_table(Question::Table)
            .columns([Question::Content])
            .values_panic([question.question.as_str().into()])
            .to_string(query::SqliteQueryBuilder);

        // when encountering a question that already exists on the db (19 => unique constraint
        // conflict) just skip it and go to the next question
        match self.conn.execute(question_insert_query) {
            Err(sqlite::Error { code: Some(19), .. }) => return Ok(false),
            res => res?,
        };

        let question_id = {
            let select_query = query::Query::select()
                .columns([Question::Id])
                .from(Question::Table)
                .cond_where(query::Expr::col(Question::Content).is(question.question.as_str()))
                .to_string(query::SqliteQueryBuilder);

            let mut statement = self.conn.prepare(&select_query)?;
            let State::Row = statement.next()? else {
                return Err(anyhow::anyhow!("question doesn't exist after insertion").into());
            };

            statement.read::<i64, _>(Question::Id.to_string().as_str())?
        };

        for (i, answer) in question.answers.iter().enumerate() {
            let correct = i == question.correct_answer_index;
            let answer_insert_query = query::Query::insert()
                .into_table(Answer::Table)
                .columns([Answer::Content, Answer::Correct, Answer::QuestionId])
                .values_panic([answer.into(), correct.into(), question_id.into()])
                .to_string(query::SqliteQueryBuilder);
            self.conn.execute(answer_insert_query)?;
        }

        Ok(true)
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
    Phone,
    City,
    Street,
    Apartment,
    BirthDate,
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
            .col(query::ColumnDef::new(User::Phone).text().not_null())
            .col(query::ColumnDef::new(User::City).text().not_null())
            .col(query::ColumnDef::new(User::Street).text().not_null())
            .col(query::ColumnDef::new(User::Apartment).unsigned().not_null())
            .col(query::ColumnDef::new(User::BirthDate).text().not_null())
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

#[derive(query::Iden, Clone, Copy)]
enum Statistics {
    Table,
    Id, // also the user_id
    CorrectAnswers,
    TotalAnswers,
    AverageAnswerTime, // in seconds
    TotalGames,
    OverallScore,
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
                    .primary_key(),
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
            .col(
                query::ColumnDef::new(Statistics::OverallScore)
                    .double()
                    .not_null(),
            )
            .foreign_key(
                query::ForeignKey::create()
                    .from(Statistics::Table, Statistics::Id)
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
        let db = SqliteDatabase::connect(":memory:").unwrap();
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
        let db = SqliteDatabase::connect(":memory:")?;
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
                Statistics::OverallScore,
                Statistics::Id,
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
            insert_stats(&mut db, stat.0, stat.1, stat.2, 12, user_id as i64)?;
        }

        let scores = stats.map(|stat| calc_score(Duration::from_secs_f64(stat.2), stat.0));
        let highscores = db.get_five_highscores()?;

        assert_eq!(
            highscores,
            [
                Some(("user2".parse().unwrap(), scores[1])),
                Some(("user3".parse().unwrap(), scores[2])),
                Some(("user4".parse().unwrap(), scores[3])),
                Some(("user1".parse().unwrap(), scores[0])),
                None
            ]
        );

        Ok(())
    }
}
