use crate::domain::{Error, FetchResult, OriginalUrl, ShortCode};
use async_trait::async_trait;
use chrono::Duration;

/// Result of `cleanup` operation of `LinkService`
#[derive(Default, Copy, Clone)]
pub struct CleanupResult {
    /// Count of removed links that expired
    pub expired_links_removed: u64,

    /// Count of removed links that exceeded count of clicks
    pub exceeded_links_removed: u64,
}

/// Describes service that working with links. It's useful if
/// you need to create short code from original URL, or fetch
/// original URL by short code
#[async_trait]
pub trait LinkService: Send + Sync {
    /// Create short code from original URL while also retaining
    /// the IP address of the person who created the link (`creator_ip`)
    /// and sets lifetime of a short code(`lifetime`) with limit of
    /// clicks (`clicks_limit`)
    /// # Returns
    /// Short code saved to `Repository` or `domain::Error`
    async fn create_short_code(
        &self,
        url: OriginalUrl,
        creator_ip: String,
        lifetime: Option<Duration>,
        clicks_limit: Option<i64>,
    ) -> Result<ShortCode, Error>;

    /// Fetch original URL by short code and increases clicks count
    /// # Returns
    /// Original URL related with `code` or `domain::Error`
    async fn fetch_original_url(&self, code: ShortCode) -> Result<OriginalUrl, Error>;

    /// Discover link by short code
    /// # Returns
    /// `FetchResult` or `domain::Error`
    async fn discover(&self, code: ShortCode) -> Result<FetchResult, Error>;

    /// Removes expired links and links that exceeded clicks limit
    /// # Returns
    /// `CleanupResult` or `domain::Error`
    async fn cleanup(&self) -> Result<CleanupResult, Error>;
}
