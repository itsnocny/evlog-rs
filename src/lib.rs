//! # evlog
//!
//! Structured logging with wide events for Rust.
//!
//! Accumulate context over any unit of work (HTTP request, background job, script)
//! and emit a single, comprehensive log event.
//!
//! ## Quick Start
//!
//! ```rust
//! use evlog::{init_logger, Config, EventLogger, json};
//! use evlog::simple::log;
//!
//! init_logger(Config::builder().service("my-app").pretty(true).build());
//!
//! // Simple logging
//! log::info("app", "Server started");
//!
//! // Wide events
//! let mut ev = EventLogger::new();
//! ev.set("user", json!({"id": 1, "plan": "pro"}));
//! ev.emit();
//! ```

pub mod level;
pub mod env;
pub mod event;
pub mod config;
pub mod drain;
pub mod pretty;
pub mod redact;
pub mod sampling;
pub mod error;
pub mod logger;
pub mod simple;
pub mod frameworks;

pub use level::Level;
pub use event::{Event, ErrorInfo, Fields};
pub use config::{Config, ConfigBuilder, init_logger, global_config, shutdown};
pub use drain::{Drain, ConsoleDrain, BatchedDrain, FanoutDrain};
pub use env::EnvContext;
pub use error::{EvlogError, ParsedError};
pub use logger::{EventLogger, LoggerHandle};
pub use sampling::{SamplingConfig, KeepRule};
pub use redact::RedactConfig;

/// Re-export serde_json::json! for convenience.
pub use serde_json::json;
