use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    let app = forum::app();

    axum::serve(listener, app).await
}
