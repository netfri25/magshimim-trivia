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
}

impl ToString for Address {
    fn to_string(&self) -> String {
        format!("{}, {} {}", self.city, self.street, self.apartment)
    }
}
