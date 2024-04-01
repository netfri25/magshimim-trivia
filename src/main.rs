

mod handler;

mod server;
use crate::server::Server;

mod messages;

fn main() {
    Server::build("127.0.0.1:6969").unwrap().run();
}
