use forum::CONFIG;
use sqlx::PgPool;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let pool = PgPool::connect(&CONFIG.database.conn_string())
        .await
        .expect("Failed to connect to database.");

    let app = forum::app(pool);
    let listener = TcpListener::bind(format!("0.0.0.0:{}", CONFIG.port)).await?;

    axum::serve(listener, app).await
}
