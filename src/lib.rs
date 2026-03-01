use axum::{Router, extract::Request};
use axum_macros::FromRef;
use sqlx::PgPool;
use tower::Layer;
use tower_http::{
    normalize_path::{NormalizePath, NormalizePathLayer},
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};

mod config;
mod domain;
mod routes;
mod session;
mod utils;

pub use config::CONFIG;

#[derive(Clone, FromRef)]
pub struct AppState {
    pool: sqlx::PgPool,
}

pub fn app(pool: PgPool) -> NormalizePath<Router> {
    let state = AppState { pool };

    let fallback = {
        let index_html = ServeFile::new("frontend/build/index.html");
        ServeDir::new("frontend/build").fallback(index_html)
    };

    let svc = Router::new()
        .nest("/api", routes::routes())
        .fallback_service(fallback)
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
