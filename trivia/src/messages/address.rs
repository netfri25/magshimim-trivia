use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Address {
    city: Box<str>,
    street: Box<str>,
    apartment: u64,
}

impl Address {
    pub fn new(city: impl Into<Box<str>>, street: impl Into<Box<str>>, apartment: u64) -> Self {
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

    pub fn apartment(&self) -> u64 {
        self.apartment
    }
}

impl ToString for Address {
    fn to_string(&self) -> String {
        format!("{}, {} {}", self.city, self.street, self.apartment)
    }
}
