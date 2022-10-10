use crate::errors::ClientError;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ClientActions {
    Buy,
    Sell,
}

impl FromStr for ClientActions {
    type Err = ClientError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "BUY" => Ok(ClientActions::Buy),
            "SELL" => Ok(ClientActions::Sell),
            _ => Err(ClientError::UnknownAction),
        }
    }
}

impl Display for ClientActions {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self {
            ClientActions::Buy => write!(f, "BUY"),
            ClientActions::Sell => write!(f, "SELL"),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum ServerActions {
    Trade,
    Ack,
}

impl Display for ServerActions {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self {
            ServerActions::Trade => write!(f, "TRADE"),
            ServerActions::Ack => write!(f, "ACK"),
        }
    }
}
