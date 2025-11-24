use sqlx::{
    sqlite::{SqliteArguments, SqlitePoolOptions},
    Sqlite, SqlitePool, Transaction,
};
use std::time::Duration;
use tokio::sync::OnceCell;

static DB: OnceCell<DB> = OnceCell::const_new();

pub struct DBOption {
    pub max_connections: u32,
    pub acquire_timeout: Duration,
}

pub struct DB {
    pool: SqlitePool,
    transaction: Option<Transaction<'static, Sqlite>>,
}

impl DB {
    async fn new(db_url: &str, option: DBOption) -> Self {
        let pool = SqlitePoolOptions::new()
            .max_connections(option.max_connections)
            .acquire_timeout(option.acquire_timeout)
            .connect(db_url)
            .await
            .expect("Cannot connect to database");

        DB {
            pool,
            transaction: None,
        }
    }

    pub async fn get() -> &'static DB {
        DB.get_or_init(async || {
            dotenvy::dotenv().expect("Failed to load dotenv");
            let db_url = std::env::var("DATABASE_URL").expect("Failed to get URL");
            let db_option = DBOption {
                max_connections: 5,
                acquire_timeout: Duration::from_secs(3),
            };
            DB::new(&db_url, db_option).await
        })
        .await
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    pub async fn execute<'a>(
        &mut self,
        query: sqlx::query::Query<'a, Sqlite, SqliteArguments<'a>>,
    ) -> Result<(), sqlx::error::Error> {
        if let Some(ref mut t) = self.transaction {
            query.execute(&mut **t).await?;
        } else {
            query.execute(&self.pool).await?;
        }
        Ok(())
    }
}
