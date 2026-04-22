//! SQLite persistence.
//!
//! - Schema is defined via `sqlx` migrations in `../../migrations/`.
//! - All queries are async; we use runtime `sqlx::query*` for Phase 1 and
//!   will move hot paths to the compile-time checked `sqlx::query!` macros
//!   once the schema stabilizes.
//! - Migrations are versioned and idempotent (see CLAUDE.md rule 8).

pub mod models;
pub mod queries;

use std::path::Path;

use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;

#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("sqlx error: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("migration failed: {0}")]
    Migrate(#[from] sqlx::migrate::MigrateError),
}

/// Open (or create) the SQLite database at `path` and run all pending
/// migrations. The parent directory must already exist.
pub async fn open(path: &Path) -> Result<SqlitePool, DbError> {
    let opts = SqliteConnectOptions::new()
        .filename(path)
        .create_if_missing(true)
        .foreign_keys(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .busy_timeout(std::time::Duration::from_secs(5));

    let pool = SqlitePoolOptions::new()
        .max_connections(8)
        .connect_with(opts)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    tracing::info!(db = %path.display(), "orbit database ready");
    Ok(pool)
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn memory_pool() -> SqlitePool {
        let opts = SqliteConnectOptions::new()
            .in_memory(true)
            .create_if_missing(true)
            .foreign_keys(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(opts)
            .await
            .unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        pool
    }

    #[tokio::test]
    async fn migration_creates_expected_tables() {
        let pool = memory_pool().await;
        let rows: Vec<(String,)> = sqlx::query_as(
            "SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE '\\_%' ESCAPE '\\' ORDER BY name",
        )
        .fetch_all(&pool)
        .await
        .unwrap();
        let names: Vec<String> = rows.into_iter().map(|(n,)| n).collect();
        assert!(names.contains(&"agents".to_string()));
        assert!(names.contains(&"conversations".to_string()));
        assert!(names.contains(&"messages".to_string()));
    }

    #[tokio::test]
    async fn migration_is_idempotent() {
        // Re-running migrations against a fresh pool (which has the baseline
        // applied) should be a no-op — the migration runner tracks what's
        // been applied via the `_sqlx_migrations` table.
        let pool = memory_pool().await;
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    }
}
