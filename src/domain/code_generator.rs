use crate::domain::ShortCode;

/// Trait to generate short codes
pub trait CodeGenerator: Send + Sync {
    /// Generate short code
    /// # Returns
    /// `ShortCode` with generated short code
    fn generate(&self) -> ShortCode;
}
