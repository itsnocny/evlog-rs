use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Log severity level.
///
/// Levels are ordered: `Debug < Info < Warn < Error`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    Debug = 0,
    Info = 1,
    Warn = 2,
    Error = 3,
}

impl Level {
    /// Returns the numeric value of this level.
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    /// Returns `true` if this level is at least `min`.
    pub fn is_at_least(self, min: Level) -> bool {
        self >= min
    }
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Level::Debug => write!(f, "debug"),
            Level::Info => write!(f, "info"),
            Level::Warn => write!(f, "warn"),
            Level::Error => write!(f, "error"),
        }
    }
}

impl FromStr for Level {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "debug" => Ok(Level::Debug),
            "info" => Ok(Level::Info),
            "warn" | "warning" => Ok(Level::Warn),
            "error" => Ok(Level::Error),
            _ => Err(format!("unknown log level: {}", s)),
        }
    }
}

impl Default for Level {
    fn default() -> Self {
        Level::Info
    }
}
