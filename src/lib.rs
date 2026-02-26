use axum::Router;

mod routes;

pub fn app() -> Router {
    Router::new().nest("/api", routes::routes())
}
