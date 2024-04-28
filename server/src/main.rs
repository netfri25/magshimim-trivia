use trivia::db::{Database, SqliteDatabase};
use trivia::server::TriviaServer;

fn main() {
    let mut db = match SqliteDatabase::connect("trivia-db.sqlite") {
        Ok(db) => db,
        Err(err) => {
            eprintln!("[FATAL ERROR] unable to connect to db: {}", err);
            return;
        }
    };

    if let Err(err) = db.open() {
        eprintln!("[FATAL ERROR] unable to open db: {}", err);
        return;
    }

    if let Err(err) = db.populate_questions(50) {
        eprintln!("[WARN] unable to add questions to db: {}", err);
    }

    let server = match TriviaServer::build("127.0.0.1:6969", db) {
        Ok(server) => server,
        Err(err) => {
            eprintln!("[FATAL ERROR] unable to run server: {}", err);
            return
        }
    };

    server.run()
}
