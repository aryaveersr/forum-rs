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
    domain::post::{content::Content, slug::Slug, title::Title},
    session::Session,
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
    Path(slug): Path<Slug>,
    Json(body): Json<Body>,
) -> Result<StatusCode, Error> {
    let author = get_post_author(&pool, &slug)
        .await?
        .ok_or(Error::DoesNotExist)?;

    if session.user_id != author {
        return Err(Error::Unauthorized);
    }

    update_post(&pool, slug, body).await?;
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
async fn update_post(pool: &PgPool, slug: Slug, body: Body) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE posts SET title = $1, content = $2, slug = $3 WHERE slug = $4",
        body.title.as_ref(),
        body.content.as_ref(),
        body.slug.as_ref(),
        slug.as_ref()
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
