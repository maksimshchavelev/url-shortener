use crate::domain::{Error, OriginalUrl, ShortCode};
use async_trait::async_trait;
use chrono::*;

/// Result of fetch operation in `Repository`
#[derive(Default, Clone)]
pub struct FetchResult {
    /// ID of link
    pub id: i64,

    /// Count of clicks
    pub clicks: i64,

    /// Clicks limit
    pub clicks_limit: Option<i64>,

    /// Date and time when a link was created
    pub created_at: DateTime<Utc>,

    /// Date and time when a link expires
    pub expires_at: Option<DateTime<Utc>>,

    /// Short code
    pub code: ShortCode,

    /// Original URL
    pub url: OriginalUrl,

    /// IP that created short link
    pub creator_ip: String,
}

/// Request to save short code with additional info
#[derive(Default, Clone)]
pub struct SaveRequest {
    /// Short code to save
    pub code: ShortCode,

    /// Original URL
    pub url: OriginalUrl,

    /// Count of clicks
    pub clicks: i64,

    /// Limit of clicks
    pub clicks_limit: Option<i64>,

    /// When link was created
    pub created_at: DateTime<Utc>,

    /// When link expires
    pub expires_at: Option<DateTime<Utc>>,

    /// IP that created short link
    pub creator_ip: String,
}

/// Describes repository that manages URL's
/// This trait is useful if you need to fetch original URL by short code
/// or store short code that related with original URL
#[async_trait]
pub trait Repository: Send + Sync {
    /// Fetch record by short code
    /// # Returns
    /// `FetchResult` or `domain::Error`
    async fn fetch(&self, code: ShortCode) -> Result<FetchResult, Error>;

    /// Fetch a record by short code, <b>increasing the number of clicks</b>
    /// # Returns
    /// `FetchResult` or `domain::Error`
    async fn fetch_for_click(&self, code: ShortCode) -> Result<FetchResult, Error>;

    /// Save short code related with original URL
    /// # Returns
    /// Nothing or `domain::Error`
    async fn save(&self, request: SaveRequest) -> Result<(), Error>;

    /// Removes links that have expired
    /// # Returns
    /// Count of removed links or `domain::Error`
    async fn cleanup_expired_links(&self) -> Result<u64, Error>;

    /// Removes links that have exceeded their click limit
    /// # Returns
    /// Count of removed links or `domain::Error`
    async fn cleanup_links_exceeded_clicks_limit(&self) -> Result<u64, Error>;
}
