use axum::{
    Router,
    routing::{get, post},
};

use crate::AppState;

mod create;
mod delete;
mod get;
mod list;
mod update;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", post(create::handler))
        .route("/list", get(list::handler))
        .route(
            "/{slug}",
            get(get::handler)
                .delete(delete::handler)
                .patch(update::handler),
        )
}
