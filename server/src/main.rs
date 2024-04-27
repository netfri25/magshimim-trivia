use trivia::db::{Database, SqliteDatabase};
use trivia::server::TriviaServer;

fn main() {
    let mut db = SqliteDatabase::connect("trivia-db.sqlite").unwrap_or_else(|err| {
        eprintln!("[FATAL ERROR] unable to connect to db: {}", err);
        std::process::exit(1);
    });

    db.open()?;
    db.populate_questions(50).unwrap_or_else(|err| {
        eprintln!("[WARN] unable to add questions to db: {}", err);
    });

    TriviaServer::build("127.0.0.1:6969", db)
        .map(TriviaServer::run)
        .unwrap_or_else(|err| {
            eprintln!("[FATAL ERROR] unable to run server: {}", err);
        });
}
