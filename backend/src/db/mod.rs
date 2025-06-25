use dotenvy::dotenv;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::env;

pub async fn get_pg_pool() -> PgPool {
    dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .expect("Failed to create pool")
}
