use axum::{ServiceExt, extract::Request};
use forum::CONFIG;
use sqlx::PgPool;
use tokio::net::TcpListener;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    tracing_subscriber::fmt::init();

    let pool = PgPool::connect(&CONFIG.database.conn_string())
        .await
        .expect("Failed to connect to database.");

    let app = forum::app(pool);
    let listener = TcpListener::bind(format!("0.0.0.0:{}", CONFIG.port)).await?;

    info!("Starting server on port {}.", CONFIG.port);

    axum::serve(listener, ServiceExt::<Request>::into_make_service(app)).await
}
