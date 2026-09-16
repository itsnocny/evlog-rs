#![allow(unused_imports)]

use evlog::*;
use evlog::error::EvlogError;
use evlog::event::Event;
use evlog::redact::RedactConfig;
use evlog::sampling::SamplingConfig;
use serde_json::json;

#[test]
fn level_ordering() {
    assert!(Level::Debug < Level::Info);
    assert!(Level::Info < Level::Warn);
    assert!(Level::Warn < Level::Error);
}

#[test]
fn level_from_str() {
    use std::str::FromStr;
    assert_eq!(Level::from_str("debug").unwrap(), Level::Debug);
    assert_eq!(Level::from_str("INFO").unwrap(), Level::Info);
    assert_eq!(Level::from_str("warning").unwrap(), Level::Warn);
    assert_eq!(Level::from_str("error").unwrap(), Level::Error);
    assert!(Level::from_str("invalid").is_err());
}

#[test]
fn level_display() {
    assert_eq!(format!("{}", Level::Info), "info");
    assert_eq!(format!("{}", Level::Error), "error");
}
