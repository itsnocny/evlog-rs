use serde_json::Value;
use crate::event::Event;

#[derive(Debug, Clone)]
pub struct RedactConfig {
    pub paths: Vec<String>,
    pub patterns: Vec<String>,
    pub replacement: String,
    pub builtins: bool,
}

impl Default for RedactConfig {
    fn default() -> Self {
        Self {
            paths: Vec::new(),
            patterns: Vec::new(),
            replacement: "[REDACTED]".to_string(),
            builtins: true,
        }
    }
}

pub struct RedactConfigBuilder {
    config: RedactConfig,
}

impl RedactConfigBuilder {
    pub fn new() -> Self {
        Self {
            config: RedactConfig::default(),
        }
    }

    pub fn path(mut self, path: impl Into<String>) -> Self {
        self.config.paths.push(path.into());
        self
    }

    pub fn pattern(mut self, pattern: impl Into<String>) -> Self {
        self.config.patterns.push(pattern.into());
        self
    }

    pub fn replacement(mut self, replacement: impl Into<String>) -> Self {
        self.config.replacement = replacement.into();
        self
    }

    pub fn builtins(mut self, builtins: bool) -> Self {
        self.config.builtins = builtins;
        self
    }

    pub fn build(self) -> RedactConfig {
        self.config
    }
}

impl RedactConfig {
    pub fn builder() -> RedactConfigBuilder {
        RedactConfigBuilder::new()
    }

    pub fn redact_event(&self, event: &mut Event) {
        let mut fields = std::mem::take(&mut event.fields);
        self.redact_map(&mut fields);
        event.fields = fields;
    }

    fn redact_map(&self, map: &mut serde_json::Map<String, Value>) {
        // We mutate the JSON map in-place rather than allocating a new one.
        // `iter_mut()` gives us a mutable reference (`&mut Value`) to each property in the object.
        for (key, value) in map.iter_mut() {
            if self.should_redact_key(key) {
                // To replace the value, we dereference the mutable pointer `*value` 
                // and overwrite it with a new `Value::String`.
                *value = Value::String(self.replacement.clone());
            } else {
                // If the key is safe, we recursively check its inner content 
                // (in case it's a nested object or array).
                self.redact_value(value);
            }
        }
    }

    fn redact_value(&self, value: &mut Value) {
        match value {
            Value::String(s) => {
                // Check if the string's content matches any of the sensitive patterns
                for pat in &self.patterns {
                    if s.contains(pat.as_str()) {
                        *value = Value::String(self.replacement.clone());
                        return;
                    }
                }
            }
            // If the value is a nested JSON object, recurse into it
            Value::Object(map) => self.redact_map(map),
            
            // If the value is a JSON array, iterate and recursively redact each item
            Value::Array(arr) => {
                for item in arr.iter_mut() {
                    self.redact_value(item);
                }
            }
            // Numbers, booleans, and nulls are safe and ignored
            _ => {}
        }
    }

    fn should_redact_key(&self, key: &str) -> bool {
        let key_lower = key.to_lowercase();

        if self.builtins {
            let builtins = [
                "password", "passwd", "secret", "token", "api_key", "apikey",
                "access_token", "refresh_token", "authorization", "cookie",
                "ssn", "credit_card", "card_number", "cvv", "private_key"
            ];
            if builtins.contains(&key_lower.as_str()) {
                return true;
            }
        }

        for p in &self.paths {
            if *p == key || *p == key_lower {
                return true;
            }
        }

        false
    }
}
