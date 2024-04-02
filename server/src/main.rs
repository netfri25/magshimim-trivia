use trivia::db::SqliteDatabase;
use trivia::server::Server;

fn main() {
    let db = SqliteDatabase::connect(":memory:").unwrap();
    Server::build("127.0.0.1:6969", db).unwrap().run();
}
