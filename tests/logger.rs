#![allow(unused_imports)]

use evlog::*;
use evlog::error::EvlogError;
use evlog::event::Event;
use evlog::redact::RedactConfig;
use evlog::sampling::SamplingConfig;
use serde_json::json;

#[test]
fn set_metric_does_not_panic_on_non_object() {
    let logger = EventLogger::new();
    let handle = LoggerHandle::new(logger);
    handle.set("metrics", json!(42));
    handle.track("some_op", || {
        std::thread::sleep(std::time::Duration::from_millis(1));
    });
}

#[test]
fn set_metric_works_normally() {
    init_logger(Config::builder().service("test").silent(true).build());
    let logger = EventLogger::new();
    let handle = LoggerHandle::new(logger);
    handle.track("db_query", || {
        std::thread::sleep(std::time::Duration::from_millis(1));
    });
    
    let event = handle.emit();
    if let Some(ev) = event {
        let metrics = ev.fields.get("metrics").unwrap();
        assert!(metrics.get("db_query").is_some());
        assert!(metrics["db_query"].as_f64().unwrap() >= 1.0);
    }
}

#[test]
fn logger_set_after_emit_is_noop() {
    init_logger(Config::builder().service("test").silent(true).build());
    let mut logger = EventLogger::new();
    logger.emit();
    logger.set("key", json!("value"));
}

#[test]
fn logger_fork_retains_context() {
    init_logger(Config::builder().service("test").silent(true).build());
    let logger = EventLogger::new();
    let handle = LoggerHandle::new(logger);
    handle.set("tenant", json!("acme"));
    
    let child_handle = handle.fork();
    let child_event = child_handle.emit();
    
    if let Some(ev) = child_event {
        assert_eq!(ev.fields.get("tenant"), Some(&json!("acme")));
        assert!(ev.method.is_none());
    }
}

#[test]
fn logger_skip_emit_returns_none() {
    init_logger(Config::builder().service("test").silent(true).build());
    let mut logger = EventLogger::new();
    logger.skip_emit = true;
    assert!(logger.emit().is_none());
}
