use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Address {
    city: Box<str>,
    street: Box<str>,
    apartment: u32,
}

impl Address {
    pub fn new(city: impl Into<Box<str>>, street: impl Into<Box<str>>, apartment: u32) -> Self {
        let city = city.into();
        let street = street.into();
        Self {
            city,
            street,
            apartment,
        }
    }

    pub fn city(&self) -> &str {
        &self.city
    }

    pub fn street(&self) -> &str {
        &self.street
    }

    pub fn apartment(&self) -> u32 {
        self.apartment
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}, {} {}", self.city, self.street, self.apartment)
    }
}
