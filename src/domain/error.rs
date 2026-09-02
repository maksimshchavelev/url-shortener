use thiserror::Error;

/// Represents an error
#[derive(Debug, Error)]
pub enum Error {
    /// Short code already exists
    #[error("Duplicate code")]
    DuplicateCode,

    /// URL not found by short code
    #[error("URL not found")]
    URLNotFound,

    /// URL is too long
    #[error("URL is too long")]
    URLTooLong,

    /// URL has invalid format
    #[error("Invalid URL")]
    InvalidURL,

    /// Short code is too long
    #[error("Short code is too long")]
    ShortCodeTooLong,

    /// Internal error occurred
    #[error("Internal error")]
    Internal(#[from] Box<dyn std::error::Error>),
}

impl Error {
    /// Create `Error::Internal` from error that implements
    /// `std::error::Error`
    pub fn from_internal<E>(err: E) -> Self
    where
        E: std::error::Error + 'static,
    {
        Self::Internal(Box::new(err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_internal_creates_error() {
        use std::error::Error as _;
        use std::io;

        let error = Error::from_internal(io::Error::from(io::ErrorKind::HostUnreachable));
        assert_eq!(
            error
                .source()
                .unwrap()
                .downcast_ref::<io::Error>()
                .unwrap()
                .kind(),
            io::ErrorKind::HostUnreachable
        );
    }
}
