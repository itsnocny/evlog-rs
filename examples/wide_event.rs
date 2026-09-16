use evlog_rs::{EventLogger, init_logger, Config, json};

fn main() {
    init_logger(
        Config::builder()
            .service("checkout-api")
            .pretty(true)
            .build()
    );

    // Simulate a request handler — accumulate context, emit once
    let mut log = EventLogger::with_request("POST", "/api/checkout");

    log.set("user", json!({"id": 1, "plan": "pro"}));
    log.set("cart", json!({"id": 42, "items": 3, "total": 99.99}));
    log.set("payment", json!({"method": "card", "status": "success"}));

    log.emit();

    // Background job example
    let mut fields = serde_json::Map::new();
    fields.insert("job_id".into(), json!("sync-001"));
    fields.insert("queue".into(), json!("emails"));

    let mut job = EventLogger::with_fields(fields);
    job.set("source", json!("postgres"));
    job.set("target", json!("s3"));
    job.set("records", json!({"found": 1250, "synced": 1250}));
    job.emit();
}
