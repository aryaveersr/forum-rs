use axum::{Router, routing::post};

use crate::AppState;

mod create;

pub fn routes() -> Router<AppState> {
    Router::new().route("/", post(create::handler))
}
