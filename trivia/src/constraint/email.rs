use std::fmt;
use std::ops::Deref;
use std::str::FromStr;

use fancy_regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Email(Box<str>);

impl Email {
    pub fn create(text: impl Into<Box<str>>) -> Result<Self, Error> {
        // I can unwrap here because I already know at compile time that the regex that I've
        // entered is a correct regex
        let regex = Regex::new(super::EMAIL.regex).unwrap();
        let text = text.into();
        regex
            .is_match(&text)
            .is_ok_and(|b| b)
            .then(|| Self(text))
            .ok_or(Error(super::EMAIL.error.into()))
    }
}

impl FromStr for Email {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::create(s)
    }
}

impl AsRef<str> for Email {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl Deref for Email {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for Email {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, thiserror::Error)]
#[error("{0}")]
pub struct Error(String);
