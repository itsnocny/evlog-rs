use tracing_core::{Event, Subscriber, Field};
use tracing_subscriber::layer::Context;
use tracing_subscriber::Layer;
use crate::level::Level as EvlogLevel;
use serde_json::{Map, Value};

/// A layer that intercepts `tracing` events and funnels them into `evlog` as simple logs.
pub struct TracingBridge;

impl<S: Subscriber> Layer<S> for TracingBridge {
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let evlog_level = match *event.metadata().level() {
            tracing_core::Level::TRACE => EvlogLevel::Debug,
            tracing_core::Level::DEBUG => EvlogLevel::Debug,
            tracing_core::Level::INFO => EvlogLevel::Info,
            tracing_core::Level::WARN => EvlogLevel::Warn,
            tracing_core::Level::ERROR => EvlogLevel::Error,
        };

        let mut visitor = EvlogVisitor(Map::new());
        event.record(&mut visitor);

        let mut message = None;
        if let Some(msg) = visitor.0.remove("message") {
            if let Value::String(s) = msg {
                message = Some(s);
            } else {
                message = Some(msg.to_string());
            }
        }

        crate::simple::log_from_tracing(
            evlog_level,
            event.metadata().target(),
            message.as_deref().unwrap_or(""),
            visitor.0,
        );
    }
}

struct EvlogVisitor(Map<String, Value>);

impl tracing_core::field::Visit for EvlogVisitor {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        self.0.insert(field.name().to_string(), Value::String(format!("{:?}", value)));
    }
    fn record_f64(&mut self, field: &Field, value: f64) {
        self.0.insert(field.name().to_string(), serde_json::json!(value));
    }
    fn record_i64(&mut self, field: &Field, value: i64) {
        self.0.insert(field.name().to_string(), serde_json::json!(value));
    }
    fn record_u64(&mut self, field: &Field, value: u64) {
        self.0.insert(field.name().to_string(), serde_json::json!(value));
    }
    fn record_bool(&mut self, field: &Field, value: bool) {
        self.0.insert(field.name().to_string(), serde_json::json!(value));
    }
    fn record_str(&mut self, field: &Field, value: &str) {
        self.0.insert(field.name().to_string(), Value::String(value.to_string()));
    }
}
