let db_url = env::var("DATABASE_URL").map_err(|_| "DATABASE_URL must be set")?;
let pool = PgPoolOptions::new()
    .max_connections(5)
    .connect(&db_url)
    .await
    .map_err(|_| "Failed to create pool")?; 