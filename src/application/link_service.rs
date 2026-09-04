use crate::domain;
use crate::domain::{
    CodeGenerator, Error, FetchResult, OriginalUrl, Repository, SaveRequest, ShortCode,
};
use async_trait::async_trait;
use chrono::{Duration, Utc};
use tracing::info;
use url::Url;

/// Describes service that working with links. It's useful if
/// you need to create short code from original URL, or fetch
/// original URL by short code
pub struct LinkService {
    pub generator: Box<dyn CodeGenerator>,
    pub repository: Box<dyn Repository>,
}

impl LinkService {
    /// Create a new `LinkService` with generator and repository
    pub fn new(generator: Box<dyn CodeGenerator>, repository: Box<dyn Repository>) -> Self {
        Self {
            generator,
            repository,
        }
    }
}

#[async_trait]
impl domain::LinkService for LinkService {
    async fn create_short_code(
        &self,
        url: OriginalUrl,
        created_ip: String,
        lifetime: Option<Duration>,
        clicks_limit: Option<i64>,
    ) -> Result<ShortCode, Error> {
        if url.0.chars().count() > 2048 {
            return Err(Error::URLTooLong);
        }

        let url = match Url::parse(&url.0) {
            Err(e) => {
                info!(error = ?e, "Failed to parse provided URL");
                return Err(Error::InvalidURL);
            }
            Ok(url) if !matches!(url.scheme(), "http" | "https") => {
                info!("Invalid URL scheme");
                return Err(Error::InvalidURL);
            }
            Ok(url) => url,
        };

        let code = self.generator.generate();

        self.repository
            .save(SaveRequest {
                created_at: Utc::now(),
                expires_at: lifetime.and_then(|duration| Some(Utc::now() + duration)),
                clicks: 0,
                clicks_limit,
                created_ip,
                url: OriginalUrl(url.to_string()),
                code: code.clone(),
            })
            .await?;

        Ok(code)
    }

    async fn fetch_original_url(&self, code: ShortCode) -> Result<FetchResult, Error> {
        if code.0.chars().count() > 8 {
            return Err(Error::ShortCodeTooLong);
        }

        let url = self.repository.fetch(code).await?;
        Ok(url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::CodeGenerator;
    use crate::domain::{FetchResult, LinkService as _};
    use crate::domain::{SaveRequest, ShortCode};
    use async_trait::async_trait;
    use std::collections::HashMap;
    use tokio::sync::Mutex;

    /// Stores links in memory (for testing)
    struct TestRepository {
        links: Mutex<HashMap<ShortCode, OriginalUrl>>,
        /// Always return `Error::DuplicateCode` when saving code
        code_already_exists: bool,
    }

    impl TestRepository {
        /// Create new `TestRepository`
        /// `code_already_exists` means that repository will always
        /// return `Error::DuplicateCode` when saving code
        fn new(code_already_exists: bool) -> Self {
            TestRepository {
                links: Mutex::new(HashMap::default()),
                code_already_exists,
            }
        }
    }

    #[async_trait]
    impl Repository for TestRepository {
        async fn fetch(&self, code: ShortCode) -> Result<FetchResult, Error> {
            match self.links.lock().await.get(&code).cloned() {
                Some(value) => {
                    let mut res = FetchResult::default();
                    res.url = value;
                    Ok(res)
                }
                None => Err(Error::URLNotFound),
            }
        }

        async fn fetch_for_click(&self, code: ShortCode) -> Result<FetchResult, Error> {
            self.fetch(code).await
        }

        async fn save(&self, request: SaveRequest) -> Result<(), Error> {
            if self.code_already_exists {
                return Err(Error::DuplicateCode);
            }

            self.links.lock().await.insert(request.code, request.url);
            Ok(())
        }

        async fn cleanup_expired_links(&self) -> Result<u64, Error> {
            Ok(0)
        }

        async fn cleanup_links_exceeded_click_limit(&self) -> Result<u64, Error> {
            Ok(0)
        }
    }

    fn prepare_service(code_already_exists: bool) -> LinkService {
        let generator = Box::new(CodeGenerator::new());
        let repository = Box::new(TestRepository::new(code_already_exists));

        LinkService::new(generator, repository)
    }

    #[tokio::test]
    async fn generates_valid_code() {
        let url = OriginalUrl("https://example.com".to_string());
        let service = prepare_service(false);

        let code = service
            .create_short_code(url, "192.168.0.1".to_string(), None, None)
            .await
            .unwrap()
            .0;

        // Note: code length is 8, see application::CodeGenerator docs
        assert_eq!(code.chars().count(), 8);
    }

    #[tokio::test]
    async fn cant_save_too_long_url() {
        let url = OriginalUrl("https://example.com".to_string().repeat(2048));
        let service = prepare_service(false);

        let code = service
            .create_short_code(url, "192.168.0.1".to_string(), None, None)
            .await;

        assert!(matches!(code.err().unwrap(), Error::URLTooLong));
    }

    #[tokio::test]
    async fn cant_save_invalid_url_without_scheme() {
        let url = OriginalUrl("example.com".to_string());
        let service = prepare_service(false);

        let code = service
            .create_short_code(url, "192.168.0.1".to_string(), None, None)
            .await;

        assert!(matches!(code.err().unwrap(), Error::InvalidURL));
    }

    #[tokio::test]
    async fn cant_save_invalid_url_with_invalid_scheme() {
        let url = OriginalUrl("htt://example.com".to_string());
        let service = prepare_service(false);

        let code = service
            .create_short_code(url, "192.168.0.1".to_string(), None, None)
            .await;

        assert!(matches!(code.err().unwrap(), Error::InvalidURL));
    }

    #[tokio::test]
    async fn can_save_http_url() {
        let url = OriginalUrl("http://example.com".to_string());
        let service = prepare_service(false);

        assert!(matches!(
            service
                .create_short_code(url, "192.168.0.1".to_string(), None, None)
                .await,
            Ok(_)
        ));
    }

    #[tokio::test]
    async fn cant_save_existing_code() {
        let url = OriginalUrl("https://example.com".to_string());
        let service = prepare_service(true);

        assert!(matches!(
            service
                .create_short_code(url, "192.168.0.1".to_string(), None, None)
                .await,
            Err(Error::DuplicateCode)
        ));
    }

    #[tokio::test]
    async fn fetch_url_by_saved_code() {
        let url = OriginalUrl("https://example.com".to_string());
        let service = prepare_service(false);

        let code = service
            .create_short_code(url.clone(), "192.168.0.1".to_string(), None, None)
            .await
            .unwrap();

        let fetched_url = service.fetch_original_url(code).await.unwrap();

        assert_eq!(format!("{}/", url.0), fetched_url.url.0);
    }

    #[tokio::test]
    async fn url_normalization() {
        let url = OriginalUrl("https:example.com///////some page".to_string());
        let service = prepare_service(false);

        let code = service
            .create_short_code(url.clone(), "192.168.0.1".to_string(), None, None)
            .await
            .unwrap();

        let fetched_url = service.fetch_original_url(code).await.unwrap();

        assert_eq!(
            OriginalUrl("https://example.com///////some%20page".to_string()),
            fetched_url.url
        );
    }

    #[tokio::test]
    async fn fetch_url_by_not_existing_code() {
        let service = prepare_service(false);

        let fetched_url = service.fetch_original_url(ShortCode("a".to_string())).await;
        assert!(matches!(fetched_url, Err(Error::URLNotFound)));
    }

    #[tokio::test]
    async fn cant_redirect_by_too_long_code() {
        let service = prepare_service(false);

        let fetched_url = service
            .fetch_original_url(ShortCode("a".to_string().repeat(9)))
            .await;

        assert!(matches!(fetched_url, Err(Error::ShortCodeTooLong)));
    }
}
