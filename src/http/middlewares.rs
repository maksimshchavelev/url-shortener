use crate::http::client_ip::ClientIP;
use axum::body::Body;
use axum::http::StatusCode;
use axum::{extract::ConnectInfo, extract::Request, middleware::Next, response::Response};
use log::error;
use std::net::SocketAddr;
use tracing::{Instrument, info_span};

/// HTTP middlewares
pub struct Middlewares;

impl Middlewares {
    /// Creates logging span with user IP
    /// Uses `ip_extractor` middleware
    pub async fn ip_logger(request: Request, next: Next) -> Response {
        let ip = match request.extensions().get::<ClientIP>().cloned() {
            Some(ip) => ip,
            None => {
                error!(
                    "Failed to receive IP in ip_logger middleware, did you forget to apply ip_extractor middleware?"
                );
                return Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(Body::empty())
                    .unwrap();
            }
        };

        let span = info_span!("Request", client_ip = ip.0);
        next.run(request).instrument(span).await
    }

    /// Extracts client IP address
    /// Prefers `X-Real-IP` for determine user IP. If `X-Real-IP` header not provided,
    /// uses socket address to determine user IP
    pub async fn ip_extractor(
        ConnectInfo(connect): ConnectInfo<SocketAddr>,
        mut request: Request,
        next: Next,
    ) -> Response {
        let ip = request
            .headers()
            .get("X-Real-IP")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| Some(value.to_string()))
            .unwrap_or_else(|| connect.ip().to_string());

        request.extensions_mut().insert(ClientIP(ip.clone()));
        next.run(request).await
    }
}
