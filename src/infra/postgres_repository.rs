use crate::domain::{Error, FetchResult, OriginalUrl, Repository, SaveRequest, ShortCode};
use async_trait::async_trait;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;

/// Repository that working with Postgres
pub struct PostgresRepository {
    pool: PgPool,
}

impl PostgresRepository {
    /// Creates new `PostgresRepository` and connects to database
    /// The database connection address is specified in `url`. The
    /// `max_connections` parameter specifies the maximum number
    /// of connections
    pub async fn new(url: &str, max_connections: u32) -> Result<Self, Error> {
        let pool = PgPoolOptions::new()
            .max_connections(max_connections)
            .acquire_timeout(Duration::from_secs(5))
            .connect(url)
            .await
            .map_err(Error::from_internal)?;

        Ok(Self { pool })
    }

    /// Apply database migrations
    pub async fn migrate(&self) -> Result<(), Error> {
        sqlx::migrate!()
            .run(&self.pool)
            .await
            .map_err(Error::from_internal)?;
        Ok(())
    }
}

#[async_trait]
impl Repository for PostgresRepository {
    async fn fetch(&self, code: ShortCode) -> Result<FetchResult, Error> {
        let record = sqlx::query!(
            "SELECT * FROM links WHERE code = $1
                      AND (expires_at IS NULL OR expires_at >= NOW())
                      AND (clicks_limit IS NULL OR clicks_limit >= clicks)",
            code.0
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(Error::from_internal)?;

        match record {
            Some(record) => Ok(FetchResult {
                id: record.id,
                clicks: record.clicks,
                clicks_limit: record.clicks_limit,
                created_at: record.created_at,
                expires_at: record.expires_at,
                code: ShortCode(record.code),
                creator_ip: record.creator_ip,
                url: OriginalUrl(record.url),
            }),
            None => Err(Error::URLNotFound),
        }
    }

    async fn fetch_for_click(&self, code: ShortCode) -> Result<FetchResult, Error> {
        let record = sqlx::query!(
            "UPDATE links SET clicks = clicks + 1 WHERE code = $1
                        AND (expires_at IS NULL OR expires_at >= NOW())
                        AND (clicks_limit IS NULL OR clicks_limit >= clicks)
            RETURNING *",
            code.0
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(Error::from_internal)?;

        match record {
            Some(record) => Ok(FetchResult {
                id: record.id,
                clicks: record.clicks,
                clicks_limit: record.clicks_limit,
                created_at: record.created_at,
                expires_at: record.expires_at,
                code: ShortCode(record.code),
                creator_ip: record.creator_ip,
                url: OriginalUrl(record.url),
            }),
            None => Err(Error::URLNotFound),
        }
    }

    async fn save(&self, request: SaveRequest) -> Result<(), Error> {
        let result = sqlx::query!(
            "INSERT INTO links (code, url, clicks, clicks_limit, created_at, expires_at, creator_ip) VALUES ($1, $2, $3, $4, $5, $6, $7)",
            request.code.0,
            request.url.0,
            request.clicks,
            request.clicks_limit,
            request.created_at,
            request.expires_at,
            request.creator_ip
        )
            .execute(&self.pool)
            .await;

        match result {
            Ok(_) => Ok(()),
            Err(sqlx::Error::Database(db_err)) if db_err.code().as_deref() == Some("23505") => {
                Err(Error::DuplicateCode)
            }
            Err(e) => Err(Error::from_internal(e)),
        }
    }

    async fn cleanup_expired_links(&self) -> Result<u64, Error> {
        let res = sqlx::query!("DELETE FROM links WHERE expires_at < NOW()")
            .execute(&self.pool)
            .await
            .map_err(Error::from_internal)?;

        Ok(res.rows_affected())
    }

    async fn cleanup_links_exceeded_clicks_limit(&self) -> Result<u64, Error> {
        let res = sqlx::query!("DELETE FROM links WHERE clicks > clicks_limit")
            .execute(&self.pool)
            .await
            .map_err(Error::from_internal)?;

        Ok(res.rows_affected())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::*;
    use sqlx::{Row, query};

    fn request(code: ShortCode, url: OriginalUrl) -> SaveRequest {
        SaveRequest {
            url,
            code,
            expires_at: Some(Utc::now() + chrono::Duration::days(1)),
            clicks_limit: Some(100),
            creator_ip: "192.168.0.1".to_string(),
            clicks: 10,
            created_at: Utc::now(),
        }
    }

    #[sqlx::test]
    async fn fetch_url_by_not_existing_code(pool: PgPool) {
        let repo = PostgresRepository { pool };
        let result = repo.fetch(ShortCode("code".to_string())).await;

        assert!(matches!(result.err().unwrap(), Error::URLNotFound));
    }

    #[sqlx::test]
    async fn save_code(pool: PgPool) {
        let repo = PostgresRepository { pool };
        let result = repo.save(SaveRequest::default()).await;

        assert!(result.is_ok());
    }

    #[sqlx::test]
    async fn cant_save_same_code_twice(pool: PgPool) {
        let repo = PostgresRepository { pool };

        let req = request(
            ShortCode("code".to_string()),
            OriginalUrl("example.com".to_string()),
        );

        let result1 = repo.save(req.clone()).await;
        assert!(result1.is_ok());

        let result2 = repo.save(req.clone()).await;
        assert!(result2.is_err());
        assert!(matches!(result2, Err(Error::DuplicateCode)));
    }

    #[sqlx::test]
    async fn fetch_saved_code(pool: PgPool) {
        let repo = PostgresRepository { pool };

        let code = ShortCode("code".to_string());
        let url = OriginalUrl("example.com".to_string());

        let req = request(code.clone(), url);

        // -- test --

        let save_result = repo.save(req.clone()).await;
        assert!(save_result.is_ok());

        let fetch_result = repo.fetch(code).await.unwrap();

        assert_eq!(fetch_result.url, req.url);
        assert_eq!(fetch_result.code, req.code);

        let delta = fetch_result.created_at - req.created_at;
        assert!(delta.num_seconds().abs() < 1);

        assert_eq!(
            fetch_result.clicks_limit.unwrap(),
            req.clicks_limit.unwrap()
        );

        let delta = fetch_result.expires_at.unwrap() - req.expires_at.unwrap();
        assert!(delta.num_seconds().abs() < 1);

        assert_eq!(fetch_result.creator_ip, req.creator_ip);
        assert_eq!(fetch_result.clicks, req.clicks);
    }

    #[sqlx::test]
    async fn saved_codes_gets_different_ids(pool: PgPool) {
        let repo = PostgresRepository { pool };

        let code1 = ShortCode("code1".to_string());
        let code2 = ShortCode("code2".to_string());

        let url = OriginalUrl("example.com".to_string());

        // --------- requests ---------
        let req1 = request(code1.clone(), url.clone());
        let req2 = request(code2.clone(), url);

        // --------- save ---------
        let save_result1 = repo.save(req1).await;
        assert!(save_result1.is_ok());

        let save_result2 = repo.save(req2).await;
        assert!(save_result2.is_ok());

        // --------- fetch ---------
        let fetch_id = async |code: ShortCode| {
            query("SELECT id FROM links WHERE code = $1")
                .bind(code.0)
                .fetch_one(&repo.pool)
                .await
                .unwrap()
                .get::<i64, _>(0)
        };

        let id1 = fetch_id(code1).await;
        let id2 = fetch_id(code2).await;

        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
    }

    #[sqlx::test]
    async fn cant_fetch_expired_links(pool: PgPool) {
        let req = SaveRequest::default();

        let mut link = req.clone();
        let mut expired_link = req.clone();

        link.expires_at = None;
        link.code = ShortCode("code1".to_string());

        expired_link.expires_at = Some(Utc::now());
        expired_link.code = ShortCode("code2".to_string());

        // ---------- tests ----------
        let repo = PostgresRepository { pool };

        // ---------- save ----------
        repo.save(link).await.unwrap();
        repo.save(expired_link).await.unwrap();

        // ---------- fetch ----------
        assert!(repo.fetch(ShortCode("code1".to_string())).await.is_ok());

        assert!(matches!(
            repo.fetch(ShortCode("code2".to_string()))
                .await
                .err()
                .unwrap(),
            Error::URLNotFound
        ));
    }

    #[sqlx::test]
    async fn cant_fetch_links_exceeded_clicks_limit(pool: PgPool) {
        let req = SaveRequest::default();

        let mut link = req.clone();
        let mut expired_link = req.clone();
        let mut infinity_link = req.clone();

        link.clicks_limit = Some(10);
        link.clicks = 10;
        link.code = ShortCode("code1".to_string());

        infinity_link.clicks_limit = None;
        infinity_link.code = ShortCode("code2".to_string());

        expired_link.clicks_limit = Some(10);
        expired_link.clicks = 11;
        expired_link.code = ShortCode("code3".to_string());

        // ---------- tests ----------
        let repo = PostgresRepository { pool };

        // ---------- save ----------
        repo.save(link).await.unwrap();
        repo.save(expired_link).await.unwrap();
        repo.save(infinity_link).await.unwrap();

        // ---------- fetch ----------
        assert!(repo.fetch(ShortCode("code1".to_string())).await.is_ok());
        assert!(repo.fetch(ShortCode("code2".to_string())).await.is_ok());

        assert!(matches!(
            repo.fetch(ShortCode("code3".to_string()))
                .await
                .err()
                .unwrap(),
            Error::URLNotFound
        ));
    }

    #[sqlx::test]
    async fn fetch_for_click_increases_clicks_count(pool: PgPool) {
        let mut link = SaveRequest::default();

        link.clicks = 0;
        link.code = ShortCode("code1".to_string());

        // ---------- tests ----------
        let repo = PostgresRepository { pool };

        // ---------- save ----------
        repo.save(link).await.unwrap();

        // ---------- fetch ----------
        assert_eq!(
            repo.fetch(ShortCode("code1".to_string()))
                .await
                .unwrap()
                .clicks,
            0
        );

        // ---------- fetch_for_click ----------
        assert_eq!(
            repo.fetch_for_click(ShortCode("code1".to_string()))
                .await
                .unwrap()
                .clicks,
            1
        );

        // ---------- fetch ----------
        assert_eq!(
            repo.fetch(ShortCode("code1".to_string()))
                .await
                .unwrap()
                .clicks,
            1
        );
    }

    #[sqlx::test]
    async fn cleans_expired_links(pool: PgPool) {
        let req = SaveRequest::default();

        let mut expired_req_1 = req.clone();
        let mut expired_req_2 = req.clone();

        expired_req_1.expires_at = Some(Utc::now());
        expired_req_1.code = ShortCode("code1".to_string());

        expired_req_2.expires_at = Some(Utc::now() - chrono::Duration::seconds(1));
        expired_req_2.code = ShortCode("code2".to_string());

        let mut infinity_req = req.clone();
        let mut not_expired_req = req.clone();

        infinity_req.expires_at = None;
        infinity_req.code = ShortCode("code3".to_string());

        not_expired_req.expires_at = Some(Utc::now() + chrono::Duration::days(1));
        not_expired_req.code = ShortCode("code4".to_string());

        // ---------- tests ----------
        let repo = PostgresRepository { pool };

        // ---------- save ----------
        repo.save(expired_req_1).await.unwrap();
        repo.save(expired_req_2).await.unwrap();
        repo.save(infinity_req).await.unwrap();
        repo.save(not_expired_req).await.unwrap();

        // ---------- cleanup ----------
        let removed = repo.cleanup_expired_links().await.unwrap();
        assert_eq!(removed, 2);

        // ---------- fetch not removed ----------
        assert!(matches!(
            repo.fetch(ShortCode("code1".to_string()))
                .await
                .err()
                .unwrap(),
            Error::URLNotFound
        ));

        assert!(matches!(
            repo.fetch(ShortCode("code2".to_string()))
                .await
                .err()
                .unwrap(),
            Error::URLNotFound
        ));

        assert!(repo.fetch(ShortCode("code3".to_string())).await.is_ok());
        assert!(repo.fetch(ShortCode("code4".to_string())).await.is_ok());
    }

    #[sqlx::test]
    async fn cleans_links_exceeded_clicks_limit(pool: PgPool) {
        let req = SaveRequest::default();

        let mut exceeded_req_1 = req.clone();
        let mut exceeded_req_2 = req.clone();

        exceeded_req_1.clicks_limit = Some(5);
        exceeded_req_1.clicks = 6;
        exceeded_req_1.code = ShortCode("code1".to_string());

        exceeded_req_2.clicks_limit = Some(10);
        exceeded_req_2.clicks = 100;
        exceeded_req_2.code = ShortCode("code2".to_string());

        let mut infinity_req = req.clone();
        let mut not_exceeded_req = req.clone();

        infinity_req.clicks_limit = None;
        infinity_req.code = ShortCode("code3".to_string());

        not_exceeded_req.code = ShortCode("code4".to_string());
        not_exceeded_req.clicks_limit = Some(10);
        not_exceeded_req.clicks = 10;

        // ---------- tests ----------
        let repo = PostgresRepository { pool };

        // ---------- save ----------
        repo.save(exceeded_req_1).await.unwrap();
        repo.save(exceeded_req_2).await.unwrap();
        repo.save(infinity_req).await.unwrap();
        repo.save(not_exceeded_req).await.unwrap();

        // ---------- cleanup ----------
        let removed = repo.cleanup_links_exceeded_clicks_limit().await.unwrap();
        assert_eq!(removed, 2);

        // ---------- fetch not removed ----------
        assert!(matches!(
            repo.fetch(ShortCode("code1".to_string()))
                .await
                .err()
                .unwrap(),
            Error::URLNotFound
        ));

        assert!(matches!(
            repo.fetch(ShortCode("code2".to_string()))
                .await
                .err()
                .unwrap(),
            Error::URLNotFound
        ));

        assert!(repo.fetch(ShortCode("code3".to_string())).await.is_ok());
        assert!(repo.fetch(ShortCode("code4".to_string())).await.is_ok());
    }
}
