use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use sqlx::PgPool;
use thiserror::Error;
use uuid::Uuid;

use crate::{
    auth::Session,
    models::post::{
        content::{Content, ContentError},
        slug::Slug,
        title::{Title, TitleError},
    },
};

#[derive(Deserialize)]
pub struct Body {
    title: String,
    content: String,
}

#[tracing::instrument(
    name = "Create Post",
    fields(title = body.title)
    skip_all,
)]
pub async fn handler(
    session: Session,
    State(pool): State<PgPool>,
    Json(body): Json<Body>,
) -> Result<String, Error> {
    let title = Title::try_from(body.title)?;
    let content = Content::try_from(body.content)?;
    let slug = generate_slug(&pool, &title).await?;

    insert_post(&pool, session.user_id, &title, &content, &slug).await?;

    Ok(slug.to_string())
}

#[tracing::instrument(name = "Generate Slug", skip_all)]
async fn generate_slug(pool: &PgPool, title: &Title) -> Result<Slug, sqlx::Error> {
    let mut slug = Slug::from(title);

    while sqlx::query_scalar!(
        r#"SELECT EXISTS(SELECT 1 FROM posts WHERE slug = $1) AS "exists!""#,
        slug.to_string()
    )
    .fetch_one(pool)
    .await?
    {
        slug.randomize();
    }

    Ok(slug)
}

#[tracing::instrument(name = "Insert Post", skip_all)]
async fn insert_post(
    pool: &PgPool,
    author_id: Uuid,
    title: &Title,
    content: &Content,
    slug: &Slug,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"INSERT INTO posts (id, author_id, title, content, slug) VALUES ($1, $2, $3, $4, $5)"#,
        Uuid::new_v4(),
        author_id,
        title.as_ref(),
        content.as_ref(),
        slug.to_string(),
    )
    .execute(pool)
    .await?;

    Ok(())
}

#[derive(Debug, Error)]
#[error(transparent)]
pub enum Error {
    Title(#[from] TitleError),
    Content(#[from] ContentError),
    Database(#[from] sqlx::Error),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Error::Title(_) | Error::Content(_) => StatusCode::BAD_REQUEST.into_response(),

            Error::Database(_) => {
                tracing::error!("{self}");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}
