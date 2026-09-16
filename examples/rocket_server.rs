#[macro_use]
extern crate rocket;

use evlog::{frameworks::rocket::EvlogFairing, json, Config, LoggerHandle, init_logger};

#[get("/")]
fn handler(log: LoggerHandle) -> &'static str {
    log.set("user", json!({"id": 77, "name": "Charlie", "role": "viewer"}));
    log.info("Processing user request from Rocket!");
    "Hello from Rocket!"
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    init_logger(Config::builder().service("rocket-api").pretty(true).build());

    let config = rocket::Config::figment()
        .merge(("port", 3002))
        .merge(("address", "127.0.0.1"));

    println!("Rocket server running on http://127.0.0.1:3002");
    let _ = rocket::custom(config)
        .attach(EvlogFairing)
        .mount("/", routes![handler])
        .launch()
        .await?;

    Ok(())
}
