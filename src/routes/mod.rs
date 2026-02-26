use axum::{Router, routing::get};

mod health_check;

pub fn routes() -> Router {
    Router::new().route("/health_check", get(health_check::handler))
}
