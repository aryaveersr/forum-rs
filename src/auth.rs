use axum::{
    extract::{FromRef, FromRequestParts, OptionalFromRequestParts},
    http::{StatusCode, header::ToStrError, request::Parts},
    response::IntoResponse,
};
use chrono::Utc;
use sqlx::PgPool;
use thiserror::Error;
use uuid::Uuid;

pub struct Session {
    pub user_id: Uuid,
    pub session_id: Uuid,
}

impl<S> FromRequestParts<S> for Session
where
    S: Send + Sync,
    PgPool: FromRef<S>,
{
    type Rejection = SessionError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, SessionError> {
        match <Session as OptionalFromRequestParts<S>>::from_request_parts(parts, state).await {
            Ok(Some(session)) => Ok(session),
            Ok(None) => Err(SessionError::NotFound),
            Err(err) => Err(err),
        }
    }
}

impl<S> OptionalFromRequestParts<S> for Session
where
    S: Send + Sync,
    PgPool: FromRef<S>,
{
    type Rejection = SessionError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> Result<Option<Self>, Self::Rejection> {
        let pool = PgPool::from_ref(state);

        let id_str = match parts.headers.get("Session-ID") {
            Some(id) => id.to_str()?,
            None => return Ok(None),
        };

        let session_id = Uuid::parse_str(id_str)?;

        let row = sqlx::query!(
            "SELECT id, user_id FROM sessions WHERE id = $1 AND expires_at > $2",
            session_id,
            Utc::now()
        )
        .fetch_optional(&pool)
        .await?;

        match row {
            Some(row) => Ok(Some(Session {
                user_id: row.user_id,
                session_id: row.id,
            })),

            None => Ok(None),
        }
    }
}

#[derive(Debug, Error)]
#[error(transparent)]
pub enum SessionError {
    Database(#[from] sqlx::Error),
    Uuid(#[from] uuid::Error),
    ToStr(#[from] ToStrError),

    #[error("Session not found")]
    NotFound,
}

impl IntoResponse for SessionError {
    fn into_response(self) -> axum::response::Response {
        match self {
            SessionError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            SessionError::Uuid(_) | SessionError::ToStr(_) => StatusCode::BAD_REQUEST,
            SessionError::NotFound => StatusCode::UNAUTHORIZED,
        }
        .into_response()
    }
}
