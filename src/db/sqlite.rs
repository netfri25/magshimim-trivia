use std::path::Path;

use sea_query as query;
use sqlite::{ConnectionThreadSafe, Connection};

use super::{DBResult, Database};

pub struct SqliteDatabase {
    conn: ConnectionThreadSafe,
}

impl SqliteDatabase {
    pub fn connect(path: impl AsRef<Path>) -> DBResult<Self> {
        let conn = Connection::open_thread_safe(path)?;
        Ok(Self { conn })
    }
}

impl Database for SqliteDatabase {
    fn open(&mut self) -> DBResult<()> {
        // already opens the connection on creation
        // to design it like the cpp way where you have uninitialized variables (the
        // database connection) is unsafe, and I don't really want to deal with unsafe.

        // create the users table
        let statement = User::create_table_statement().to_string(query::SqliteQueryBuilder);
        self.conn.execute(statement)?;
        Ok(())
    }

    fn close(self) -> DBResult<()> {
        Ok(()) // drop the connection and return without any errors
    }

    fn user_exists(&self, username: &str) -> DBResult<bool> {
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

    fn password_matches(&self, username: &str, password: &str) -> DBResult<bool> {
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
    fn add_user(&mut self, username: &str, password: &str, email: &str) -> DBResult<()> {
        let statement = query::Query::insert()
            .into_table(User::Table)
            .columns([User::Username, User::Password, User::Email])
            .values_panic([username.into(), password.into(), email.into()])
            .to_string(query::SqliteQueryBuilder);
        self.conn.execute(statement)?;
        Ok(())
    }
}


// Table definitions

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
            .col(query::ColumnDef::new(User::Id).integer().primary_key().not_null().auto_increment())
            .col(query::ColumnDef::new(User::Username).text().unique_key().not_null())
            .col(query::ColumnDef::new(User::Password).text().unique_key().not_null())
            .col(query::ColumnDef::new(User::Email).text().not_null())
            .to_owned()
    }
}

mod tests {
    struct RemoveDB<'a>(&'a str);
    impl Drop for RemoveDB<'_> {
        fn drop(&mut self) {
            std::fs::remove_file(self.0).ok();
        }
    }

    #[test]
    fn signup() {
        use crate::db::Database;
        use super::SqliteDatabase;

        const DB_PATH: &str = "test-db.sqlite";

        // because of the `Drop` implementation, it will assure us that the DB_PATH file will be
        // removed at the end of this function / when it panics
        let _remove_db = RemoveDB(DB_PATH);

        let mut db = SqliteDatabase::connect(DB_PATH).unwrap();
        db.open().unwrap();
        assert!(!db.user_exists("me").unwrap());
        db.add_user("me", "password1234", "main@example.com").unwrap();
        assert!(db.user_exists("me").unwrap());
        assert!(db.password_matches("me", "password1234").unwrap());
        assert!(!db.password_matches("me", "jqwemnedfk").unwrap());
    }
}
