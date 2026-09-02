use axum::{extract::ConnectInfo, extract::Request, middleware::Next, response::Response};
use std::net::SocketAddr;
use tracing::{Instrument, info_span};

/// HTTP middlewares
pub struct Middlewares;

impl Middlewares {
    /// Creates logging span with user IP
    /// Prefers `X-Real-IP` for determine user IP. If `X-Real-IP` header not provided,
    /// uses socket address to determine user IP
    pub async fn ip_logger(
        ConnectInfo(connect): ConnectInfo<SocketAddr>,
        request: Request,
        next: Next,
    ) -> Response {
        let ip = request
            .headers()
            .get("X-Real-IP")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| Some(value.to_string()))
            .unwrap_or_else(|| connect.ip().to_string());

        let span = info_span!("Request", client_ip = ip);
        next.run(request).instrument(span).await
    }
}
