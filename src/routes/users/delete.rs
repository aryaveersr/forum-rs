use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use sqlx::PgPool;
use thiserror::Error;
use uuid::Uuid;

use crate::{domain::user::username::Username, session::Session};

#[tracing::instrument(name = "Delete User", skip(pool))]
pub async fn handler(
    session: Session,
    State(pool): State<PgPool>,
    Path(username): Path<Username>,
) -> Result<StatusCode, Error> {
    let id = get_user_id(&pool, username)
        .await?
        .ok_or(Error::DoesNotExist)?;

    if session.user_id != id {
        return Err(Error::Unauthorized);
    }

    delete_user(&pool, id).await?;
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

#[tracing::instrument("Delete user", skip_all)]
async fn delete_user(pool: &PgPool, id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query!("DELETE FROM users WHERE id = $1", id)
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

    #[error("Unauthorized to delete user")]
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
