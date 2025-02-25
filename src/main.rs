use std::error::Error;

use axum::{
    middleware,
    routing::{get, patch, post},
    Router,
};
use axum_prometheus::PrometheusMetricLayer;
use dotenvy::dotenv;
use routes::health;
use sqlx::postgres::PgPoolOptions;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{fmt::layer, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crate::auth::auth;
use crate::routes::{create_link, get_link_statistics, redirect, update_link};
mod auth;
mod routes;
mod utils;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL is not set");

    let db = PgPoolOptions::new()
        .max_connections(20)
        .connect(&db_url)
        .await
        .expect("Failed to connect to database");

    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("link_shorterner=debug")),
        )
        .with(layer())
        .init();

    let (prometheus_layer, metric_handle) = PrometheusMetricLayer::pair();

    let app = Router::new()
        .route("/create", post(create_link))
        .route("/:id/statistics", get(get_link_statistics))
        .route_layer(middleware::from_fn_with_state(db.clone(), auth))
        .route(
            "/:id",
            patch(update_link)
                .route_layer(middleware::from_fn_with_state(db.clone(), auth))
                .get(redirect),
        )
        .route("/metrics", get(|| async move { metric_handle.render() }))
        .route("/health", get(health))
        .layer(TraceLayer::new_for_http())
        .layer(prometheus_layer)
        .with_state(db);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind");

    tracing::debug!(
        "Server is running on http://{}",
        listener.local_addr().expect("Failed to get local address")
    );

    axum::serve(listener, app).await.expect("Server failed");

    Ok(())
}
