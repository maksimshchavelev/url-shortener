use serde::{Deserialize, Serialize};

/// Describes JSON request to create short code
#[derive(Deserialize, Debug)]
pub struct CreateLinkRequest {
    /// Original URL that will be shorted
    pub url: String,
}

/// Describes JSON response on create short code request
#[derive(Serialize, Debug)]
pub struct CreateLinkResponse {
    /// Original URL
    pub url: String,
    /// Short code
    pub code: String,
}
