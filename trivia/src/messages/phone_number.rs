use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// phone number should have one of the format <prefix>-<number>
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct PhoneNumber {
    prefix: Box<str>,
    number: Box<str>,
}

impl PhoneNumber {
    pub fn prefix(&self) -> &str {
        &self.prefix
    }

    pub fn number(&self) -> &str {
        &self.number
    }
}

impl ToString for PhoneNumber {
    fn to_string(&self) -> String {
        format!("{}-{}", self.prefix, self.number)
    }
}

impl FromStr for PhoneNumber {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some((prefix, number)) = s.split_once('-') else {
            return Err(Error::NoPrefix);
        };

        if !prefix
            .chars()
            .chain(number.chars())
            .all(|c| c.is_ascii_digit())
        {
            return Err(Error::UnexpectedCharacter);
        }

        if !(2..=3).contains(&prefix.len()) {
            return Err(Error::InvalidPrefixLength);
        }

        let prefix = prefix.into();
        let number = number.into();
        Ok(PhoneNumber { prefix, number })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("unable to find the phone prefix")]
    NoPrefix,

    #[error("phone prefix should be between 2 to 3 digits")]
    InvalidPrefixLength,

    #[error("phone number should only contain digits (except for the prefix seperator)")]
    UnexpectedCharacter,
}
