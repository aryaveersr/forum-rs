use axum::{Router, routing::get};

use crate::AppState;

mod delete;
mod get;
mod update;

pub fn routes() -> Router<AppState> {
    Router::new().route(
        "/{username}",
        get(get::handler)
            .patch(update::handler)
            .delete(delete::handler),
    )
}
