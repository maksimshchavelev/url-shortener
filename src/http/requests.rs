use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Describes JSON request to create short code
#[derive(Deserialize, Debug)]
pub struct CreateLinkRequest {
    /// Original URL that will be shorted
    pub url: String,

    /// Limit of clicks for a link
    #[serde(default)]
    pub clicks_limit: Option<i64>,

    /// How long will the link remain active?
    #[serde(default)]
    pub lifetime_seconds: Option<i64>,
}

/// Describes JSON response on create short code request
#[derive(Serialize, Debug)]
pub struct CreateLinkResponse {
    /// Original URL
    pub url: String,

    /// Short code
    pub code: String,
}

// Describes JSON response to discover link request
#[derive(Serialize, Debug)]
pub struct DiscoverLinkResponse {
    /// Original URL
    pub url: String,

    /// Short code
    pub code: String,

    /// Count of clicks
    pub clicks: i64,

    /// Limit of clicks
    pub clicks_limit: Option<i64>,

    /// When short link created
    pub created_at: DateTime<Utc>,

    /// When short link expires
    pub expires_at: Option<DateTime<Utc>>,
}
