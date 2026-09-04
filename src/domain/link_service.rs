use crate::domain::{Error, FetchResult, OriginalUrl, ShortCode};
use async_trait::async_trait;
use chrono::Duration;

/// Describes service that working with links. It's useful if
/// you need to create short code from original URL, or fetch
/// original URL by short code
#[async_trait]
pub trait LinkService: Send + Sync {
    /// Create short code from original URL while also retaining
    /// the IP address of the person who created the link (`created_ip`)
    /// and sets lifetime of a short code(`lifetime`) with limit of
    /// clicks (`clicks_limit`)
    /// # Returns
    /// Short code saved to `Repository` or `domain::Error`
    async fn create_short_code(
        &self,
        url: OriginalUrl,
        created_ip: String,
        lifetime: Option<Duration>,
        clicks_limit: Option<i64>,
    ) -> Result<ShortCode, Error>;

    /// Fetch original URL by short code and increases clicks count
    /// # Returns
    /// Original URL related with `code` or `domain::Error`
    async fn fetch_original_url(&self, code: ShortCode) -> Result<OriginalUrl, Error>;
}
