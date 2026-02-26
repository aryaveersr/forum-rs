use axum::{Router, extract::Request};
use axum_macros::FromRef;
use sqlx::PgPool;
use tower::Layer;
use tower_http::{
    normalize_path::{NormalizePath, NormalizePathLayer},
    trace::TraceLayer,
};
use tracing::Level;
use uuid::Uuid;

mod config;
mod routes;
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
                Level::DEBUG,
                "request",
                method = %req.method(),
                uri = %req.uri(),
                req_id = %Uuid::new_v4().to_string().split('-').next_back().unwrap(),
            )
        }));

    NormalizePathLayer::trim_trailing_slash().layer(svc)
}
