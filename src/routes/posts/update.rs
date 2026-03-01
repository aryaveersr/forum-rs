use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use sqlx::PgPool;
use thiserror::Error;
use uuid::Uuid;

use crate::{
    auth::Session,
    domain::post::{content::Content, slug::Slug, title::Title},
};

#[derive(Deserialize)]
pub struct Body {
    title: Title,
    content: Content,
    slug: Slug,
}

#[tracing::instrument(
    name = "Update Post",
    fields(
        title = body.title.as_ref(),
        slug = body.slug.as_ref()
    ),
    skip(pool, body)
)]
pub async fn handler(
    session: Session,
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(body): Json<Body>,
) -> Result<StatusCode, Error> {
    let post_author = get_post_author(&pool, id)
        .await?
        .ok_or(Error::DoesNotExist)?;

    if session.user_id != post_author {
        return Err(Error::Unauthorized);
    }

    update_post(&pool, id, body).await?;
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
async fn update_post(pool: &PgPool, id: Uuid, body: Body) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE posts SET title = $1, content = $2, slug = $3 WHERE id = $4",
        body.title.as_ref(),
        body.content.as_ref(),
        body.slug.as_ref(),
        id
    )
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

    #[error("Unauthorized to update post")]
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
