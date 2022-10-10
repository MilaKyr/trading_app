use crate::consts::{SHOW_LOG_LEVEL, SHOW_LOG_TARGET};
use crate::errors::Error;
use env_logger::{Builder, Target};
use log::LevelFilter;
use std::net::IpAddr;

pub fn init_logs() {
    Builder::from_default_env()
        .filter_level(LevelFilter::Info)
        .target(Target::Stdout)
        .format_target(SHOW_LOG_TARGET)
        .format_level(SHOW_LOG_LEVEL)
        .format_timestamp_secs()
        .init()
}

pub fn get_greeting_message(ip: IpAddr, port: u16) -> Result<String, Error> {
    let msg = format!("connected ('{}', '{}')", ip, port);
    Ok(msg)
}

pub fn split_at_colon(message: &str) -> Option<(String, String)> {
    let message: &str = message.split("\n").next()?;
    let message: String = message.trim().to_string();
    message.find(":").map(|index| {
        let part1 = message[0..index].trim().to_string();
        let part2 = message[index + 1..].trim().to_string();
        (part1, part2)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_at_colon() {
        let s = "buy:apple";
        match split_at_colon(s) {
            Some((part1, part2)) => {
                assert_eq!(part1, "buy");
                assert_eq!(part2, "apple");
            }
            None => assert!(false),
        }
    }

    #[test]
    fn test_split_at_colon_with_spaces() {
        let s = "buy : apple  ";
        match split_at_colon(s) {
            Some((part1, part2)) => {
                assert_eq!(part1, "buy");
                assert_eq!(part2, "apple");
            }
            None => assert!(false),
        }
    }

    #[test]
    fn test_split_at_colon_with_newline() {
        let s = "buy : apple  \n onion";
        match split_at_colon(s) {
            Some((part1, part2)) => {
                assert_eq!(part1, "buy");
                assert_eq!(part2, "apple");
            }
            None => assert!(false),
        }
    }

    #[test]
    fn test_split_at_colon_no_action() {
        let s = "apple  ";
        assert!(split_at_colon(s).is_none());
    }

    #[test]
    fn test_split_at_colon_no_action_newline() {
        let s = "buy\n: apple";
        assert!(split_at_colon(s).is_none());
    }

    #[test]
    fn test_split_at_colon_no_action_newline_before_colon() {
        let s = "buy:\n apple";
        match split_at_colon(s) {
            Some((part1, part2)) => {
                assert_eq!(part1, "buy");
                assert_eq!(part2, "");
            }
            None => assert!(false),
        }
    }
}
