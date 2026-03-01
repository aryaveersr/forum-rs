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

use crate::domain::user::username::Username;

#[tracing::instrument(name = "Get User", skip(pool))]
pub async fn handler(
    State(pool): State<PgPool>,
    Path(username): Path<Username>,
) -> Result<Json<User>, Error> {
    let user = get_user(&pool, username)
        .await?
        .ok_or(Error::DoesNotExist)?;

    Ok(Json(user))
}

#[derive(Serialize, FromRow)]
pub struct User {
    id: Uuid,
    display_name: String,
}

#[tracing::instrument(name = "Get user from database", skip_all)]
async fn get_user(pool: &PgPool, username: Username) -> Result<Option<User>, sqlx::Error> {
    let user = sqlx::query_as!(
        User,
        r#"SELECT id, display_name FROM users WHERE username = $1"#,
        username.as_ref()
    )
    .fetch_optional(pool)
    .await?;

    Ok(user)
}

#[derive(Debug, Error)]
#[error(transparent)]
pub enum Error {
    Database(#[from] sqlx::Error),

    #[error("User does not exist")]
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
