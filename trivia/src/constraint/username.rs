use core::fmt;
use std::ops::Deref;
use std::str::FromStr;

use fancy_regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Hash, Eq)]
pub struct Username(Box<str>);

impl Username {
    pub fn create(text: impl Into<Box<str>>) -> Result<Self, Error> {
        // I can unwrap here because I already know at compile time that the regex that I've
        // entered is a correct regex
        let regex = Regex::new(super::USERNAME.regex).unwrap();
        let text = text.into();
        regex
            .is_match(&text)
            .is_ok_and(|b| b)
            .then(|| Self(text))
            .ok_or(Error(super::USERNAME.error.into()))
    }
}

impl FromStr for Username {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::create(s)
    }
}

impl AsRef<str> for Username {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl Deref for Username {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for Username {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, thiserror::Error)]
#[error("{0}")]
pub struct Error(String);
