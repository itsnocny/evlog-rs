use axum::{routing::get, Extension, Router};
use evlog::{frameworks::axum::EvlogLayer, json, Config, LoggerHandle, init_logger, define_error_catalog};
use tokio::net::TcpListener;
use std::time::Duration;
use evlog::event::Event;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Registry};

define_error_catalog! {
    pub enum ApiError {
        PaymentFailed(amount: u32) => {
            message: format!("Payment of {} failed", amount),
            code: "PAYMENT_FAILED",
            status: 402,
            why: "Insufficient funds in the provided card",
            fix: "Prompt user for another payment method"
        }
    }
}

evlog::dump_schemas!(".evlog_schema.md", ApiError);

async fn health_handler() -> &'static str {
    "OK"
}

async fn handler(Extension(log): Extension<LoggerHandle>) -> &'static str {
    // 1. Déclencher un log "tracing" classique, qui sera attrapé par evlog !
    tracing::info!(some_param="123", "User arrived at the checkout page");

    log.set("user", json!({"id": 42, "name": "Alice", "role": "admin"}));
    
    // 2. Mesurer une opération asynchrone (exemple: appel BDD) via `track_async`
    let result = log.track_async("db_query_users", async {
        tokio::time::sleep(Duration::from_millis(30)).await;
        "Data"
    }).await;
    log.set("db_result", result);
    
    // 3. Mesurer une opération synchrone (exemple: hashage mdp) via `track`
    log.track("hash_password", || {
        std::thread::sleep(Duration::from_millis(15));
    });

    log.error_evlog(&ApiError::PaymentFailed(99).into_evlog());
    
    let bg_log = log.fork();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(50)).await;
        bg_log.set("job_type", "send_email");
        bg_log.info("Email recu");
        bg_log.emit();
    });

    "Hello from Axum!"
}

#[tokio::main]
async fn main() {
    init_logger(
        Config::builder()
            .service("axum-api")
            .pretty(true)
            .exclude_route("/health")
            .enrich(|event: &mut Event| {
                event.fields.insert("version".to_string(), json!("v1.0.42"));
            })
            .build()
    );

    // Initialisation du pont tracing -> evlog
    Registry::default()
        .with(evlog::frameworks::tracing::TracingBridge)
        .init();

    let app = Router::new()
        .route("/health", get(health_handler))
        .route("/", get(handler))
        .layer(EvlogLayer);
    
    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();
    println!("Axum server running on http://127.0.0.1:3000");
    axum::serve(listener, app).await.unwrap();
}
