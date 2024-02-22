use super::error::AppError;
use axum::{extract::Multipart, routing::post, Router};
use image::io::Reader as ImageReader;
use std::io::Cursor;
use tower_http::services::ServeDir;

pub fn task() -> Router {
    Router::new()
        .nest_service("/assets", ServeDir::new("assets"))
        .route("/red_pixels", post(red_pixels_route))
}

async fn red_pixels_route(mut multipart: Multipart) -> Result<String, AppError> {
    let mut red_pixel_count = 0;

    while let Some(field) = multipart.next_field().await.unwrap() {
        let data = field.bytes().await.unwrap();

        let img: image::DynamicImage = ImageReader::new(Cursor::new(data))
            .with_guessed_format()?
            .decode()?;

        red_pixel_count += img
            .to_rgb8()
            .enumerate_pixels()
            .fold(0, |count, (_, _, pixel)| {
                let (r, g, b) = (pixel[0] as u32, pixel[1] as u32, pixel[2] as u32);
                count + if r > g + b { 1 } else { 0 }
            });
    }

    Ok(red_pixel_count.to_string())
}
