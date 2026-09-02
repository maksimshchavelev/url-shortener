use crate::domain::{Error, OriginalUrl, ShortCode};
use async_trait::async_trait;

/// Describes service that working with links. It's useful if
/// you need to create short code from original URL, or fetch
/// original URL by short code
#[async_trait]
pub trait LinkService: Send + Sync {
    /// Create short code from original URL
    /// # Returns
    /// Short code saved to `Repository` or `domain::Error`
    async fn create_short_code(&self, url: OriginalUrl) -> Result<ShortCode, Error>;

    /// Fetch original URL by short code
    /// # Returns
    /// Original URL related with `code` or `domain::Error`
    async fn fetch_original_url(&self, code: ShortCode) -> Result<OriginalUrl, Error>;
}
