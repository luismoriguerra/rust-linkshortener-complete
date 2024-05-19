use axum::response::IntoResponse;
use reqwest::StatusCode;

pub async fn health() -> impl IntoResponse {
    (StatusCode::OK, "Serice is healthy!")
}
