mod server;
use crate::server::Server;

mod db;
use crate::db::SqliteDatabase;

mod messages;
mod managers;
mod handler;

fn main() {
    let db = SqliteDatabase::connect(":memory:").unwrap();
    Server::build("127.0.0.1:6969", db).unwrap().run();
}
