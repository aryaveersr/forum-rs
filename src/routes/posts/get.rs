use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use sqlx::{FromRow, PgPool};
use thiserror::Error;
use uuid::Uuid;

use crate::domain::post::slug::Slug;

#[tracing::instrument(name = "Get Post", skip(pool))]
pub async fn handler(
    State(pool): State<PgPool>,
    Path(slug): Path<Slug>,
) -> Result<Json<Post>, Error> {
    let post = get_post(&pool, slug).await?.ok_or(Error::DoesNotExist)?;

    Ok(Json(post))
}

#[derive(Serialize, FromRow)]
pub struct Post {
    id: Uuid,
    title: String,
    content: String,
}

#[tracing::instrument(skip_all)]
async fn get_post(pool: &PgPool, slug: Slug) -> Result<Option<Post>, sqlx::Error> {
    let post = sqlx::query_as!(
        Post,
        r#"SELECT id, title, content FROM posts WHERE slug = $1"#,
        slug.as_ref()
    )
    .fetch_optional(pool)
    .await?;

    Ok(post)
}

#[derive(Debug, Error)]
#[error(transparent)]
pub enum Error {
    Database(#[from] sqlx::Error),

    #[error("Post does not exist")]
    DoesNotExist,
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Error::DoesNotExist => StatusCode::NOT_FOUND.into_response(),

            Error::Database(_) => {
                tracing::error!("{self}");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}
