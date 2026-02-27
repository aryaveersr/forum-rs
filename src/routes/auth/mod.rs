use axum::{Router, routing::post};

use crate::AppState;

mod login;
mod register;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/login", post(login::handler))
        .route("/register", post(register::handler))
}
