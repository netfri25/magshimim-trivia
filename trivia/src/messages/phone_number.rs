use std::fmt;
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

impl fmt::Display for PhoneNumber {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}-{}", self.prefix, self.number)
    }
}

impl FromStr for PhoneNumber {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some((prefix, number)) = s.split_once('-') else {
            return Err(Error::NoPrefix);
        };

        if !prefix.starts_with('0') {
            return Err(Error::NonZeroPrefix);
        }

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

        if number.len() != 7 {
            return Err(Error::InvalidNumberLength);
        }

        let prefix = prefix.into();
        let number = number.into();
        Ok(PhoneNumber { prefix, number })
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, thiserror::Error)]
pub enum Error {
    #[error("unable to find the phone prefix")]
    NoPrefix,

    #[error("phone prefix should start with the digit 0")]
    NonZeroPrefix,

    #[error("phone prefix should be between 2 to 3 digits")]
    InvalidPrefixLength,

    #[error("phone number should only contain digits (except for the prefix seperator)")]
    UnexpectedCharacter,

    #[error("number should be 7 digits (not including the prefix)")]
    InvalidNumberLength,
}
