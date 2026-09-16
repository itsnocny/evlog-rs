# evlog 🦀

> Structured logging with wide events for Rust. Accumulate context, emit once.

A Rust port of [evlog](https://www.evlog.dev) — the TypeScript structured logging library based on **wide events**.

## What are Wide Events?

Instead of scattering logs throughout your code, you accumulate context over a unit of work (HTTP request, background job, script) and emit a **single, comprehensive event**.

```
# Traditional logging — noise
INFO  Job started
INFO  User authenticated { userId: 1 }
INFO  Fetching data { source: "postgres" }
INFO  Processing complete
INFO  Job finished { duration: 234 }

# Wide events — one event, all context
[INFO] POST /api/checkout (234ms)
  ├─ user: { id=1 plan=pro }
  ├─ cart: { items=3 total=99.99 }
  └─ payment: { method=card status=success }
```

## Quick Start

```rust
use evlog::{init_logger, Config, EventLogger, json};
use evlog::simple::log;

fn main() {
    init_logger(
        Config::builder()
            .service("my-app")
            .pretty(true)
            .build()
    );

    // Simple fire-and-forget logs
    log::info("app", "Server started on port 3000");
    log::warn("cache", "Cache miss for key user:42");

    // Wide event — accumulate context, emit once
    let mut ev = EventLogger::with_request("POST", "/api/checkout");
    ev.set("user", json!({"id": 1, "plan": "pro"}));
    ev.set("cart", json!({"items": 3, "total": 99.99}));
    ev.set("payment", json!({"method": "card", "status": "success"}));
    ev.emit();
}
```

## Features

### Simple Logging

Tagged fire-and-forget logs, like `console.log` but structured:

```rust
use evlog::simple::log;

log::info("auth", "User logged in");
log::warn("cache", "Cache miss");
log::error("payment", "Stripe webhook failed");
log::debug("router", "Matched route /api/checkout");

// Structured events
log::info_event(json!({
    "action": "user_login",
    "user_id": 42,
    "provider": "github"
}));
```

### Wide Events

Accumulate context over a unit of work, emit once:

```rust
use evlog::{EventLogger, json};

// HTTP request
let mut log = EventLogger::with_request("POST", "/api/checkout");
log.set("user", json!({"id": 1, "plan": "pro"}));
log.set("cart", json!({"items": 3, "total": 99.99}));
log.emit();

// Background job
let mut fields = serde_json::Map::new();
fields.insert("job_id".into(), json!("sync-001"));

let mut job = EventLogger::with_fields(fields);
job.set("records", json!({"found": 1250, "synced": 1250}));
job.emit();
```

### Structured Errors

Errors with actionable context — `code`, `why`, `fix`, `link`:

```rust
use evlog::error::EvlogError;

let err = EvlogError::new("Payment failed")
    .code("PAYMENT_DECLINED")
    .status(402)
    .why("Card declined by issuer (insufficient funds)")
    .fix("Try a different payment method or contact your bank")
    .link("https://docs.example.com/payments/declined");
```

Or with the `create_error!` macro:

```rust
use evlog::create_error;

let err = create_error!(
    message: "Payment failed",
    code: "PAYMENT_DECLINED",
    status: 402,
    why: "Card declined by issuer",
    fix: "Try a different payment method",
);
```

### Sampling

Head sampling (probabilistic by level) + tail sampling (force-keep by condition):

```rust
use evlog::{Config, SamplingConfig, KeepRule, Level, init_logger};

init_logger(
    Config::builder()
        .service("my-app")
        .sampling(
            SamplingConfig::builder()
                .rate(Level::Info, 10)    // Keep 10% of info logs
                .rate(Level::Error, 100)  // Always keep errors
                .keep_status(400)         // Keep all 4xx/5xx
                .keep_duration(1000)      // Keep slow requests (>1s)
                .keep_path("/api/payments/**")
                .build()
        )
        .build()
);
```

### Auto-Redaction

Automatically redacts sensitive fields in production:

```rust
use evlog::{Config, RedactConfig, init_logger};

init_logger(
    Config::builder()
        .redact(
            RedactConfig::builder()
                .builtins(true)  // password, token, api_key, etc.
                .path("custom_secret")
                .pattern("sk_live_")
                .build()
        )
        .build()
);
```

### Custom Drains

Send events to external services by implementing the `Drain` trait:

```rust
use evlog::{Drain, Event};

struct AxiomDrain { /* ... */ }

impl Drain for AxiomDrain {
    fn send(&self, event: &Event) {
        // Send to Axiom, Datadog, etc.
    }
}
```

## Framework Integrations

`evlog` integrates seamlessly with popular Rust web frameworks, automatically creating a wide event per request and injecting it into the request extensions for your handlers to use.

Enable the features in your `Cargo.toml`:

```toml
[dependencies]
evlog = { version = "0.1", features = ["axum", "actix", "rocket"] }
```

### Axum

```rust
use axum::{Router, routing::get, Extension};
use evlog::{frameworks::axum::EvlogLayer, LoggerHandle, json};

async fn handler(Extension(log): Extension<LoggerHandle>) -> &'static str {
    log.set("user", json!({"id": 1}));
    "Hello"
}

// In your app setup:
let app = Router::new()
    .route("/", get(handler))
    .layer(EvlogLayer);
```

### Actix-Web

```rust
use actix_web::{web, App, HttpServer, HttpMessage, HttpRequest};
use evlog::{frameworks::actix::EvlogMiddleware, LoggerHandle, json};

async fn index(req: HttpRequest) -> &'static str {
    if let Some(log) = req.extensions().get::<LoggerHandle>() {
        log.set("user", json!({"id": 1}));
    }
    "Hello"
}

// In your App setup:
App::new()
    .wrap(EvlogMiddleware)
    .route("/", web::get().to(index))
```

### Rocket

```rust
#[macro_use] extern crate rocket;
use evlog::{frameworks::rocket::EvlogFairing, LoggerHandle, json};

#[get("/")]
fn index(log: LoggerHandle) -> &'static str {
    log.set("user", json!({"id": 1}));
    "Hello"
}

// In your launch block:
rocket::build()
    .attach(EvlogFairing)
    .mount("/", routes![index])
```

## Output Formats

**Development** (pretty, tree format with colors):
```
14:08:56.932 [app] Server starting on port 3000
14:08:56.932 WARN [cache] Cache miss for key user:42

[INFO] POST /api/checkout in 234ms
  ├─ user: { id=1 plan=pro }
  ├─ cart: { items=3 total=99.99 }
  └─ payment: { method=card status=success }

ERROR POST /api/checkout in 123ms
  ├─ error: Card declined
  │    Why: Issuer declined the charge
  │    Fix: Ask the customer to use another card
  ├─ user: { id=1 plan=pro }
  └─ cart: { items=3 total=99.99 }
```

**Production** (JSON):
```json
{"level":"info","timestamp":"2024-01-15T14:08:56Z","service":"my-app","method":"POST","path":"/api/checkout","duration_ms":234,"user":{"id":1,"plan":"pro"},"cart":{"items":3,"total":99.99}}
```

## Configuration

```rust
use evlog::{init_logger, Config, Level};

init_logger(
    Config::builder()
        .service("my-api")
        .environment("production")
        .pretty(false)          // JSON output
        .silent(false)          // Write to stdout
        .min_level(Level::Info) // Filter debug logs
        .build()
);
```

Auto-detected environment variables:
| Field | Env Vars |
|-------|----------|
| `service` | `SERVICE_NAME` |
| `environment` | `RUST_ENV`, `APP_ENV` |
| `version` | `APP_VERSION` |
| `commit_hash` | `COMMIT_SHA`, `GITHUB_SHA` |
| `region` | `AWS_REGION`, `FLY_REGION` |

## License

MIT
