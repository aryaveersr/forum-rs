use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use sqlx::PgPool;
use thiserror::Error;

use crate::session::Session;

#[derive(Deserialize)]
pub struct LogoutQuery {
    all: bool,
}

#[tracing::instrument(name = "Logout User", skip(pool))]
pub async fn handler(
    session: Session,
    State(pool): State<PgPool>,
    Query(LogoutQuery { all }): Query<LogoutQuery>,
) -> Result<StatusCode, Error> {
    if all {
        session.delete_all_sessions(&pool).await?;
    } else {
        session.delete_session(&pool).await?;
    }

    Ok(StatusCode::OK)
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
