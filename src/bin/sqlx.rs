use anyhow::{Context, Result};

use sqlx::postgres::PgPool;

// $ DATABASE_URL=postgres://jari:12345678@localhost/jari cargo run --bin sqlx
//
#[async_std::main]
async fn main() -> Result<()> {
    let db_url = std::env::var("DATABASE_URL").context("DATABASE_URL not set")?;
    let pool = PgPool::connect(&db_url).await?;
    let row: (i64,) = sqlx::query_as("SELECT $1")
        .bind(150_i64)
        .fetch_one(&pool).await?;
    dbg!(&row);
    assert_eq!(row.0, 150);

    Ok(())
}
