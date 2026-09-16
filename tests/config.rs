#![allow(unused_imports)]

use evlog::*;
use evlog::error::EvlogError;
use evlog::event::Event;
use evlog::redact::RedactConfig;
use evlog::sampling::SamplingConfig;
use serde_json::json;

#[test]
fn middleware_exclude_works() {
    let config = Config::builder()
        .exclude_route("/health")
        .exclude_route("/metrics")
        .build();
    assert!(!config.middleware.should_log("/health"));
    assert!(!config.middleware.should_log("/metrics"));
    assert!(config.middleware.should_log("/api/users"));
}

#[test]
fn middleware_include_works() {
    let config = Config::builder()
        .include_route("/api/**")
        .build();
    assert!(config.middleware.should_log("/api/users"));
    assert!(!config.middleware.should_log("/health"));
}

#[test]
fn global_config_returns_default_without_panic() {
    let c = global_config();
    assert!(c.enabled);
}
