use actix_web::{web, App, HttpMessage, HttpRequest, HttpServer};
use evlog_rs::{frameworks::actix::EvlogMiddleware, json, Config, LoggerHandle, init_logger};

async fn handler(req: HttpRequest) -> &'static str {
    // Actix request extensions
    if let Some(log) = req.extensions().get::<LoggerHandle>() {
        log.set("user", json!({"id": 99, "name": "Bob", "role": "editor"}));
        log.info("Processing user request from Actix!");
    }
    "Hello from Actix-Web!"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    init_logger(Config::builder().service("actix-api").pretty(true).build());

    println!("Actix server running on http://127.0.0.1:3001");
    HttpServer::new(|| {
        App::new()
            .wrap(EvlogMiddleware)
            .route("/", web::get().to(handler))
    })
    .bind(("127.0.0.1", 3001))?
    .run()
    .await
}
