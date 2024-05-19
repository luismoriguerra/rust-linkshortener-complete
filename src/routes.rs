use axum::{
    body::Body,
    extract::{Path, State},
    response::{IntoResponse, Response},
};
use reqwest::StatusCode;
use serde::Serialize;

use crate::utils::internal_error;

const DEFAULT_CACHE_CONTROL_HEADER_VALUE: &str =
    "public, max-age=300, s-maxage=300, stale-while-revalidate=300, stale-if-error=300";

#[derive(Debug, serde::Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Link {
    pub id: String,
    pub target_url: String,
}

pub async fn redirect(
    State(db): State<sqlx::PgPool>,
    Path(requested_link): Path<String>,
) -> Result<Response, (StatusCode, String)> {
    let timeout = tokio::time::Duration::from_millis(300);

    let request = sqlx::query_as!(
        Link,
        "SELECT id, target_url FROM links WHERE id = $1",
        requested_link
    )
    .fetch_optional(&db);

    let link_timeout = tokio::time::timeout(timeout, request);

    let link: Link = link_timeout
        .await
        .map_err(internal_error)?
        .map_err(internal_error)?
        .ok_or_else(|| "Not Founda".to_string())
        .map_err(|e| (StatusCode::NOT_FOUND, e))?;

    tracing::debug!(
        "Redirecting link id {} to {}",
        requested_link,
        link.target_url
    );

    Ok(Response::builder()
        .status(StatusCode::TEMPORARY_REDIRECT)
        .header("Location", link.target_url)
        .header("Cache-Control", DEFAULT_CACHE_CONTROL_HEADER_VALUE)
        .body(Body::empty())
        .expect("Failed to build response"))
}

pub async fn health() -> impl IntoResponse {
    (StatusCode::OK, "Serice is healthy!")
}
