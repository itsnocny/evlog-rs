#![allow(unused_imports)]

use evlog::*;
use evlog::error::EvlogError;
use evlog::event::Event;
use evlog::redact::RedactConfig;
use evlog::sampling::SamplingConfig;
use serde_json::json;

#[test]
fn pretty_simple_event() {
    let event = Event::simple(Level::Info, "app", "Server started");
    let output = evlog::pretty::format_event(&event);
    assert!(output.contains("Server started"));
    assert!(output.contains("app"));
}

#[test]
fn pretty_wide_event_includes_fields() {
    let mut event = Event::new(Level::Info);
    event.method = Some("POST".to_string());
    event.path = Some("/api/checkout".to_string());
    event.fields.insert("user".to_string(), json!({"id": 1}));
    let output = evlog::pretty::format_event(&event);
    assert!(output.contains("POST"));
    assert!(output.contains("/api/checkout"));
    assert!(output.contains("user"));
}

#[test]
fn format_value_inline_primitives() {
    use evlog::pretty::format_value_inline;
    assert_eq!(format_value_inline(&json!(null)), "null");
    assert_eq!(format_value_inline(&json!(true)), "true");
    assert_eq!(format_value_inline(&json!(42)), "42");
    assert_eq!(format_value_inline(&json!("hello")), "hello");
}
