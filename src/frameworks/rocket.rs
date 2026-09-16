use rocket::{
    fairing::{Fairing, Info, Kind},
    request::{FromRequest, Outcome, Request},
    Response,
};
use crate::logger::{EventLogger, LoggerHandle};

/// Rocket fairing for evlog.
///
/// Emits a wide event upon request completion. The `LoggerHandle` is available
/// as a request guard so handlers can add context.
pub struct EvlogFairing;

#[rocket::async_trait]
impl Fairing for EvlogFairing {
    fn info(&self) -> Info {
        Info {
            name: "Evlog Middleware",
            kind: Kind::Request | Kind::Response,
        }
    }

    async fn on_request(&self, req: &mut rocket::Request<'_>, _data: &mut rocket::Data<'_>) {
        let method = req.method().to_string();
        let path = req.uri().path().to_string();
        let mut logger = EventLogger::with_request(method, path.clone());
        if !crate::config::global_config().middleware.should_log(&path) {
            logger.skip_emit = true;
        }
        req.local_cache(|| LoggerHandle::new(logger));
    }

    async fn on_response<'r>(&self, req: &'r Request<'_>, res: &mut Response<'r>) {
        let handle = req.local_cache(|| {
            // Fallback — should not reach here since on_request initializes it
            LoggerHandle::new(EventLogger::with_request(
                req.method().to_string(),
                req.uri().path().to_string(),
            ))
        });

        let mut log = handle.lock();
        log.set_status(res.status().code);
        log.emit();
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for LoggerHandle {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let handle = req.local_cache(|| {
            let method = req.method().to_string();
            let path = req.uri().path().to_string();
            LoggerHandle::new(EventLogger::with_request(method, path))
        });
        Outcome::Success(handle.clone())
    }
}

impl<'r> rocket::response::Responder<'r, 'static> for crate::error::EvlogError {
    fn respond_to(self, _req: &'r rocket::Request<'_>) -> rocket::response::Result<'static> {
        let status = rocket::http::Status::from_code(self.status)
            .unwrap_or(rocket::http::Status::InternalServerError);
        let body = self.to_response_body();
        rocket::Response::build()
            .status(status)
            .header(rocket::http::ContentType::JSON)
            .sized_body(body.len(), std::io::Cursor::new(body))
            .ok()
    }
}
