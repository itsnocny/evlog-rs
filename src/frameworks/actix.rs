use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures_util::future::LocalBoxFuture;
use std::rc::Rc;
use crate::logger::{EventLogger, LoggerHandle};

/// Actix-web middleware for evlog.
///
/// Attaches a `LoggerHandle` to every request and emits a wide event upon completion.
pub struct EvlogMiddleware;

impl<S, B> Transform<S, ServiceRequest> for EvlogMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = EvlogMiddlewareService<S>;
    type Future = std::future::Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        std::future::ready(Ok(EvlogMiddlewareService {
            service: Rc::new(service),
        }))
    }
}

pub struct EvlogMiddlewareService<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for EvlogMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let method = req.method().to_string();
        let path = req.path().to_string();

        let mut logger = EventLogger::with_request(method, &path);
        
        if !crate::config::global_config().middleware.should_log(&path) {
            logger.skip_emit = true;
        }

        let handle = LoggerHandle::new(logger);

        req.extensions_mut().insert(handle.clone());

        let srv = self.service.clone();
        Box::pin(async move {
            let res = srv.call(req).await;
            let mut log = handle.lock();
            match &res {
                Ok(response) => {
                    log.set_status(response.status().as_u16());
                }
                Err(e) => {
                    log.set_status(e.as_response_error().status_code().as_u16());
                }
            }
            log.emit();
            res
        })
    }
}

impl actix_web::ResponseError for crate::error::EvlogError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        actix_web::http::StatusCode::from_u16(self.status)
            .unwrap_or(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR)
    }

    fn error_response(&self) -> actix_web::HttpResponse {
        actix_web::HttpResponse::build(self.status_code())
            .content_type("application/json")
            .body(self.to_response_body())
    }
}
