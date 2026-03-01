use axum::{
    Router,
    routing::{delete, get, post},
};

use crate::AppState;

mod create;
mod delete;
mod get;
mod update;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", post(create::handler))
        .route("/get/{slug}", get(get::handler))
        .route("/{id}", delete(delete::handler).patch(update::handler))
}
