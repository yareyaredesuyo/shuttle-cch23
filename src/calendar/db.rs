use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Clone)]
pub struct Pool {
    pub pool: PgPool,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Order {
    pub id: i32,
    pub region_id: i32,
    pub gift_name: String,
    pub quantity: i32,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Region {
    pub id: i32,
    pub name: String,
}
