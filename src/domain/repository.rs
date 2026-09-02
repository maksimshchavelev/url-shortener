use crate::domain::{Error, OriginalUrl, ShortCode};
use async_trait::async_trait;

/// Describes repository that manages URL's
/// This trait is useful if you need to fetch original URL by short code
/// or store short code that related with original URL
#[async_trait]
pub trait Repository: Send + Sync {
    /// Fetch <b>original URL</b> by short code
    /// # Returns
    /// Original URL or `domain::Error`
    async fn fetch_url(&self, code: ShortCode) -> Result<OriginalUrl, Error>;

    /// Save short code related with original URL
    /// # Returns
    /// Nothing or `domain::Error`
    async fn save_code(&self, short: ShortCode, url: OriginalUrl) -> Result<(), Error>;
}
