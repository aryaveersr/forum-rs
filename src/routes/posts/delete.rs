use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use sqlx::PgPool;
use thiserror::Error;
use uuid::Uuid;

use crate::session::Session;

#[tracing::instrument(name = "Delete Post", skip(pool))]
pub async fn handler(
    session: Session,
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, Error> {
    let post_author = get_post_author(&pool, id)
        .await?
        .ok_or(Error::DoesNotExist)?;

    if session.user_id != post_author {
        return Err(Error::Unauthorized);
    }

    delete_post(&pool, id).await?;
    Ok(StatusCode::OK)
}

#[tracing::instrument(skip_all)]
async fn get_post_author(pool: &PgPool, id: Uuid) -> Result<Option<Uuid>, sqlx::Error> {
    let row = sqlx::query_scalar!("SELECT author_id FROM posts WHERE id = $1", id)
        .fetch_optional(pool)
        .await?;

    Ok(row)
}

#[tracing::instrument(skip_all)]
async fn delete_post(pool: &PgPool, id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query!("DELETE FROM posts WHERE id = $1", id)
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
