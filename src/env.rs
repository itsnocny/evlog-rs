use std::env;

/// Environment context.
#[derive(Debug, Clone)]
pub struct EnvContext {
    pub service: String,
    pub environment: String,
    pub version: Option<String>,
    pub commit_hash: Option<String>,
    pub region: Option<String>,
}

impl Default for EnvContext {
    fn default() -> Self {
        Self {
            service: "app".to_string(),
            environment: "development".to_string(),
            version: None,
            commit_hash: None,
            region: None,
        }
    }
}

impl EnvContext {
    /// Auto-detect environment context from standard env vars.
    pub fn auto_detect() -> Self {
        let mut ctx = Self::default();

        if let Ok(val) = env::var("SERVICE_NAME") {
            ctx.service = val;
        }

        if let Ok(val) = env::var("RUST_ENV").or_else(|_| env::var("APP_ENV")) {
            ctx.environment = val;
        }

        ctx.version = env::var("APP_VERSION").ok();

        ctx.commit_hash = env::var("COMMIT_SHA")
            .or_else(|_| env::var("GITHUB_SHA"))
            .or_else(|_| env::var("VERCEL_GIT_COMMIT_SHA"))
            .or_else(|_| env::var("CF_PAGES_COMMIT_SHA"))
            .ok();

        ctx.region = env::var("VERCEL_REGION")
            .or_else(|_| env::var("AWS_REGION"))
            .or_else(|_| env::var("FLY_REGION"))
            .or_else(|_| env::var("CF_REGION"))
            .ok();

        ctx
    }

    /// Return a new builder for EnvContext.
    pub fn builder() -> EnvContextBuilder {
        EnvContextBuilder::new()
    }
}

/// Builder for EnvContext.
pub struct EnvContextBuilder {
    ctx: EnvContext,
}

impl EnvContextBuilder {
    pub fn new() -> Self {
        Self {
            ctx: EnvContext::auto_detect(),
        }
    }

    pub fn service(mut self, service: impl Into<String>) -> Self {
        self.ctx.service = service.into();
        self
    }

    pub fn environment(mut self, environment: impl Into<String>) -> Self {
        self.ctx.environment = environment.into();
        self
    }

    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.ctx.version = Some(version.into());
        self
    }

    pub fn commit_hash(mut self, hash: impl Into<String>) -> Self {
        self.ctx.commit_hash = Some(hash.into());
        self
    }

    pub fn region(mut self, region: impl Into<String>) -> Self {
        self.ctx.region = Some(region.into());
        self
    }

    pub fn build(self) -> EnvContext {
        self.ctx
    }
}

impl Default for EnvContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}
