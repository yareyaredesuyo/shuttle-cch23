use axum::{http::StatusCode, response::IntoResponse, routing::post, Json, Router};

use regex::Regex;
use serde::Deserialize;
use serde_json::json;
use sha256;

pub fn task() -> Router {
    Router::new()
        .route("/nice", post(nice_route))
        .route("/game", post(game_route))
}

#[derive(Deserialize, Debug)]
struct Nice {
    input: String,
}

async fn nice_route(Json(body): Json<Nice>) -> impl IntoResponse {
    if is_nice_string(&body.input) {
        (StatusCode::OK, Json(json!({ "result": "nice" })))
    } else {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({ "result": "naughty" })),
        )
    }
}

fn is_nice_string(s: &str) -> bool {
    // Check for at least three vowels
    let vowel_count = s.chars().filter(|c| "aeiouy".contains(*c)).count();
    if vowel_count < 3 {
        return false;
    }

    // Check for at least one letter that appears twice in a row

    let has_double_letter = s
        .as_bytes()
        .windows(2)
        .any(|w| w[0] == w[1] && w[0].is_ascii_alphabetic());

    if !has_double_letter {
        return false;
    }

    // Check for substrings ab, cd, pq, or xy
    let forbidden_substrings = ["ab", "cd", "pq", "xy"];
    if forbidden_substrings.iter().any(|substr| s.contains(substr)) {
        return false;
    }

    // If all conditions are met, it's a Nice String
    true
}

async fn game_route(Json(body): Json<Nice>) -> impl IntoResponse {
    let input = body.input;

    // rule1: must be at least 8 characters long
    if input.len() < 8 {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "result": "naughty", "reason": "8 chars" })),
        );
    }

    // rule2: must contain uppercase letters, lowercase letters, and digits
    if !(input.chars().any(|c| c.is_ascii_uppercase())
        && input.chars().any(|c| c.is_ascii_lowercase())
        && input.chars().any(|c| c.is_ascii_digit()))
    {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "result": "naughty", "reason": "more types of chars" })),
        );
    }

    // rule3: must contain at least 5 digits
    if input.chars().filter(char::is_ascii_digit).count() < 5 {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "result": "naughty", "reason": "55555" })),
        );
    }

    // rule4: all integers (sequences of consecutive digits) in the string must add up to 2023

    if Regex::new(r"\d+")
        .expect("not valid regex")
        .captures_iter(&input)
        .map(|c| c.extract::<0>().0)
        .map(|s| s.parse::<i32>().expect("parse error"))
        .sum::<i32>()
        != 2023
    {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "result": "naughty", "reason": "math is hard" })),
        );
    }

    // rule5: must contain the letters j, o, and y in that order and in no other order

    if !Regex::new(r"^([^joy]*)j([^joy]*)o([^joy]*)y([^joy]*)$")
        .expect("invalid regex pattern")
        .is_match(&input)
    {
        return (
            StatusCode::NOT_ACCEPTABLE,
            Json(json!({ "result": "naughty", "reason": "not joyful enough" })),
        );
    }

    // rule6: must contain a letter that repeats with exactly one other letter between them (like xyx)
    if !input.as_bytes().windows(3).any(|c| {
        c[0] != c[1] && c[0] == c[2] && c[0].is_ascii_alphabetic() && c[1].is_ascii_alphabetic()
    }) {
        return (
            StatusCode::UNAVAILABLE_FOR_LEGAL_REASONS,
            Json(json!({ "result": "naughty", "reason": "illegal: no sandwich" })),
        );
    }

    // rule7: must contain at least one unicode character in the range [U+2980, U+2BFF]
    if !Regex::new(r"[\u{2980}-\u{2BFF}]")
        .expect("Invalid regex pattern")
        .is_match(&input)
    {
        return (
            StatusCode::RANGE_NOT_SATISFIABLE,
            Json(json!({ "result": "naughty", "reason": "outranged" })),
        );
    }

    // rule8: must contain at least one emoji
    if !regex::Regex::new(r"[\p{Emoji}--\p{Ascii}]")
        .expect("not valid regex")
        .is_match(&input)
    {
        return (
            StatusCode::UPGRADE_REQUIRED,
            Json(json!({ "result": "naughty", "reason": "😳" })),
        );
    }

    // rule9: the hexadecimal representation of the sha256 hash of the string must end with an a
    if !sha256::digest(input.clone()).ends_with('a') {
        return (
            StatusCode::IM_A_TEAPOT,
            Json(json!({ "result": "naughty", "reason": "not a coffee brewer" })),
        );
    }

    (
        StatusCode::OK,
        Json(json!({"result": "nice", "reason": "that's a nice password"})),
    )
}
