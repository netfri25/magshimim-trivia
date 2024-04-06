use trivia::db::SqliteDatabase;
use trivia::server::TriviaServer;

fn main() {
    let db = SqliteDatabase::connect(":memory:").unwrap();
    TriviaServer::build("127.0.0.1:6969", db).unwrap().run();
}
