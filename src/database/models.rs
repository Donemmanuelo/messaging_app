use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Insertable, AsChangeset, Serialize, Deserialize)]
#[table_name = "users"]
pub struct User {
    pub id: Option<i32>,
    pub username: String,
    pub password: String,
}

#[derive(Queryable, Insertable, AsChangeset, Serialize, Deserialize)]
#[table_name = "messages"]
pub struct Message {
    pub id: Option<i32>,
    pub from_user_id: i32,
    pub to_user_id: i32,
    pub content: String,
    pub timestamp: chrono::NaiveDateTime,
}

#[derive(Queryable, Insertable, AsChangeset, Serialize, Deserialize)]
#[table_name = "chats"]
pub struct Chat {
    pub id: Option<i32>,
    pub user1_id: i32,
    pub user2_id: i32,
}