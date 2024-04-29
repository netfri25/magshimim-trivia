use std::sync::Mutex;

use trivia::db::{Database, SqliteDatabase};

mod communicator;
mod defer;

mod server;
use server::Server;

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

    let db = Mutex::new(db);

    let server = match Server::build("127.0.0.1:6969", &db) {
        Ok(server) => server,
        Err(err) => {
            eprintln!("[FATAL ERROR] unable to run server: {}", err);
            return;
        }
    };

    server.run()
}
