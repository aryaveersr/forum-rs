use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use sqlx::{FromRow, PgPool};
use thiserror::Error;
use uuid::Uuid;

#[tracing::instrument(name = "List Posts", skip(pool))]
pub async fn handler(State(pool): State<PgPool>) -> Result<Json<Vec<Post>>, Error> {
    let posts = get_posts(&pool).await?;

    Ok(Json(posts))
}

#[derive(Serialize, FromRow)]
pub struct Post {
    id: Uuid,
    title: String,
    content: String,
    username: String,
}

#[tracing::instrument(name = "Get post from database", skip_all)]
async fn get_posts(pool: &PgPool) -> Result<Vec<Post>, sqlx::Error> {
    let posts = sqlx::query_as!(
        Post,
        r#"
        SELECT
            posts.id,
            posts.title,
            posts.content,
            users.username
        FROM posts
        JOIN users
        ON posts.author_id = users.id"#,
    )
    .fetch_all(pool)
    .await?;

    Ok(posts)
}

#[derive(Debug, Error)]
#[error(transparent)]
pub enum Error {
    Database(#[from] sqlx::Error),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Error::Database(_) => {
                tracing::error!("{self}");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}
