use evlog::{EventLogger, init_logger, Config, json};
use evlog::error::EvlogError;

fn process_payment() -> Result<(), EvlogError> {
    Err(EvlogError::new("Payment failed")
        .code("PAYMENT_DECLINED")
        .status(402)
        .why("Card declined by issuer (insufficient funds)")
        .fix("Try a different payment method or contact your bank")
        .link("https://docs.example.com/payments/declined"))
}

fn main() {
    init_logger(
        Config::builder()
            .service("checkout-api")
            .pretty(true)
            .build()
    );

    let mut log = EventLogger::with_request("POST", "/api/checkout");
    log.set("user", json!({"id": 1, "plan": "pro"}));
    log.set("cart", json!({"items": 3, "total": 99.99}));

    match process_payment() {
        Ok(()) => {
            log.set("payment", json!({"status": "success"}));
        }
        Err(err) => {
            log.error_evlog(&err);
        }
    }

    log.emit();
}
