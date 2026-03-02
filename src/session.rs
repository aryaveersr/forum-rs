use axum::{
    extract::{FromRef, FromRequestParts, OptionalFromRequestParts},
    http::{StatusCode, header::ToStrError, request::Parts},
    response::IntoResponse,
};
use axum_extra::extract::{
    CookieJar,
    cookie::{Cookie, SameSite},
};
use chrono::{Duration, Utc};
use sqlx::PgPool;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug)]
pub struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
}

impl Session {
    #[tracing::instrument(skip_all)]
    pub async fn new(pool: &PgPool, user_id: Uuid) -> Result<Self, sqlx::Error> {
        let id = Uuid::new_v4();

        sqlx::query!(
            "INSERT INTO sessions (id, user_id, expires_at) VALUES ($1, $2, $3)",
            id,
            user_id,
            Utc::now() + Duration::days(7)
        )
        .execute(pool)
        .await?;

        Self::delete_expired_sessions(pool).await?;
        Ok(Self { id, user_id })
    }

    #[tracing::instrument(skip_all)]
    async fn from_id(pool: &PgPool, session_id: Uuid) -> Result<Option<Session>, sqlx::Error> {
        let row = sqlx::query_as!(
            Session,
            "SELECT id, user_id FROM sessions WHERE id = $1 AND expires_at > $2",
            session_id,
            Utc::now()
        )
        .fetch_optional(pool)
        .await?;

        Ok(row)
    }

    #[tracing::instrument(skip_all)]
    pub async fn delete_session(&self, pool: &PgPool) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM sessions WHERE id = $1", self.id)
            .execute(pool)
            .await?;

        Ok(())
    }

    #[tracing::instrument(skip_all)]
    pub async fn delete_all_sessions(&self, pool: &PgPool) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM sessions WHERE user_id = $1", self.user_id)
            .execute(pool)
            .await?;

        Ok(())
    }

    #[tracing::instrument(skip_all)]
    async fn delete_expired_sessions(pool: &PgPool) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM sessions WHERE expires_at < $1", Utc::now())
            .execute(pool)
            .await?;

        Ok(())
    }

    pub fn cookie(&self) -> Cookie<'static> {
        let builder = Cookie::build(("session", self.id.to_string()))
            .http_only(true)
            .same_site(SameSite::Strict)
            .expires(time::OffsetDateTime::now_utc() + time::Duration::days(7));

        if cfg!(debug_assertions) {
            builder.into()
        } else {
            builder.secure(true).into()
        }
    }
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
        let jar = CookieJar::from_headers(&parts.headers);

        let id = match jar.get("session") {
            Some(id) => Uuid::parse_str(id.value())?,
            None => return Ok(None),
        };

        Ok(Session::from_id(&pool, id).await?)
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
