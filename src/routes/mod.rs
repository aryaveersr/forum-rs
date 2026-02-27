use axum::{Router, routing::get};

use crate::AppState;

mod auth;
mod guarded_test;
mod health_check;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/health_check", get(health_check::handler))
        .route("/test", get(guarded_test::handler))
        .nest("/auth", auth::routes())
}
