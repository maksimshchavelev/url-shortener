use crate::domain::{AppState, Error, OriginalUrl, ShortCode};
use crate::http::client_ip::ClientIP;
use crate::http::requests::{CreateLinkRequest, CreateLinkResponse};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect, Response};
use axum::{Extension, Json, extract::Path, extract::State};
use chrono::Duration;
use std::sync::Arc;
use tracing::{debug, info, instrument};

/// HTTP handlers
pub struct Handlers;

impl Handlers {
    /// Handle create short code request
    #[instrument(skip(state, request, ip), fields(
        url_len = request.url.len(),
        lifetime_seconds = request.lifetime_seconds,
        clicks_limit = request.clicks_limit))]
    pub async fn handle_create(
        State(state): State<Arc<AppState>>,
        Extension(ip): Extension<ClientIP>,
        Json(request): Json<CreateLinkRequest>,
    ) -> Result<Response, Error> {
        debug!(
            original_url = truncate_with_ellipsis(&request.url, 80),
            "Got URL"
        );

        let code = state
            .link_service
            .create_short_code(
                OriginalUrl(request.url.clone()),
                ip.0,
                request
                    .lifetime_seconds
                    .and_then(|lifetime| Some(Duration::seconds(lifetime))),
                request.clicks_limit,
            )
            .await
            .map_err(|e| e.log())?;

        info!(short_code = code.0, "Short code created");

        Ok((
            StatusCode::CREATED,
            Json(CreateLinkResponse {
                url: request.url,
                code: code.0,
            }),
        )
            .into_response())
    }

    /// Handle redirect request
    #[instrument(skip(state, code), fields(code_len = code.len()))]
    pub async fn handle_redirect(
        Path(code): Path<String>,
        State(state): State<Arc<AppState>>,
    ) -> Result<Redirect, Error> {
        debug!(
            short_code = truncate_with_ellipsis(&code, 16),
            "Got short code"
        );

        let url = state
            .link_service
            .fetch_original_url(ShortCode(code.clone()))
            .await
            .map_err(|e| e.log())?;

        info!(
            original_url = truncate_with_ellipsis(&url.0, 80),
            "Redirecting"
        );

        Ok(Redirect::temporary(&url.0))
    }
}

/// Truncate provided `&str` and append ellipsis if `&str` is too long
/// # Returns
/// New truncated `String`
fn truncate_with_ellipsis(s: &str, max_len: usize) -> String {
    if s.chars().count() <= max_len {
        s.to_string()
    } else {
        let mut truncated: String = s.chars().take(max_len).collect();
        truncated.push_str("...");
        truncated
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_appends_ellipsis() {
        let s = truncate_with_ellipsis("abc", 2);
        assert_eq!(s, "ab...");
    }

    #[test]
    fn truncate_not_truncates_string_with_len_equal_to_max_len() {
        let s = truncate_with_ellipsis("abc", 3);
        assert_eq!(s, "abc");
    }
}
