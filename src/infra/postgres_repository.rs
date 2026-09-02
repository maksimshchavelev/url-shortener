use crate::domain::{Error, OriginalUrl, Repository, ShortCode};
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
    async fn fetch_url(&self, code: ShortCode) -> Result<OriginalUrl, Error> {
        let record = sqlx::query!("SELECT url FROM links WHERE code = $1", code.0)
            .fetch_optional(&self.pool)
            .await
            .map_err(Error::from_internal)?;

        match record {
            Some(url) => Ok(OriginalUrl(url.url)),
            None => Err(Error::URLNotFound),
        }
    }

    async fn save_code(&self, code: ShortCode, url: OriginalUrl) -> Result<(), Error> {
        let result = sqlx::query!(
            "INSERT INTO links (code, url) VALUES ($1, $2)",
            code.0,
            url.0
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::{Row, query};

    #[sqlx::test]
    async fn fetch_url_by_not_existing_code(pool: PgPool) {
        let repo = PostgresRepository { pool };
        let result = repo.fetch_url(ShortCode("code".to_string())).await;

        assert!(matches!(result.err().unwrap(), Error::URLNotFound));
    }

    #[sqlx::test]
    async fn save_code(pool: PgPool) {
        let repo = PostgresRepository { pool };
        let result = repo
            .save_code(
                ShortCode("code".to_string()),
                OriginalUrl("example.com".to_string()),
            )
            .await;

        assert!(result.is_ok());
    }

    #[sqlx::test]
    async fn cant_save_same_code_twice(pool: PgPool) {
        let repo = PostgresRepository { pool };

        let code = ShortCode("code".to_string());
        let url = OriginalUrl("example.com".to_string());

        let result1 = repo.save_code(code.clone(), url.clone()).await;
        assert!(result1.is_ok());

        let result2 = repo.save_code(code, url).await;
        assert!(result2.is_err());
        assert!(matches!(result2, Err(Error::DuplicateCode)));
    }

    #[sqlx::test]
    async fn fetch_saved_code(pool: PgPool) {
        let repo = PostgresRepository { pool };

        let code = ShortCode("code".to_string());
        let url = OriginalUrl("example.com".to_string());

        let save_result = repo.save_code(code.clone(), url.clone()).await;
        assert!(save_result.is_ok());

        let fetch_result = repo.fetch_url(code).await;
        assert!(fetch_result.is_ok());
        assert_eq!(fetch_result.unwrap(), url);
    }

    #[sqlx::test]
    async fn saved_codes_gets_different_ids(pool: PgPool) {
        let repo = PostgresRepository { pool };

        let code1 = ShortCode("code1".to_string());
        let code2 = ShortCode("code2".to_string());

        let url = OriginalUrl("example.com".to_string());

        // --------- save ---------
        let save_result1 = repo.save_code(code1.clone(), url.clone()).await;
        assert!(save_result1.is_ok());

        let save_result2 = repo.save_code(code2.clone(), url.clone()).await;
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
}
