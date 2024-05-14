use trivia::db::{Database, TurboSqliteDatabase};

mod communicator;
mod defer;

mod server;
use server::Server;

fn main() {
    eprintln!("[INFO] starting...");
    let db = match TurboSqliteDatabase::connect("trivia-db.sqlite") {
        Ok(db) => db,
        Err(err) => {
            eprintln!("[FATAL ERROR] unable to initialize db: {}", err);
            return;
        }
    };

    std::thread::scope(|s| {
        s.spawn(|| {
            if let Err(err) = db.populate_questions(50) {
                eprintln!("[WARN] unable to add questions to db: {}", err);
            } else {
                eprintln!("[INFO] successfully added 50 (or less) questions to the db");
            }
        });

        let server = match Server::build("127.0.0.1:6969", &db) {
            Ok(server) => server,
            Err(err) => {
                eprintln!("[FATAL ERROR] unable to run server: {}", err);
                std::process::exit(1);
            }
        };

        server.run()
    });
}
