use std::path::Path;
use std::time::Duration;

use sea_query as query;
use query::Iden;
use sqlite::{Connection, ConnectionThreadSafe, State};

use super::{opentdb, Database, Score};
use super::question::QuestionData;

pub struct SqliteDatabase {
    conn: ConnectionThreadSafe,
}

impl SqliteDatabase {
    pub fn connect(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let conn = Connection::open_thread_safe(path)?;
        Ok(Self { conn })
    }

    pub fn populate_questions(&mut self, amount: u8) -> anyhow::Result<()> {
        let questions = opentdb::get_questions(amount)?;

        for question in questions {
            let question_insert_query = query::Query::insert()
                .into_table(Question::Table)
                .columns([Question::Content])
                .values_panic([question.question().into()])
                .to_string(query::SqliteQueryBuilder);
            self.conn.execute(question_insert_query)?;

            let question_id = {
                let select_query = query::Query::select()
                    .columns([Question::Id])
                    .from(Question::Table)
                    .cond_where(query::Expr::col(Question::Content).is(question.question()))
                    .to_string(query::SqliteQueryBuilder);

                let mut statement = self.conn.prepare(&select_query)?;
                let State::Row = statement.next()? else {
                    anyhow::bail!("question doesn't exist after insertion");
                };

                statement.read::<i64, _>(Question::Id.to_string().as_str())?
            };

            let possible_answers = question.possible_answers();

            let correct_answer_insert_query = query::Query::insert()
                .into_table(Answer::Table)
                .columns([Answer::Content, Answer::Correct, Answer::QuestionId])
                .values_panic([possible_answers.correct_answer.into(), true.into(), question_id.into()])
                .to_string(query::SqliteQueryBuilder);
            self.conn.execute(correct_answer_insert_query)?;

            for incorrect_answer in possible_answers.incorrect_answers {
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
    fn open(&mut self) -> anyhow::Result<()> {
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

        Ok(())
    }

    fn close(self) -> anyhow::Result<()> {
        Ok(()) // drop the connection and return without any errors
    }

    fn user_exists(&self, username: &str) -> anyhow::Result<bool> {
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

    fn password_matches(&self, username: &str, password: &str) -> anyhow::Result<bool> {
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
    fn add_user(&mut self, username: &str, password: &str, email: &str) -> anyhow::Result<()> {
        let statement = query::Query::insert()
            .into_table(User::Table)
            .columns([User::Username, User::Password, User::Email])
            .values_panic([username.into(), password.into(), email.into()])
            .to_string(query::SqliteQueryBuilder);
        self.conn.execute(statement)?;
        Ok(())
    }

    fn get_questions(&self, amount: u8) -> anyhow::Result<Vec<QuestionData>> {
        let select_question_query = query::Query::select()
            .columns([Question::Content, Question::Id])
            .from(Question::Table)
            .order_by_expr(query::Func::random().into(), query::Order::Asc)
            .limit(amount.into())
            .to_string(query::SqliteQueryBuilder);

        let mut output = Vec::new();

        let mut questions_iter = self.conn.prepare(select_question_query)?;
        while let State::Row = questions_iter.next()? {
            let question_content = questions_iter.read::<String, _>(Question::Content.to_string().as_str())?;
            let question_id = questions_iter.read::<i64, _>(Question::Id.to_string().as_str())?;

            let get_answers_query = query::Query::select()
                .columns([Answer::Correct, Answer::Content])
                .from(Answer::Table)
                .and_where(query::Expr::col(Answer::QuestionId).is(question_id))
                .to_string(query::SqliteQueryBuilder);

            let mut question = QuestionData {
                question: question_content,
                correct_answer: String::new(),
                incorrect_answers: Vec::new(),
            };

            let mut answers_iter = self.conn.prepare(get_answers_query)?;
            while let State::Row = answers_iter.next()? {
                let answer_content = answers_iter.read::<String, _>(Answer::Content.to_string().as_str())?;
                let answer_correct = answers_iter.read::<i64, _>(Answer::Correct.to_string().as_str())? == 1;

                if answer_correct {
                    question.correct_answer = answer_content;
                } else {
                    question.incorrect_answers.push(answer_content)
                }
            }

            output.push(question);
        }

        Ok(output)
    }

    // NOTE: LOTS of copy & paste, I can probably factor this out but it's not really necessary

    fn get_player_average_answer_time(&self, username: &str) -> anyhow::Result<Duration> {
        let statement = query::Query::select()
            .column(Statistics::AverageAnswerTime)
            .from(Statistics::Table)
            .inner_join(
                User::Table,
                query::Expr::col((Statistics::Table, Statistics::UserId))
                    .equals((User::Table, User::Id))
            )
            .and_where(query::Expr::col((User::Table, User::Username)).eq(username))
            .to_string(query::SqliteQueryBuilder);

        let mut iter = self.conn.prepare(statement)?;
        if let State::Done = iter.next()? {
            anyhow::bail!("User doesn't exist");
        }

        let average_answer_time = iter.read::<f64, _>(Statistics::AverageAnswerTime.to_string().as_str())?;
        Ok(Duration::from_secs_f64(average_answer_time))
    }

    fn get_correct_answers_count(&self, username: &str) -> anyhow::Result<i64> {
        let statement = query::Query::select()
            .column(Statistics::CorrectAnswers)
            .from(Statistics::Table)
            .inner_join(
                User::Table,
                query::Expr::col((Statistics::Table, Statistics::UserId))
                    .equals((User::Table, User::Id))
            )
            .and_where(query::Expr::col((User::Table, User::Username)).eq(username))
            .to_string(query::SqliteQueryBuilder);

        let mut iter = self.conn.prepare(statement)?;
        if let State::Done = iter.next()? {
            anyhow::bail!("User doesn't exist");
        }

        let correct_answers = iter.read::<i64, _>(Statistics::CorrectAnswers.to_string().as_str())?;
        Ok(correct_answers)
    }

    fn get_total_answers_count(&self, username: &str) -> anyhow::Result<i64> {
        let statement = query::Query::select()
            .column(Statistics::TotalAnswers)
            .from(Statistics::Table)
            .inner_join(
                User::Table,
                query::Expr::col((Statistics::Table, Statistics::UserId))
                    .equals((User::Table, User::Id))
            )
            .and_where(query::Expr::col((User::Table, User::Username)).eq(username))
            .to_string(query::SqliteQueryBuilder);

        let mut iter = self.conn.prepare(statement)?;
        if let State::Done = iter.next()? {
            anyhow::bail!("User doesn't exist");
        }

        let total_answers = iter.read::<i64, _>(Statistics::TotalAnswers.to_string().as_str())?;
        Ok(total_answers)
    }

    fn get_games_count(&self, username: &str) -> anyhow::Result<i64> {
        let statement = query::Query::select()
            .column(Statistics::TotalGames)
            .from(Statistics::Table)
            .inner_join(
                User::Table,
                query::Expr::col((Statistics::Table, Statistics::UserId))
                    .equals((User::Table, User::Id))
            )
            .and_where(query::Expr::col((User::Table, User::Username)).eq(username))
            .to_string(query::SqliteQueryBuilder);

        let mut iter = self.conn.prepare(statement)?;
        if let State::Done = iter.next()? {
            anyhow::bail!("User doesn't exist");
        }

        let total_games = iter.read::<i64, _>(Statistics::TotalGames.to_string().as_str())?;
        Ok(total_games)
    }

    fn get_score(&self, username: &str) -> anyhow::Result<super::Score> {
        let statement = query::Query::select()
            .column(Statistics::Score)
            .from(Statistics::Table)
            .inner_join(
                User::Table,
                query::Expr::col((Statistics::Table, Statistics::UserId))
                    .equals((User::Table, User::Id))
            )
            .and_where(query::Expr::col((User::Table, User::Username)).eq(username))
            .to_string(query::SqliteQueryBuilder);

        let mut iter = self.conn.prepare(statement)?;
        if let State::Done = iter.next()? {
            anyhow::bail!("User doesn't exist");
        }

        let total_games = iter.read::<Score, _>(Statistics::Score.to_string().as_str())?;
        Ok(total_games)
    }

    fn get_five_highscores(&self) -> anyhow::Result<[super::Score; 5]> {
        let statement = query::Query::select()
            .column(Statistics::Score)
            .from(Statistics::Table)
            .order_by(Statistics::Score, query::Order::Desc)
            .limit(5)
            .to_string(query::SqliteQueryBuilder);

        let mut scores = [0.0; 5];
        let mut index = 0;
        let mut iter = self.conn.prepare(statement)?;
        while let Ok(State::Row) = iter.next() {
            scores[index] = iter.read::<Score, _>(Statistics::Score.to_string().as_str())?;
            index += 1;
        }

        Ok(scores)
    }
}

#[allow(unused)]
fn calc_score(average_answer_time: Duration, correct_answers: i64, total_answers: i64) -> Score {
    // TODO: the user can just spam wrong answers and still get a really good score
    //       find a way to prevent this, meaning a new score evaluation algorithm
    let answer_ratio = correct_answers as f64 / total_answers as f64;
    let time_ratio = 1. / average_answer_time.as_secs_f64().max(1.);
    answer_ratio * time_ratio
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
            .col(query::ColumnDef::new(Question::Content).text().not_null().unique_key())
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
            .col(query::ColumnDef::new(Statistics::Id).integer().not_null().primary_key().auto_increment())
            .col(query::ColumnDef::new(Statistics::CorrectAnswers).integer().not_null())
            .col(query::ColumnDef::new(Statistics::TotalAnswers).integer().not_null())
            .col(query::ColumnDef::new(Statistics::AverageAnswerTime).double().not_null())
            .col(query::ColumnDef::new(Statistics::TotalGames).integer().not_null())
            .col(query::ColumnDef::new(Statistics::Score).double().not_null())
            .col(query::ColumnDef::new(Statistics::UserId).integer().not_null())
            .foreign_key(
                query::ForeignKey::create()
                    .from(Statistics::Table, Statistics::UserId)
                    .to(User::Table, User::Id)
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

// enum Statistics {
//     Table,
//     Id,
//     CorrectAnswers,
//     TotalAnswers,
//     AverageAnswerTime, // in seconds
//     TotalGames,
//     Score,
//     UserId,
// }

    fn insert_stats(
        db: &mut SqliteDatabase,
        correct: i64,
        total_answers: i64,
        avg_time: f64,
        total_games: i64,
        user_id: i64
    ) -> anyhow::Result<()> {
        let score = calc_score(Duration::from_secs_f64(avg_time), correct, total_answers);
        let statement = query::Query::insert()
            .columns([
                Statistics::CorrectAnswers,
                Statistics::TotalAnswers,
                Statistics::AverageAnswerTime,
                Statistics::TotalGames,
                Statistics::Score,
                Statistics::UserId
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

        let stats = [
            (10, 20, 2.2),
            (12, 20, 1.3),
            (15, 20, 2.6),
            (17, 20, 3.2),
        ];

        for (user_id, stat) in stats.iter().enumerate() {
            let user_id = user_id + 1;
            insert_stats(&mut db, stat.0, stat.1, stat.2, 12, user_id as i64)?;
        }

        let scores = stats.map(|stat| calc_score(Duration::from_secs_f64(stat.2), stat.0, stat.1));
        let highscores = db.get_five_highscores()?;

        assert_eq!(highscores, [scores[1], scores[2], scores[3], scores[0], 0.]);

        Ok(())
    }
}
