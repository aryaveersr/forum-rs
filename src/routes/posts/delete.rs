use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use sqlx::PgPool;
use thiserror::Error;
use uuid::Uuid;

use crate::{domain::post::slug::Slug, session::Session};

#[tracing::instrument(name = "Delete Post", skip(pool))]
pub async fn handler(
    session: Session,
    State(pool): State<PgPool>,
    Path(slug): Path<Slug>,
) -> Result<StatusCode, Error> {
    let author = get_post_author(&pool, &slug)
        .await?
        .ok_or(Error::DoesNotExist)?;

    if session.user_id != author {
        return Err(Error::Unauthorized);
    }

    delete_post(&pool, slug).await?;
    Ok(StatusCode::OK)
}

#[tracing::instrument(skip_all)]
async fn get_post_author(pool: &PgPool, slug: &Slug) -> Result<Option<Uuid>, sqlx::Error> {
    let row = sqlx::query_scalar!("SELECT author_id FROM posts WHERE slug = $1", slug.as_ref())
        .fetch_optional(pool)
        .await?;

    Ok(row)
}

#[tracing::instrument(skip_all)]
async fn delete_post(pool: &PgPool, slug: Slug) -> Result<(), sqlx::Error> {
    sqlx::query!("DELETE FROM posts WHERE slug = $1", slug.as_ref())
        .execute(pool)
        .await?;

    Ok(())
}

#[derive(Debug, Error)]
#[error(transparent)]
pub enum Error {
    Database(#[from] sqlx::Error),

    #[error("Post does not exist")]
    DoesNotExist,

    #[error("Unauthorized to delete post")]
    Unauthorized,
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Error::Unauthorized => StatusCode::UNAUTHORIZED.into_response(),
            Error::DoesNotExist => StatusCode::NOT_FOUND.into_response(),

            Error::Database(_) => {
                tracing::error!("{self}");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}
