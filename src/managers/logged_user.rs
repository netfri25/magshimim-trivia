
#[derive(PartialEq, Eq, Hash)]
pub struct LoggedUser {
    username: String,
}

impl LoggedUser {
    pub fn new(username: impl Into<String>) -> Self {
        let username = username.into();
        Self { username }
    }

    pub fn username(&self) -> &str {
        &self.username
    }
}


impl PartialEq<str> for LoggedUser {
    fn eq(&self, other: &str) -> bool {
        self.username() == other
    }
}
