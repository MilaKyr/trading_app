use crate::errors::ClientError;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Product {
    Apple,
    Pear,
    Tomato,
    Potato,
    Onion,
}

impl FromStr for Product {
    type Err = ClientError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "APPLE" => Ok(Product::Apple),
            "PEAR" => Ok(Product::Pear),
            "TOMATO" => Ok(Product::Tomato),
            "POTATO" => Ok(Product::Potato),
            "ONION" => Ok(Product::Onion),
            _ => Err(ClientError::UnknownProduct),
        }
    }
}

impl Display for Product {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self {
            Product::Apple => write!(f, "APPLE"),
            Product::Pear => write!(f, "PEAR"),
            Product::Tomato => write!(f, "TOMATO"),
            Product::Potato => write!(f, "POTATO"),
            Product::Onion => write!(f, "ONION"),
        }
    }
}
