use std::time::Instant;
use std::sync::{Arc, Mutex, MutexGuard};
use serde_json::Value;
use uuid::Uuid;
use std::error::Error;

use crate::event::{Event, ErrorInfo, Fields};
use crate::level::Level;
use crate::config::global_config;
use crate::error::EvlogError;

/// A wide event logger that accumulates context and emits a single event.
pub struct EventLogger {
    event: Event,
    start: Instant,
    sealed: bool,
    pub skip_emit: bool,
    level_override: Option<Level>,
}

impl EventLogger {
    /// Create a new wide event logger with an auto-generated request ID.
    pub fn new() -> Self {
        let mut event = Event::new(Level::Info);
        event.request_id = Some(Uuid::new_v4().to_string());
        Self {
            event,
            start: Instant::now(),
            sealed: false,
            skip_emit: false,
            level_override: None,
        }
    }

    /// Create a logger pre-populated with initial fields.
    pub fn with_fields(initial: Fields) -> Self {
        let mut logger = Self::new();
        logger.event.fields = initial;
        logger
    }

    /// Create a logger for an HTTP request.
    pub fn with_request(method: impl Into<String>, path: impl Into<String>) -> Self {
        let mut logger = Self::new();
        logger.event.method = Some(method.into());
        logger.event.path = Some(path.into());
        logger
    }

    /// Fork the logger for a background job.
    /// Retains context (request_id, fields) but resets timers and HTTP state.
    pub fn fork(&self) -> Self {
        let mut event = Event::new(self.event.level);
        event.request_id = self.event.request_id.clone();
        event.service = self.event.service.clone();
        event.environment = self.event.environment.clone();
        event.fields = self.event.fields.clone();
        // Method, path, tag, error, status, etc., are intentionally NOT cloned.
        
        Self {
            event,
            start: Instant::now(),
            sealed: false,
            skip_emit: false,
            level_override: None,
        }
    }

    pub fn set(&mut self, key: impl Into<String>, value: impl Into<Value>) {
        if self.sealed {
            eprintln!("[evlog] warning: dropped keys after emit");
            return;
        }
        self.event.fields.insert(key.into(), value.into());
    }

    pub fn set_many(&mut self, fields: impl IntoIterator<Item = (String, Value)>) {
        if self.sealed {
            eprintln!("[evlog] warning: dropped keys after emit");
            return;
        }
        for (k, v) in fields {
            self.event.fields.insert(k, v);
        }
    }

    pub fn error(&mut self, err: &(dyn Error + 'static)) {
        if self.sealed { return; }
        self.level_override = Some(Level::Error);
        let error_info = if let Some(evlog_err) = err.downcast_ref::<EvlogError>() {
            evlog_err.to_error_info()
        } else {
            ErrorInfo {
                name: "Error".to_string(), message: err.to_string(),
                code: None, why: None, fix: None, link: None, stack: None, internal: None,
            }
        };
        self.event.error = Some(error_info);
    }

    pub fn error_evlog(&mut self, err: &EvlogError) {
        if self.sealed { return; }
        self.level_override = Some(Level::Error);
        self.event.error = Some(err.to_error_info());
    }

    pub fn warn(&mut self, msg: impl Into<String>) {
        if self.sealed { return; }
        let current = self.level_override.unwrap_or(self.event.level);
        if current < Level::Warn { self.level_override = Some(Level::Warn); }
        self.event.message = Some(msg.into());
    }

    pub fn info(&mut self, msg: impl Into<String>) {
        if self.sealed { return; }
        self.event.message = Some(msg.into());
    }

    pub fn set_level(&mut self, level: Level) {
        if self.sealed { return; }
        self.level_override = Some(level);
    }
    
    pub fn set_status(&mut self, status: u16) {
        if !self.sealed { self.event.status = Some(status); }
    }

    pub fn emit(&mut self) -> Option<Event> {
        if self.sealed {
            eprintln!("[evlog] warning: emit called on sealed logger");
            return None;
        }
        if self.skip_emit {
            self.sealed = true;
            return None; // Middleware route was excluded
        }
        self.sealed = true;
        self.event.duration_ms = Some(self.start.elapsed().as_millis() as f64);

        if let Some(lvl) = self.level_override {
            self.event.level = lvl;
        }

        let config = global_config();
        if !config.enabled || self.event.level < config.min_level {
            return None;
        }

        self.event.service = Some(config.env.service.clone());
        self.event.environment = Some(config.env.environment.clone());

        if let Some(ref version) = config.env.version {
            self.event.fields.entry("version".to_string())
                .or_insert_with(|| serde_json::json!(version));
        }
        if let Some(ref commit) = config.env.commit_hash {
            self.event.fields.entry("commit_hash".to_string())
                .or_insert_with(|| serde_json::json!(commit));
        }
        if let Some(ref region) = config.env.region {
            self.event.fields.entry("region".to_string())
                .or_insert_with(|| serde_json::json!(region));
        }

        let mut final_event = self.event.clone();

        // Apply global enrich callback
        if let Some(ref enrich_fn) = config.enrich {
            enrich_fn(&mut final_event);
        }

        if let Some(ref redact) = config.redact {
            redact.redact_event(&mut final_event);
        }

        if let Some(ref sampling) = config.sampling {
            if !sampling.should_sample(&final_event) { return None; }
        }

        config.drain.send(&final_event);
        Some(final_event)
    }
}

impl Default for EventLogger {
    fn default() -> Self { Self::new() }
}

#[derive(Clone)]
pub struct LoggerHandle {
    inner: Arc<Mutex<EventLogger>>,
}

impl LoggerHandle {
    pub fn new(logger: EventLogger) -> Self {
        Self { inner: Arc::new(Mutex::new(logger)) }
    }

    pub fn lock(&self) -> MutexGuard<'_, EventLogger> {
        self.inner.lock().unwrap()
    }

    pub fn fork(&self) -> LoggerHandle {
        LoggerHandle::new(self.lock().fork())
    }

    pub fn set(&self, key: impl Into<String>, value: impl Into<Value>) {
        self.lock().set(key, value);
    }

    pub fn set_many(&self, fields: impl IntoIterator<Item = (String, Value)>) {
        self.lock().set_many(fields);
    }

    pub fn error(&self, err: &(dyn Error + 'static)) {
        self.lock().error(err);
    }

    pub fn error_evlog(&self, err: &EvlogError) {
        self.lock().error_evlog(err);
    }

    pub fn warn(&self, msg: impl Into<String>) {
        self.lock().warn(msg);
    }

    pub fn info(&self, msg: impl Into<String>) {
        self.lock().info(msg);
    }

    /// Measure the duration of a synchronous closure and store it in metrics.
    pub fn track<F, R>(&self, name: &str, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let start = Instant::now();
        let result = f();
        let elapsed = start.elapsed().as_millis() as f64;
        self.set_metric(name, elapsed);
        result
    }

    /// Measure the duration of an asynchronous future and store it in metrics.
    pub async fn track_async<F, R>(&self, name: &str, fut: F) -> R
    where
        F: std::future::Future<Output = R>,
    {
        let start = Instant::now();
        let result = fut.await;
        let elapsed = start.elapsed().as_millis() as f64;
        self.set_metric(name, elapsed);
        result
    }

    fn set_metric(&self, name: &str, elapsed: f64) {
        let mut lock = self.lock();
        if lock.sealed { return; }
        
        let metrics = lock.event.fields
            .entry("metrics".to_string())
            .or_insert_with(|| serde_json::json!({}));
            
        if let Some(obj) = metrics.as_object_mut() {
            obj.insert(name.to_string(), serde_json::json!(elapsed));
        }
    }

    pub fn emit(&self) -> Option<Event> {
        self.lock().emit()
    }
}
