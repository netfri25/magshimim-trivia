use trivia::db::SqliteDatabase;
use trivia::server::TriviaServer;

fn main() {
    let db = SqliteDatabase::connect("trivia-db.sqlite").unwrap();
    TriviaServer::build("127.0.0.1:6969", db).unwrap().run();
}
