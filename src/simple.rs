use crate::level::Level;
use crate::event::Event;
use crate::config::global_config;
use serde_json::Value;

pub mod log {
    use super::*;

    pub fn info(tag: &str, message: &str) {
        emit_simple(Level::Info, Some(tag), Some(message), None);
    }

    pub fn warn(tag: &str, message: &str) {
        emit_simple(Level::Warn, Some(tag), Some(message), None);
    }

    pub fn error(tag: &str, message: &str) {
        emit_simple(Level::Error, Some(tag), Some(message), None);
    }

    pub fn debug(tag: &str, message: &str) {
        emit_simple(Level::Debug, Some(tag), Some(message), None);
    }

    pub fn info_event(fields: Value) {
        emit_simple(Level::Info, None, None, Some(fields));
    }

    pub fn warn_event(fields: Value) {
        emit_simple(Level::Warn, None, None, Some(fields));
    }

    pub fn error_event(fields: Value) {
        emit_simple(Level::Error, None, None, Some(fields));
    }

    pub fn debug_event(fields: Value) {
        emit_simple(Level::Debug, None, None, Some(fields));
    }
}

pub(crate) fn log_from_tracing(
    level: Level,
    tag: &str,
    message: &str,
    fields: serde_json::Map<String, Value>,
) {
    emit_simple(level, Some(tag), Some(message), Some(Value::Object(fields)));
}

fn emit_simple(level: Level, tag: Option<&str>, message: Option<&str>, fields: Option<Value>) {
    let config = global_config();

    if !config.enabled || level < config.min_level {
        return;
    }

    let mut event = match (tag, message) {
        (Some(t), Some(m)) => Event::simple(level, t, m),
        _ => Event::new(level),
    };

    if let Some(Value::Object(map)) = fields {
        event.fields = map;
    }

    event.service = Some(config.env.service.clone());
    event.environment = Some(config.env.environment.clone());

    if let Some(ref version) = config.env.version {
        event.fields.entry("version".to_string())
            .or_insert_with(|| serde_json::json!(version));
    }
    if let Some(ref commit) = config.env.commit_hash {
        event.fields.entry("commit_hash".to_string())
            .or_insert_with(|| serde_json::json!(commit));
    }
    if let Some(ref region) = config.env.region {
        event.fields.entry("region".to_string())
            .or_insert_with(|| serde_json::json!(region));
    }

    if let Some(ref enrich_fn) = config.enrich {
        enrich_fn(&mut event);
    }

    if let Some(ref redact) = config.redact {
        redact.redact_event(&mut event);
    }

    if let Some(ref sampling) = config.sampling {
        if !sampling.should_sample(&event) {
            return;
        }
    }

    config.drain.send(&event);
}
