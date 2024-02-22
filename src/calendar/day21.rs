use super::error::AppError;
use axum::{extract::Path, routing::get, Router};
use country_boundaries::{CountryBoundaries, LatLon, BOUNDARIES_ODBL_360X180};
use dms_coordinates::DMS;
use s2::{cell::Cell, cellid::CellID};

pub fn task() -> Router {
    Router::new()
        .route("/coords/:id", get(coords_route))
        .route("/country/:id", get(country_route))
}

async fn coords_route(Path(id): Path<String>) -> Result<String, AppError> {
    let id = u64::from_str_radix(&id, 2)?;
    let center = Cell::from(CellID(id)).center();

    let lat = DMS::from_ddeg_latitude(center.latitude().deg());
    let lng = DMS::from_ddeg_longitude(center.longitude().deg());

    Ok(format!(
        "{}°{}'{:.3}''{} {}°{}'{:.3}''{}",
        lat.degrees,
        lat.minutes,
        lat.seconds,
        lat.cardinal.unwrap(),
        lng.degrees,
        lng.minutes,
        lng.seconds,
        lng.cardinal.unwrap()
    ))
}

async fn country_route(Path(id): Path<String>) -> Result<String, AppError> {
    let id = u64::from_str_radix(&id, 2)?;
    let center = Cell::from(CellID(id)).center();

    let lat = center.latitude().deg();
    let lng = center.longitude().deg();

    let boundaries = CountryBoundaries::from_reader(BOUNDARIES_ODBL_360X180)?;
    let ids = boundaries.ids(LatLon::new(lat, lng)?);

    let country_id = ids.last().unwrap();
    let country = isocountry::CountryCode::for_alpha2(country_id)?.name();

    Ok(format!(
        "{}",
        country.split_ascii_whitespace().next().unwrap()
    ))
}
