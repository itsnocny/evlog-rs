#![allow(unused_imports)]

use evlog::*;
use evlog::error::EvlogError;
use evlog::event::Event;
use evlog::redact::RedactConfig;
use evlog::sampling::SamplingConfig;
use serde_json::json;

#[test]
fn event_to_json() {
    let event = Event::simple(Level::Info, "test", "hello");
    let json_str = event.to_json();
    assert!(json_str.contains("\"level\":\"info\""));
    assert!(json_str.contains("\"message\":\"hello\""));
}

#[test]
fn event_fields_serialize_flat() {
    let mut event = Event::new(Level::Info);
    event.fields.insert("custom".to_string(), json!("value"));
    let json_str = event.to_json();
    assert!(json_str.contains("\"custom\":\"value\""));
}

#[test]
fn event_skips_none_fields() {
    let event = Event::new(Level::Info);
    let json_str = event.to_json();
    assert!(!json_str.contains("\"method\""));
    assert!(!json_str.contains("\"path\""));
    assert!(!json_str.contains("\"error\""));
}
