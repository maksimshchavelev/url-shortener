use crate::domain::Error;
use axum::Json;
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde_json::json;
use tracing::{error, info, warn};

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let (error_type, title, status, detail) = match self {
            Error::DuplicateCode => (
                "conflict".to_string(),
                self.to_string(),
                StatusCode::INTERNAL_SERVER_ERROR,
                "The generated short code is already taken. Please try again.".to_string(),
            ),

            Error::URLNotFound => (
                "url_not_found".to_string(),
                self.to_string(),
                StatusCode::NOT_FOUND,
                "The requested short code does not exist".to_string(),
            ),

            Error::URLTooLong => (
                "invalid_url".to_string(),
                self.to_string(),
                StatusCode::PAYLOAD_TOO_LARGE,
                "Provided URL is too long".to_string(),
            ),

            Error::InvalidURL => (
                "invalid_url".to_string(),
                self.to_string(),
                StatusCode::BAD_REQUEST,
                "Provided URL is incorrect. Only http or https schemas are allowed and schema must be lower-cased".to_string(),
            ),

            Error::ShortCodeTooLong => (
                "invalid_code".to_string(),
                self.to_string(),
                StatusCode::BAD_REQUEST,
                "Provided short code is too long".to_string()
            ),

            Error::Internal(_) => (
                "internal".to_string(),
                "Internal Server Error".to_string(),
                StatusCode::INTERNAL_SERVER_ERROR,
                "An unexpected error occurred on the server. Please try again later.".to_string(),
            ),
        };

        let body = json!({
            "type": error_type,
            "title": title,
            "detail": detail
        });

        (
            status,
            [(header::CONTENT_TYPE, "application/problem+json")],
            Json(body),
        )
            .into_response()
    }
}

impl Error {
    /// Consumes error, logs it and returns error back
    pub fn log(self) -> Self {
        match &self {
            Error::DuplicateCode => warn!("Short code already exist"),
            Error::URLNotFound => info!("URL related with this short code not found"),
            Error::URLTooLong => warn!("URL is too long"),
            Error::InvalidURL => info!("Invalid URL"),
            Error::ShortCodeTooLong => warn!("Short code is too long"),
            Error::Internal(source) => error!(error = ?source, "Internal Server Error"),
        }

        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duplicate_code_returns_correct_status() {
        let response = Error::DuplicateCode.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn url_not_found_returns_correct_status() {
        let response = Error::URLNotFound.into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn invalid_url_returns_correct_status() {
        let response = Error::InvalidURL.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn url_too_long_returns_correct_status() {
        let response = Error::URLTooLong.into_response();
        assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
    }

    #[test]
    fn short_code_too_long_returns_correct_status() {
        let response = Error::ShortCodeTooLong.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn internal_returns_correct_status() {
        let response = Error::Internal(Box::new(std::io::Error::from(
            std::io::ErrorKind::HostUnreachable,
        )))
        .into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
