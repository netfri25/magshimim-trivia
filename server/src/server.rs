use std::net::ToSocketAddrs;

use crate::communicator::{self, Communicator};

use trivia::db::{self, Database};
use trivia::handler::RequestHandlerFactory;

pub struct Server<'db> {
    comm: Communicator<'db>,
}

impl<'db, 'me: 'db> Server<'db> {
    pub fn build(addr: impl ToSocketAddrs, db: &'db (impl Database + Sync)) -> Result<Self, Error> {
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
    use std::sync::{mpsc, OnceLock};

    use trivia::db::question::QuestionData;
    use trivia::db::SqliteDatabase;
    use trivia::managers::login;
    use trivia::messages::{Request, Response};

    use super::*;

    const ADDR: &str = "127.0.0.1:1234";

    static START_SERVER: OnceLock<()> = OnceLock::new();

    fn start_server() {
        START_SERVER.get_or_init(|| {
            let (send, recv) = mpsc::sync_channel(1);

            std::thread::spawn(move || {
                let db = SqliteDatabase::connect(":memory:").unwrap();
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
        let username = "user1234".to_string();
        let password = "pass1234".to_string();
        let request = Request::Login {
            username: username.clone(),
            password,
        };

        request.write_to(&mut client).unwrap();
        let response = Response::read_from(&mut client).unwrap();
        let expected = Response::new_error(login::Error::UserDoesntExist(username));

        assert_eq!(response, expected);
    }

    #[test]
    fn signup_login() {
        start_server();

        let mut client = TcpStream::connect(ADDR).unwrap();

        let request = Request::Signup {
            username: "signup1234".to_string(),
            password: "pass1234".to_string(),
            email: "email@example.com".to_string(),
        };
        request.write_to(&mut client).unwrap();
        let response = Response::read_from(&mut client).unwrap();
        let expected = Response::Signup;
        assert_eq!(response, expected);

        let request = Request::Login {
            username: "signup1234".to_string(),
            password: "pass1234".to_string(),
        };
        request.write_to(&mut client).unwrap();
        let response = Response::read_from(&mut client).unwrap();
        let expected = Response::Login;
        assert_eq!(response, expected);
    }

    #[test]
    fn signup_signup() {
        start_server();

        let mut client = TcpStream::connect(ADDR).unwrap();
        let username = "double".to_string();

        let request = Request::Signup {
            username: username.clone(),
            password: "pass1234".to_string(),
            email: "email@example.com".to_string(),
        };
        request.write_to(&mut client).unwrap();
        let response = Response::read_from(&mut client).unwrap();
        let expected = Response::Signup;
        assert_eq!(response, expected);

        let request = Request::Signup {
            username: username.clone(),
            password: "pass1234".to_string(),
            email: "email@example.com".to_string(),
        };
        request.write_to(&mut client).unwrap();
        let response = Response::read_from(&mut client).unwrap();
        let expected = Response::new_error(login::Error::UserAlreadyExists(username));
        assert_eq!(response, expected);
    }

    #[test]
    fn signup_login_logut_login() {
        start_server();

        let mut client = TcpStream::connect(ADDR).unwrap();

        let request = Request::Signup {
            username: "multiple-login".to_string(),
            password: "pass1234".to_string(),
            email: "email@example.com".to_string(),
        };
        request.write_to(&mut client).unwrap();
        let response = Response::read_from(&mut client).unwrap();
        let expected = Response::Signup;
        assert_eq!(response, expected);

        let request = Request::Login {
            username: "multiple-login".to_string(),
            password: "pass1234".to_string(),
        };
        request.write_to(&mut client).unwrap();
        let response = Response::read_from(&mut client).unwrap();
        let expected = Response::Login;
        assert_eq!(response, expected);

        // disconnect from the server
        drop(client);

        let mut client = TcpStream::connect(ADDR).unwrap();

        let request = Request::Login {
            username: "multiple-login".to_string(),
            password: "pass1234".to_string(),
        };
        request.write_to(&mut client).unwrap();
        let response = Response::read_from(&mut client).unwrap();
        let expected = Response::Login;
        assert_eq!(response, expected);
    }

    #[test]
    fn signup_login_login() {
        start_server();

        let mut client = TcpStream::connect(ADDR).unwrap();
        let username = "login-abuser".to_string();

        let request = Request::Signup {
            username: username.clone(),
            password: "pass1234".to_string(),
            email: "email@example.com".to_string(),
        };
        request.write_to(&mut client).unwrap();
        let response = Response::read_from(&mut client).unwrap();
        let expected = Response::Signup;
        assert_eq!(response, expected);

        let request = Request::Login {
            username: username.clone(),
            password: "pass1234".to_string(),
        };
        request.write_to(&mut client).unwrap();
        let response = Response::read_from(&mut client).unwrap();
        let expected = Response::Login;
        assert_eq!(response, expected);

        let mut client = TcpStream::connect(ADDR).unwrap();

        let request = Request::Login {
            username: username.clone(),
            password: "pass1234".to_string(),
        };
        request.write_to(&mut client).unwrap();
        let response = Response::read_from(&mut client).unwrap();
        let expected = Response::new_error(login::Error::UserAlreadyConnected(username));
        assert_eq!(response, expected);
    }

    #[test]
    fn create_question() {
        start_server();

        let mut client = TcpStream::connect(ADDR).unwrap();
        let username = "creator";
        let password = "pass";
        let signup = Request::Signup {
            username: username.to_string(),
            password: password.to_string(),
            email: "".to_string(),
        };

        signup.write_to(&mut client).unwrap();
        let response = Response::read_from(&mut client).unwrap();
        let expected = Response::Signup;
        assert_eq!(response, expected);

        let login = Request::Login {
            username: username.to_string(),
            password: password.to_string(),
        };

        login.write_to(&mut client).unwrap();
        let response = Response::read_from(&mut client).unwrap();
        let expected = Response::Login;
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
        let expected = Response::CreateQuestion;
        assert_eq!(response, expected);

        // the same question as before, should already exist
        create_question.write_to(&mut client).unwrap();
        let response = Response::read_from(&mut client).unwrap();
        let expected = Response::new_error("question already exists");
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
