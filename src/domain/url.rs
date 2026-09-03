/// Represents original long URL
#[derive(Debug, Clone, PartialEq, Hash, Eq, Default)]
pub struct OriginalUrl(pub String);

/// Represents short code (not full URL)
#[derive(Debug, Clone, PartialEq, Hash, Eq, Default)]
pub struct ShortCode(pub String);
