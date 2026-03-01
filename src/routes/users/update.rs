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
    domain::user::{display_name::DisplayName, username::Username},
    session::Session,
};

#[derive(Deserialize)]
pub struct Body {
    username: Username,
    display_name: DisplayName,
}

#[tracing::instrument(
    name = "Update Post",
    fields(
        username = body.username.as_ref(),
        display_name = body.display_name.as_ref()
    ),
    skip(pool, body)
)]
pub async fn handler(
    session: Session,
    State(pool): State<PgPool>,
    Path(username): Path<Username>,
    Json(body): Json<Body>,
) -> Result<StatusCode, Error> {
    let id = get_user_id(&pool, username)
        .await?
        .ok_or(Error::DoesNotExist)?;

    if session.user_id != id {
        return Err(Error::Unauthorized);
    }

    update_user(&pool, id, body).await?;
    Ok(StatusCode::OK)
}

#[tracing::instrument("Get user ID", skip_all)]
async fn get_user_id(pool: &PgPool, username: Username) -> Result<Option<Uuid>, sqlx::Error> {
    let row = sqlx::query_scalar!(
        "SELECT id FROM users WHERE username = $1",
        username.as_ref()
    )
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

#[tracing::instrument("Update user in database", skip_all)]
async fn update_user(pool: &PgPool, id: Uuid, body: Body) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE users SET username = $1, display_name = $2 WHERE id = $3",
        body.username.as_ref(),
        body.display_name.as_ref(),
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

    #[error("User does not exist")]
    DoesNotExist,

    #[error("Unauthorized to update user")]
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
