use axum::Router;

use shuttle_runtime::CustomError;
use sqlx::PgPool;

mod calendar;
use calendar::db::Pool;

#[shuttle_runtime::main]
async fn main(#[shuttle_shared_db::Postgres] pool: PgPool) -> shuttle_axum::ShuttleAxum {
    sqlx::migrate!()
        .run(&pool)
        .await
        .map_err(CustomError::new)?;

    let pg_state = Pool { pool };

    let router = Router::new()
        .nest("/", calendar::day_1::task())
        .nest("/1", calendar::day1::task())
        .nest("/4", calendar::day4::task())
        .nest("/5", calendar::day5::task())
        .nest("/6", calendar::day6::task())
        .nest("/7", calendar::day7::task())
        .nest("/8", calendar::day8::task())
        .nest("/11", calendar::day11::task())
        .nest("/12", calendar::day12::task())
        .nest("/13", calendar::day13::task(pg_state.clone()))
        .nest("/14", calendar::day14::task())
        .nest("/15", calendar::day15::task())
        .nest("/18", calendar::day18::task(pg_state))
        .nest("/19", calendar::day19::task())
        .nest("/20", calendar::day20::task())
        .nest("/21", calendar::day21::task())
        .nest("/22", calendar::day22::task());

    Ok(router.into())
}
