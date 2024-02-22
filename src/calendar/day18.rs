use axum::{
    extract::{Path, State},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};

use serde::Serialize;

use super::db::{Order, Pool, Region};
use super::error::AppError;

pub fn task(pool: Pool) -> Router {
    Router::new()
        .route("/reset", post(reset_route))
        .route("/orders", post(orders_route))
        .route("/regions", post(regions_route))
        .route("/regions/total", get(regions_total_route))
        .route("/regions/top_list/:num", get(regions_toplist_route))
        .with_state(pool)
}

async fn reset_route(State(pool): State<Pool>) -> Result<(), AppError> {
    sqlx::query!("drop table if exists regions")
        .execute(&pool.pool)
        .await?;

    sqlx::query!("drop table if exists orders")
        .execute(&pool.pool)
        .await?;

    sqlx::query!(
        "create table regions (
          id INT PRIMARY KEY,
          name VARCHAR(50)
      )"
    )
    .execute(&pool.pool)
    .await?;
    sqlx::query!(
        "create table orders (
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
    Json(orders): Json<Vec<Order>>,
) -> Result<(), AppError> {
    for order in orders {
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

async fn regions_route(
    State(pool): State<Pool>,
    Json(regions): Json<Vec<Region>>,
) -> Result<(), AppError> {
    println!("{:?}", pool.pool);
    for Region { id, name } in regions {
        sqlx::query!("insert into regions (id, name) VALUES ($1, $2)", id, name)
            .execute(&pool.pool)
            .await?;
    }
    Ok(())
}

async fn regions_total_route(State(pool): State<Pool>) -> impl IntoResponse {
    if let Ok(records) = sqlx::query!(
        r#"SELECT regions.name as "region_name!", SUM(orders.quantity) as "total!" FROM orders
  INNER JOIN regions ON orders.region_id=regions.id GROUP BY regions.name ORDER BY regions.name"#
    )
    .fetch_all(&pool.pool)
    .await
    {
        Json(serde_json::json!(records
            .iter()
            .map(|r| {
                serde_json::json!({
                  "region": r.region_name,
                  "total": r.total})
            })
            .collect::<Vec<_>>()))
    } else {
        Json(serde_json::json!({"region": null, "total": 0}))
    }
}

#[derive(Serialize)]
struct TopList {
    region: String,
    top_gifts: Vec<String>,
}

async fn regions_toplist_route(
    Path(number): Path<i64>,
    State(pool): State<Pool>,
) -> Result<impl IntoResponse, AppError> {
    let top_lists = sqlx::query_as!(
      TopList,
      r#"select r.name as "region!", COALESCE(NULLIF(ARRAY_AGG(o.gift_name), '{NULL}'), '{}'::text[]) AS "top_gifts!" from regions r left join LATERAL  
      (
          select gift_name, sum(quantity) as sum_quantity from orders where r.id = region_id group by gift_name, region_id order by sum_quantity desc limit $1
      ) o on true group by r.name order by r.name
       ;"#, number
  )
  .fetch_all(&pool.pool)
  .await?;

    Ok(Json(top_lists))
}
