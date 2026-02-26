use axum::{Router, routing::get};

use crate::AppState;

mod health_check;

pub fn routes() -> Router<AppState> {
    Router::new().route("/health_check", get(health_check::handler))
}
