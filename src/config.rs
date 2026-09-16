use crate::drain::{ConsoleDrain, Drain};
use crate::env::EnvContext;
use crate::level::Level;
use crate::redact::RedactConfig;
use crate::sampling::SamplingConfig;
use crate::event::Event;
use std::sync::{OnceLock, LazyLock};
use std::sync::Arc;
use std::env;
use globset::{Glob, GlobSet, GlobSetBuilder};

static GLOBAL_CONFIG: OnceLock<Arc<Config>> = OnceLock::new();
static DEFAULT_CONFIG: LazyLock<Arc<Config>> = LazyLock::new(|| Arc::new(Config::default()));

#[derive(Clone)]
pub struct MiddlewareConfig {
    pub exclude: GlobSet,
    pub include: GlobSet,
    pub has_includes: bool,
}

impl MiddlewareConfig {
    pub fn should_log(&self, path: &str) -> bool {
        if self.exclude.is_match(path) {
            return false;
        }
        if self.has_includes && !self.include.is_match(path) {
            return false;
        }
        true
    }
}

impl Default for MiddlewareConfig {
    fn default() -> Self {
        Self {
            exclude: GlobSetBuilder::new().build().unwrap(),
            include: GlobSetBuilder::new().build().unwrap(),
            has_includes: false,
        }
    }
}

/// Global logger configuration.
pub struct Config {
    pub enabled: bool,
    pub env: EnvContext,
    pub pretty: bool,
    pub silent: bool,
    pub min_level: Level,
    pub sampling: Option<SamplingConfig>,
    pub redact: Option<RedactConfig>,
    pub drain: Arc<dyn Drain>,
    pub enrich: Option<Arc<dyn Fn(&mut Event) + Send + Sync>>,
    pub middleware: MiddlewareConfig,
}

impl Default for Config {
    fn default() -> Self {
        let is_prod = env::var("RUST_ENV").map(|v| v == "production").unwrap_or(false);
        Self {
            enabled: true,
            env: EnvContext::auto_detect(),
            pretty: !is_prod,
            silent: false,
            min_level: Level::Debug,
            sampling: None,
            redact: None,
            drain: Arc::new(ConsoleDrain::new(!is_prod, false)),
            enrich: None,
            middleware: MiddlewareConfig::default(),
        }
    }
}

impl Config {
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::new()
    }
}

pub struct ConfigBuilder {
    config: Config,
    exclude_routes: Vec<String>,
    include_routes: Vec<String>,
}

impl ConfigBuilder {
    pub fn new() -> Self {
        Self {
            config: Config::default(),
            exclude_routes: Vec::new(),
            include_routes: Vec::new(),
        }
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.config.enabled = enabled;
        self
    }

    pub fn env(mut self, env: EnvContext) -> Self {
        self.config.env = env;
        self
    }

    pub fn service(mut self, service: impl Into<String>) -> Self {
        self.config.env.service = service.into();
        self
    }

    pub fn environment(mut self, environment: impl Into<String>) -> Self {
        self.config.env.environment = environment.into();
        self
    }

    pub fn pretty(mut self, pretty: bool) -> Self {
        self.config.pretty = pretty;
        self
    }

    pub fn silent(mut self, silent: bool) -> Self {
        self.config.silent = silent;
        self
    }

    pub fn min_level(mut self, min_level: Level) -> Self {
        self.config.min_level = min_level;
        self
    }

    pub fn sampling(mut self, sampling: SamplingConfig) -> Self {
        self.config.sampling = Some(sampling);
        self
    }

    pub fn redact(mut self, redact: RedactConfig) -> Self {
        self.config.redact = Some(redact);
        self
    }

    pub fn drain(mut self, drain: impl Drain + 'static) -> Self {
        self.config.drain = Arc::new(drain);
        self
    }

    pub fn enrich(mut self, f: impl Fn(&mut Event) + Send + Sync + 'static) -> Self {
        self.config.enrich = Some(Arc::new(f));
        self
    }

    pub fn exclude_route(mut self, glob: impl Into<String>) -> Self {
        self.exclude_routes.push(glob.into());
        self
    }

    pub fn include_route(mut self, glob: impl Into<String>) -> Self {
        self.include_routes.push(glob.into());
        self
    }

    pub fn build(mut self) -> Config {
        self.config.drain = Arc::new(ConsoleDrain::new(self.config.pretty, self.config.silent));

        let mut exc = GlobSetBuilder::new();
        for g in &self.exclude_routes {
            if let Ok(glob) = Glob::new(g) {
                exc.add(glob);
            }
        }
        self.config.middleware.exclude = exc.build().unwrap();

        let mut inc = GlobSetBuilder::new();
        for g in &self.include_routes {
            if let Ok(glob) = Glob::new(g) {
                inc.add(glob);
            }
        }
        self.config.middleware.include = inc.build().unwrap();
        self.config.middleware.has_includes = !self.include_routes.is_empty();

        self.config
    }
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub fn init_logger(config: Config) {
    let _ = GLOBAL_CONFIG.set(Arc::new(config));
}

pub fn global_config() -> Arc<Config> {
    GLOBAL_CONFIG.get().cloned().unwrap_or_else(|| DEFAULT_CONFIG.clone())
}

/// Flush any buffered events in the global drain.
///
/// Call this before your application exits to ensure all events are sent.
/// Typically called from a signal handler or at the end of `main()`.
///
/// ```rust,no_run
/// evlog::shutdown();
/// ```
pub fn shutdown() {
    if let Some(config) = GLOBAL_CONFIG.get() {
        config.drain.flush();
    }
}
