use evlog_rs::{init_logger, Config};
use evlog_rs::simple::log;
use serde_json::json;

fn main() {
    init_logger(
        Config::builder()
            .service("my-app")
            .pretty(true)
            .build()
    );

    // Tagged logs (fire-and-forget)
    log::info("app", "Server starting on port 3000");
    log::debug("router", "Matched route /api/checkout");
    log::warn("cache", "Cache miss for key user:42");
    log::error("payment", "Stripe webhook verification failed");

    // Structured event
    log::info_event(json!({
        "action": "user_login",
        "user_id": 42,
        "method": "oauth",
        "provider": "github"
    }));
}
