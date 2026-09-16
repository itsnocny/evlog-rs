use std::collections::HashMap;
use crate::level::Level;
use crate::event::Event;
use globset::{Glob, GlobMatcher};

/// Configuration for head and tail sampling.
#[derive(Debug, Clone)]
pub struct SamplingConfig {
    /// Percentage (0-100) of events to keep per level.
    pub rates: HashMap<Level, u8>,
    /// Tail sampling rules. If ANY matches, the event is force-kept.
    pub keep: Vec<KeepRule>,
}

#[derive(Clone)]
pub enum KeepRule {
    Status(u16),
    Duration(u64),
    Path { pattern: String, matcher: GlobMatcher },
}

impl std::fmt::Debug for KeepRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Status(s) => f.debug_tuple("Status").field(s).finish(),
            Self::Duration(d) => f.debug_tuple("Duration").field(d).finish(),
            Self::Path { pattern, .. } => f.debug_tuple("Path").field(pattern).finish(),
        }
    }
}

impl KeepRule {
    pub fn matches(&self, event: &Event) -> bool {
        match self {
            KeepRule::Status(s) => event.status.map_or(false, |st| st >= *s),
            KeepRule::Duration(d) => event.duration_ms.map_or(false, |dur| dur >= *d as f64),
            KeepRule::Path { matcher, .. } => {
                event.path.as_ref().map_or(false, |ep| matcher.is_match(ep))
            }
        }
    }
}

pub struct SamplingConfigBuilder {
    rates: HashMap<Level, u8>,
    keep: Vec<KeepRule>,
}

impl SamplingConfigBuilder {
    pub fn new() -> Self {
        Self {
            rates: HashMap::new(),
            keep: Vec::new(),
        }
    }

    pub fn rate(mut self, level: Level, rate: u8) -> Self {
        self.rates.insert(level, rate);
        self
    }

    pub fn keep(mut self, rule: KeepRule) -> Self {
        self.keep.push(rule);
        self
    }

    pub fn keep_status(self, status: u16) -> Self {
        self.keep(KeepRule::Status(status))
    }

    pub fn keep_duration(self, duration: u64) -> Self {
        self.keep(KeepRule::Duration(duration))
    }

    pub fn keep_path(self, path: impl Into<String>) -> Self {
        let path = path.into();
        match Glob::new(&path) {
            Ok(glob) => self.keep(KeepRule::Path {
                pattern: path,
                matcher: glob.compile_matcher(),
            }),
            Err(_) => self,
        }
    }

    pub fn build(self) -> SamplingConfig {
        SamplingConfig {
            rates: self.rates,
            keep: self.keep,
        }
    }
}

impl SamplingConfig {
    pub fn builder() -> SamplingConfigBuilder {
        SamplingConfigBuilder::new()
    }

    pub fn should_sample(&self, event: &Event) -> bool {
        for rule in &self.keep {
            if rule.matches(event) {
                return true;
            }
        }

        let rate = self.rates.get(&event.level).copied().unwrap_or(100);

        if rate == 100 {
            return true;
        } else if rate == 0 {
            return false;
        }

        // using pseudo-random to avoid forcing a dependency, using timestamp
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().subsec_nanos();
        (now % 100) < (rate as u32)
    }
}
