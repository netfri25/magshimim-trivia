use std::net::ToSocketAddrs;

use crate::communicator::{self, Communicator};

use trivia::db::{self, Database};
use trivia::handler::RequestHandlerFactory;

pub struct Server<'db, DB: ?Sized> {
    comm: Communicator<'db, DB>,
}

impl<'db, 'me: 'db, DB> Server<'db, DB>
where
    DB: Database + Sync + ?Sized,
{
    pub fn build(addr: impl ToSocketAddrs, db: &'db DB) -> Result<Self, Error> {
        let factory = RequestHandlerFactory::new(db);
        let comm = Communicator::build(addr, factory)?;
        Ok(Self { comm })
    }

    pub fn run(&'me self) {
        std::thread::scope(|scope| {
            scope.spawn(|| self.comm.start_handle_requests());

            let mut line = String::new();
            loop {
                line.clear();
                if let Err(err) = std::io::stdin().read_line(&mut line) {
                    eprintln!("[ERROR] stdin error: {}", err);
                    continue;
                }

                let cmd = line.trim().to_lowercase();

                match cmd.as_str() {
                    "exit" => std::process::exit(0),
                    _ => eprintln!("Unknown command: {:?}", cmd),
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use std::net::TcpStream;
    use std::path::Path;
    use std::sync::{mpsc, OnceLock};

    use trivia::db::question::QuestionData;
    use trivia::db::TurboSqliteDatabase;
    use trivia::handler::login::Error::*;
    use trivia::handler::menu::Error::*;
    use trivia::managers::login::Error::*;
    use trivia::messages::{Address, Request, Response, DATE_FORMAT};
    use trivia::NaiveDate;

    use super::*;

    const ADDR: &str = "127.0.0.1:1234";

    static START_SERVER: OnceLock<()> = OnceLock::new();

    fn start_server() {
        START_SERVER.get_or_init(|| {
            let (send, recv) = mpsc::sync_channel(1);

            std::thread::spawn(move || {
                let path = Path::new("test-db.sqlite");
                if path.exists() {
                    std::fs::remove_file(path).ok();
                    std::fs::remove_file(path.with_extension("sqlite-shm")).ok();
                    std::fs::remove_file(path.with_extension("sqlite-wal")).ok();
                }
                let db = TurboSqliteDatabase::connect(path).unwrap();
                let server = Server::build(ADDR, &db).unwrap();
                // notify that the server has started running
                send.send(()).unwrap();
                server.run();
            });

            // wait until the server starts
            recv.recv().unwrap();
        });
    }

    #[test]
    fn login_without_signup() {
        start_server();

        let mut client = TcpStream::connect(ADDR).unwrap();
        let username = "user1234".into();
        let password = "Pass@123".into();
        let request = Request::Login { username, password };

        request.write_to(&mut client).unwrap();
        let response = Response::read_from(&mut client).unwrap();
        let expected = Response::Login(Err(Manager(UserDoesntExist("user1234".parse().unwrap()))));

        assert_eq!(response, expected);
    }

    #[test]
    fn signup_login() {
        start_server();

        let mut client = TcpStream::connect(ADDR).unwrap();

        let request = Request::Signup {
            username: "signup1234".into(),
            password: "Pass@123".into(),
            email: "email@example.com".into(),
            phone: "050-1122333".into(),
            address: Address::new("Netanya", "Alonim", 69),
            birth_date: NaiveDate::parse_from_str("22/04/2038", DATE_FORMAT).unwrap(),
        };
        request.write_to(&mut client).unwrap();
        let response = Response::read_from(&mut client).unwrap();
        let expected = Response::Signup(Ok(()));
        assert_eq!(response, expected);

        let request = Request::Login {
            username: "signup1234".into(),
            password: "Pass@123".into(),
        };
        request.write_to(&mut client).unwrap();
        let response = Response::read_from(&mut client).unwrap();
        let expected = Response::Login(Ok(()));
        assert_eq!(response, expected);
    }

    #[test]
    fn signup_signup() {
        start_server();

        let mut client = TcpStream::connect(ADDR).unwrap();

        let request = Request::Signup {
            username: "double".into(),
            password: "Pass@123".into(),
            email: "email@example.com".into(),
            phone: "050-1122333".into(),
            address: Address::new("Netanya", "Alonim", 69),
            birth_date: NaiveDate::parse_from_str("22/04/2038", DATE_FORMAT).unwrap(),
        };
        request.write_to(&mut client).unwrap();
        let response = Response::read_from(&mut client).unwrap();
        let expected = Response::Signup(Ok(()));
        assert_eq!(response, expected);

        let request = Request::Signup {
            username: "double".into(),
            password: "Pass@123".into(),
            email: "email@example.com".into(),
            phone: "050-1122333".into(),
            address: Address::new("Netanya", "Alonim", 69),
            birth_date: NaiveDate::parse_from_str("22/04/2038", DATE_FORMAT).unwrap(),
        };
        request.write_to(&mut client).unwrap();
        let response = Response::read_from(&mut client).unwrap();
        let expected = Response::Signup(Err(Manager(UserAlreadyExists("double".parse().unwrap()))));
        assert_eq!(response, expected);
    }

    #[test]
    fn signup_login_logut_login() {
        start_server();

        let mut client = TcpStream::connect(ADDR).unwrap();

        let request = Request::Signup {
            username: "multipleLogin".into(),
            password: "Pass@123".into(),
            email: "email@example.com".into(),
            phone: "050-1122333".into(),
            address: Address::new("Netanya", "Alonim", 69),
            birth_date: NaiveDate::parse_from_str("22/04/2038", DATE_FORMAT).unwrap(),
        };
        request.write_to(&mut client).unwrap();
        let response = Response::read_from(&mut client).unwrap();
        let expected = Response::Signup(Ok(()));
        assert_eq!(response, expected);

        let request = Request::Login {
            username: "multipleLogin".into(),
            password: "Pass@123".into(),
        };
        request.write_to(&mut client).unwrap();
        let response = Response::read_from(&mut client).unwrap();
        let expected = Response::Login(Ok(()));
        assert_eq!(response, expected);

        // disconnect from the server
        drop(client);

        let mut client = TcpStream::connect(ADDR).unwrap();

        let request = Request::Login {
            username: "multipleLogin".into(),
            password: "Pass@123".into(),
        };
        request.write_to(&mut client).unwrap();
        let response = Response::read_from(&mut client).unwrap();
        let expected = Response::Login(Ok(()));
        assert_eq!(response, expected);
    }

    #[test]
    fn signup_login_login() {
        start_server();

        let mut client = TcpStream::connect(ADDR).unwrap();

        let request = Request::Signup {
            username: "loginAbuser".into(),
            password: "Pass@123".into(),
            email: "email@example.com".into(),
            phone: "050-1122333".into(),
            address: Address::new("Netanya", "Alonim", 69),
            birth_date: NaiveDate::parse_from_str("22/04/2038", DATE_FORMAT).unwrap(),
        };
        request.write_to(&mut client).unwrap();
        let response = Response::read_from(&mut client).unwrap();
        let expected = Response::Signup(Ok(()));
        assert_eq!(response, expected);

        let request = Request::Login {
            username: "loginAbuser".into(),
            password: "Pass@123".into(),
        };
        request.write_to(&mut client).unwrap();
        let response = Response::read_from(&mut client).unwrap();
        let expected = Response::Login(Ok(()));
        assert_eq!(response, expected);

        let mut client = TcpStream::connect(ADDR).unwrap();

        let request = Request::Login {
            username: "loginAbuser".into(),
            password: "Pass@123".into(),
        };
        request.write_to(&mut client).unwrap();
        let response = Response::read_from(&mut client).unwrap();
        let expected = Response::Login(Err(Manager(UserAlreadyConnected(
            "loginAbuser".parse().unwrap(),
        ))));
        assert_eq!(response, expected);
    }

    #[test]
    fn create_question() {
        start_server();

        let mut client = TcpStream::connect(ADDR).unwrap();
        let signup = Request::Signup {
            username: "creator".into(),
            password: "Pass@123".into(),
            email: "email@example.com".into(),
            phone: "050-1122333".into(),
            address: Address::new("Netanya", "Alonim", 69),
            birth_date: NaiveDate::parse_from_str("22/04/2038", DATE_FORMAT).unwrap(),
        };

        signup.write_to(&mut client).unwrap();
        let response = Response::read_from(&mut client).unwrap();
        let expected = Response::Signup(Ok(()));
        assert_eq!(response, expected);

        let login = Request::Login {
            username: "creator".into(),
            password: "Pass@123".into(),
        };

        login.write_to(&mut client).unwrap();
        let response = Response::read_from(&mut client).unwrap();
        let expected = Response::Login(Ok(()));
        assert_eq!(response, expected);

        let question = "9 + 10 = ?".to_string();
        let answers = vec![
            "twelfsh".to_string(),
            "21".to_string(),
            "19".to_string(),
            "yes".to_string(),
        ];
        let correct_answer_index = answers.iter().position(|s| s == "21").unwrap();
        let question_data = QuestionData::new(question, answers, correct_answer_index);
        let create_question = Request::CreateQuestion(question_data);

        // shouldn't exists, new question
        create_question.write_to(&mut client).unwrap();
        let response = Response::read_from(&mut client).unwrap();
        let expected = Response::CreateQuestion(Ok(()));
        assert_eq!(response, expected);

        // the same question as before, should already exist
        create_question.write_to(&mut client).unwrap();
        let response = Response::read_from(&mut client).unwrap();
        let expected = Response::CreateQuestion(Err(QuestionAlreadyExists));
        assert_eq!(response, expected);
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Communicator(#[from] communicator::Error),

    #[error(transparent)]
    DBError(#[from] db::Error),
}
