#![allow(unused_imports)]

use evlog::*;
use evlog::error::EvlogError;
use evlog::event::Event;
use evlog::redact::RedactConfig;
use evlog::sampling::SamplingConfig;
use serde_json::json;

#[test]
fn redact_top_level_builtin_keys() {
    let config = RedactConfig::builder().builtins(true).build();
    let mut event = Event::new(Level::Info);
    event.fields.insert("password".to_string(), json!("s3cret"));
    event.fields.insert("username".to_string(), json!("alice"));
    config.redact_event(&mut event);

    assert_eq!(event.fields["password"], json!("[REDACTED]"));
    assert_eq!(event.fields["username"], json!("alice"));
}

#[test]
fn redact_nested_sensitive_keys() {
    let config = RedactConfig::builder().builtins(true).build();
    let mut event = Event::new(Level::Info);
    event.fields.insert("user".to_string(), json!({
        "name": "Alice",
        "password": "s3cret",
        "api_key": "sk-1234"
    }));
    config.redact_event(&mut event);

    let user = event.fields["user"].as_object().unwrap();
    assert_eq!(user["name"], json!("Alice"));
    assert_eq!(user["password"], json!("[REDACTED]"));
    assert_eq!(user["api_key"], json!("[REDACTED]"));
}

#[test]
fn redact_deeply_nested_keys() {
    let config = RedactConfig::builder().builtins(true).build();
    let mut event = Event::new(Level::Info);
    event.fields.insert("data".to_string(), json!({
        "level1": {
            "level2": {
                "token": "abc123"
            }
        }
    }));
    config.redact_event(&mut event);

    assert_eq!(event.fields["data"]["level1"]["level2"]["token"], json!("[REDACTED]"));
}

#[test]
fn redact_inside_arrays() {
    let config = RedactConfig::builder().builtins(true).build();
    let mut event = Event::new(Level::Info);
    event.fields.insert("items".to_string(), json!([
        {"password": "secret1", "name": "a"},
        {"password": "secret2", "name": "b"}
    ]));
    config.redact_event(&mut event);

    let items = event.fields["items"].as_array().unwrap();
    assert_eq!(items[0]["password"], json!("[REDACTED]"));
    assert_eq!(items[0]["name"], json!("a"));
    assert_eq!(items[1]["password"], json!("[REDACTED]"));
}

#[test]
fn redact_custom_paths() {
    let config = RedactConfig::builder()
        .path("my_secret_field")
        .builtins(false)
        .build();
    let mut event = Event::new(Level::Info);
    event.fields.insert("my_secret_field".to_string(), json!("value"));
    event.fields.insert("safe_field".to_string(), json!("ok"));
    config.redact_event(&mut event);

    assert_eq!(event.fields["my_secret_field"], json!("[REDACTED]"));
    assert_eq!(event.fields["safe_field"], json!("ok"));
}

#[test]
fn redact_by_value_pattern() {
    let config = RedactConfig::builder()
        .pattern("sk_live_")
        .builtins(false)
        .build();
    let mut event = Event::new(Level::Info);
    event.fields.insert("key".to_string(), json!("sk_live_abc123"));
    event.fields.insert("other".to_string(), json!("safe_value"));
    config.redact_event(&mut event);

    assert_eq!(event.fields["key"], json!("[REDACTED]"));
    assert_eq!(event.fields["other"], json!("safe_value"));
}

#[test]
fn redact_nested_value_patterns() {
    let config = RedactConfig::builder()
        .pattern("sk_live_")
        .builtins(false)
        .build();
    let mut event = Event::new(Level::Info);
    event.fields.insert("payment".to_string(), json!({
        "stripe_key": "sk_live_abc123",
        "amount": 42
    }));
    config.redact_event(&mut event);

    assert_eq!(event.fields["payment"]["stripe_key"], json!("[REDACTED]"));
    assert_eq!(event.fields["payment"]["amount"], json!(42));
}

#[test]
fn redact_custom_replacement_string() {
    let config = RedactConfig::builder()
        .builtins(true)
        .replacement("***")
        .build();
    let mut event = Event::new(Level::Info);
    event.fields.insert("password".to_string(), json!("s3cret"));
    config.redact_event(&mut event);

    assert_eq!(event.fields["password"], json!("***"));
}
