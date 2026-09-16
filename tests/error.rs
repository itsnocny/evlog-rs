#![allow(unused_imports)]

use evlog::*;
use evlog::error::EvlogError;
use evlog::event::Event;
use evlog::redact::RedactConfig;
use evlog::sampling::SamplingConfig;
use serde_json::json;

#[test]
fn evlog_error_display() {
    let err = EvlogError::new("something failed");
    assert_eq!(format!("{}", err), "something failed");
}

#[test]
fn evlog_error_builder_pattern() {
    let err = EvlogError::new("test")
        .code("TEST_ERR")
        .status(422)
        .why("because")
        .fix("do this instead")
        .link("https://docs.example.com");

    assert_eq!(err.code, Some("TEST_ERR".to_string()));
    assert_eq!(err.status, 422);
    assert_eq!(err.why, Some("because".to_string()));
    assert_eq!(err.fix, Some("do this instead".to_string()));
    assert_eq!(err.link, Some("https://docs.example.com".to_string()));
}

#[test]
fn evlog_error_implements_std_error() {
    let err = EvlogError::new("test");
    let _: &dyn std::error::Error = &err;
}

#[test]
fn evlog_error_to_error_info_includes_location() {
    let err = EvlogError::new("test");
    let info = err.to_error_info();
    assert!(info.internal.is_some());
    let internal = info.internal.unwrap();
    assert!(internal.get("location").is_some());
    let loc = internal["location"].as_str().unwrap();
    assert!(loc.contains("error.rs"), "location should point to this file, got: {}", loc);
}

#[test]
fn error_catalog_macro_generates_schema() {
    define_error_catalog! {
        pub enum TestCatalogError {
            NotFound(id: String) => {
                message: format!("Resource {} not found", id),
                code: "NOT_FOUND",
                status: 404
            },
            Unauthorized => {
                message: "Unauthorized",
                code: "UNAUTHORIZED",
                status: 401,
                why: "No token provided",
                fix: "Include a Bearer token"
            }
        }
    }

    assert!(TestCatalogError::SCHEMA_MD.contains("## TestCatalogError"));
    assert!(TestCatalogError::SCHEMA_MD.contains("- `NotFound`"));
    assert!(TestCatalogError::SCHEMA_MD.contains("- `Unauthorized`"));
}

#[test]
fn error_catalog_into_evlog() {
    define_error_catalog! {
        pub enum PaymentError {
            Declined(amount: u32) => {
                message: format!("Payment of {} declined", amount),
                code: "DECLINED",
                status: 402,
                why: "Card declined"
            }
        }
    }

    let err = PaymentError::Declined(99).into_evlog();
    assert_eq!(err.status, 402);
    assert_eq!(err.code, Some("DECLINED".to_string()));
}

#[test]
fn evlog_error_serialize_to_json_response() {
    let err = EvlogError::new("Payment failed")
        .code("PAYMENT_FAILED")
        .status(400)
        .why("Insufficient funds")
        .fix("Use another card");
    
    let json = err.to_json();
    assert!(json.contains(r#""message":"Payment failed""#));
    assert!(json.contains(r#""code":"PAYMENT_FAILED""#));
    assert!(json.contains(r#""status":400"#));
    assert!(json.contains(r#""why":"Insufficient funds""#));
    assert!(json.contains(r#""fix":"Use another card""#));
    assert!(!json.contains("location"), "HTTP JSON should not include location");
    assert!(!json.contains("stack"), "HTTP JSON should not include stack trace");

    let response = err.to_response_body();
    assert!(response.starts_with(r#"{"error":{"message""#));
}
