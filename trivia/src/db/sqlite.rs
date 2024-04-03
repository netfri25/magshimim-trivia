use std::path::Path;

use sea_query as query;
use query::Iden;
use sqlite::{Connection, ConnectionThreadSafe, State};

use super::{opentdb, Database};

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

    fn get_questions(&self, amount: u8) -> Vec<super::Question> {
        todo!()
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
    fn populate_questions() -> anyhow::Result<()> {
        let mut db = SqliteDatabase::connect("test.sqlite")?;
        db.open()?;
        db.populate_questions(100)?;
        Ok(())
    }
}
