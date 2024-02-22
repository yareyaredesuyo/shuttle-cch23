use axum::{
    extract::State,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};

use super::db::{Order, Pool};
use super::error::AppError;

pub fn task(pool: Pool) -> Router {
    Router::new()
        .route("/sql", get(sql_route))
        .route("/reset", post(reset_route))
        .route("/orders", post(orders_route))
        .route("/orders/total", get(orders_total_route))
        .route("/orders/popular", get(orders_popular_route))
        .with_state(pool)
}

async fn sql_route(State(pool): State<Pool>) -> Result<String, AppError> {
    let number = sqlx::query!("SELECT 20231213 number")
        .fetch_one(&pool.pool)
        .await?
        .number
        .unwrap();

    Ok(number.to_string())
}

async fn reset_route(State(pool): State<Pool>) -> Result<(), AppError> {
    sqlx::query!("DROP TABLE IF EXISTS orders")
        .execute(&pool.pool)
        .await?;

    sqlx::query!(
        "CREATE TABLE orders (
              id INT PRIMARY KEY,
              region_id INT,
              gift_name VARCHAR(50),
              quantity INT
        )"
    )
    .execute(&pool.pool)
    .await?;

    Ok(())
}

async fn orders_route(
    State(pool): State<Pool>,
    Json(data): Json<Vec<Order>>,
) -> Result<(), AppError> {
    for order in data {
        sqlx::query!(
            "INSERT INTO orders (id, region_id, gift_name, quantity) VALUES ($1, $2, $3, $4)",
            order.id,
            order.region_id,
            order.gift_name,
            order.quantity,
        )
        .execute(&pool.pool)
        .await?;
    }
    Ok(())
}

async fn orders_total_route(State(pool): State<Pool>) -> Result<impl IntoResponse, AppError> {
    if let Ok(record) = sqlx::query!(r#"SELECT SUM(quantity) as "total!" FROM orders"#)
        .fetch_one(&pool.pool)
        .await
    {
        Ok(Json(serde_json::json!({"total": record.total})))
    } else {
        Ok(Json(serde_json::json!({"total": 0})))
    }
}

async fn orders_popular_route(State(pool): State<Pool>) -> impl IntoResponse {
    if let Ok(record) = sqlx::query!(
        r#"SELECT gift_name as "popular!", SUM(quantity) AS gift_count
          FROM orders
          GROUP BY gift_name
          ORDER BY gift_count DESC
          LIMIT 1"#
    )
    .fetch_one(&pool.pool)
    .await
    {
        Json(serde_json::json!({"popular": record.popular}))
    } else {
        Json(serde_json::json!({"popular": null}))
    }
}
