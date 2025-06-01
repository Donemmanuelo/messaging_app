use redis::{Client, Commands, RedisError};
use std::sync::Arc;

pub struct RedisService {
    client: Arc<Client>,
}

impl RedisService {
    pub fn new(url: &str) -> Result<Self, RedisError> {
        let client = Client::open(url)?;
        Ok(Self {
            client: Arc::new(client),
        })
    }

    pub fn set(&self, key: &str, value: &str, expiry: usize) -> Result<(), RedisError> {
        let mut conn = self.client.get_connection()?;
        conn.set_ex(key, value, expiry)
    }

    pub fn get(&self, key: &str) -> Result<Option<String>, RedisError> {
        let mut conn = self.client.get_connection()?;
        conn.get(key)
    }

    pub fn del(&self, key: &str) -> Result<(), RedisError> {
        let mut conn = self.client.get_connection()?;
        conn.del(key)
    }
} 