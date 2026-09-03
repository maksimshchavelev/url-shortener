use crate::domain::{Error, OriginalUrl, ShortCode};
use async_trait::async_trait;
use chrono::*;

/// Result of fetch operation in `Repository`
#[derive(Default)]
pub struct FetchResult {
    /// ID of link
    pub id: i64,

    /// Count of clicks
    pub clicks: i64,

    /// Clicks limit
    pub clicks_limit: i64,

    /// Date and time when a link was created
    pub created_at: DateTime<Utc>,

    /// Date and time when a link expires
    pub expires_at: DateTime<Utc>,

    /// Short code
    pub code: ShortCode,

    /// Original URL
    pub url: OriginalUrl,

    /// IP that created short link
    pub created_ip: String,
}

/// Describes repository that manages URL's
/// This trait is useful if you need to fetch original URL by short code
/// or store short code that related with original URL
#[async_trait]
pub trait Repository: Send + Sync {
    /// Fetch <b>original URL</b> by short code
    /// # Returns
    /// Original URL or `domain::Error`
    async fn fetch_url(&self, code: ShortCode) -> Result<FetchResult, Error>;

    /// Save short code related with original URL
    /// # Returns
    /// Nothing or `domain::Error`
    async fn save_code(&self, short: ShortCode, url: OriginalUrl) -> Result<(), Error>;
}
