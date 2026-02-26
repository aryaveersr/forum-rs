use axum::Router;
use axum_macros::FromRef;
use sqlx::PgPool;

mod config;
mod routes;

pub use config::CONFIG;

#[derive(Clone, FromRef)]
pub struct AppState {
    pool: sqlx::PgPool,
}

pub fn app(pool: PgPool) -> Router {
    let state = AppState { pool };

    Router::new()
        .nest("/api", routes::routes())
        .with_state(state)
}
