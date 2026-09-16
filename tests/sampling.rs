#![allow(unused_imports)]

use evlog::*;
use evlog::error::EvlogError;
use evlog::event::Event;
use evlog::redact::RedactConfig;
use evlog::sampling::SamplingConfig;
use serde_json::json;

#[test]
fn sampling_default_rate_keeps_all() {
    let config = SamplingConfig::builder().build();
    let event = Event::new(Level::Info);
    assert!(config.should_sample(&event));
}

#[test]
fn sampling_rate_zero_drops_all() {
    let config = SamplingConfig::builder()
        .rate(Level::Debug, 0)
        .build();
    let event = Event::new(Level::Debug);
    assert!(!config.should_sample(&event));
}

#[test]
fn sampling_rate_100_keeps_all() {
    let config = SamplingConfig::builder()
        .rate(Level::Info, 100)
        .build();
    let event = Event::new(Level::Info);
    assert!(config.should_sample(&event));
}

#[test]
fn sampling_keep_status_overrides_rate() {
    let config = SamplingConfig::builder()
        .rate(Level::Info, 0)
        .keep_status(500)
        .build();
    let mut event = Event::new(Level::Info);
    event.status = Some(503);
    assert!(config.should_sample(&event));
}

#[test]
fn sampling_keep_duration_overrides_rate() {
    let config = SamplingConfig::builder()
        .rate(Level::Info, 0)
        .keep_duration(1000)
        .build();
    let mut event = Event::new(Level::Info);
    event.duration_ms = Some(1500.0);
    assert!(config.should_sample(&event));
}

#[test]
fn sampling_keep_path_glob_precompiled() {
    let config = SamplingConfig::builder()
        .rate(Level::Info, 0)
        .keep_path("/api/payments/**")
        .build();

    let mut event = Event::new(Level::Info);
    event.path = Some("/api/payments/checkout".to_string());
    assert!(config.should_sample(&event));

    let mut event2 = Event::new(Level::Info);
    event2.path = Some("/api/users/1".to_string());
    assert!(!config.should_sample(&event2));
}

#[test]
fn sampling_status_threshold() {
    let config = SamplingConfig::builder()
        .rate(Level::Info, 0)
        .keep_status(400)
        .build();

    let mut event = Event::new(Level::Info);
    event.status = Some(200);
    assert!(!config.should_sample(&event));

    event.status = Some(400);
    assert!(config.should_sample(&event));

    event.status = Some(500);
    assert!(config.should_sample(&event));
}
