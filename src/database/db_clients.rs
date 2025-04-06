use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use dotenv::dotenv;
use std::env;
use log::{info, error};

pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

pub fn establish_connection() -> Pool {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    info!("Connecting to database at: {}", database_url);
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    match r2d2::Pool::builder().build(manager) {
        Ok(pool) => {
            info!("Database connection established successfully");
            pool
        }
        Err(e) => {
            error!("Failed to create pool: {}", e);
            panic!("Failed to create pool: {}", e);
        }
    }
}