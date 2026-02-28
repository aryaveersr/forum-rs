use axum::{
    Router,
    routing::{get, post},
};

use crate::AppState;

mod create;
mod get;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", post(create::handler))
        .route("/{id}", get(get::handler))
}
