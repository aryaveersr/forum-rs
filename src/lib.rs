use axum::{Router, extract::Request};
use axum_macros::FromRef;
use sqlx::PgPool;
use tower::Layer;
use tower_http::{
    normalize_path::{NormalizePath, NormalizePathLayer},
    trace::TraceLayer,
};

mod auth;
mod config;
mod models;
mod routes;
mod utils;

pub use config::CONFIG;

#[derive(Clone, FromRef)]
pub struct AppState {
    pool: sqlx::PgPool,
}

pub fn app(pool: PgPool) -> NormalizePath<Router> {
    let state = AppState { pool };

    let svc = Router::new()
        .nest("/api", routes::routes())
        .with_state(state)
        .layer(TraceLayer::new_for_http().make_span_with(|req: &Request| {
            tracing::span!(
                tracing::Level::DEBUG,
                "request",
                method = %req.method(),
                uri = %req.uri(),
                req_id = %utils::random_string(),
            )
        }));

    NormalizePathLayer::trim_trailing_slash().layer(svc)
}
