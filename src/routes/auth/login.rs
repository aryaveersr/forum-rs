use argon2::{Argon2, PasswordHash, PasswordVerifier, password_hash};
use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use axum_extra::extract::CookieJar;
use serde::Deserialize;
use sqlx::PgPool;
use thiserror::Error;
use uuid::Uuid;

use crate::{
    domain::user::{password::Password, username::Username},
    session::Session,
};

#[derive(Deserialize, Debug)]
pub struct Body {
    username: Username,
    password: Password,
}

#[tracing::instrument(name = "Login User", skip(pool))]
pub async fn handler(
    State(pool): State<PgPool>,
    jar: CookieJar,
    Json(body): Json<Body>,
) -> Result<(CookieJar, StatusCode), Error> {
    let user = get_user(&pool, &body.username)
        .await?
        .ok_or(Error::DoesNotExist)?;

    if !check_password(body.password, user.password_hash).await? {
        return Err(Error::InvalidCredentials);
    }

    let session = Session::new(&pool, user.id).await?;

    Ok((jar.add(session.cookie()), StatusCode::OK))
}

struct User {
    id: Uuid,
    password_hash: String,
}

#[tracing::instrument(skip_all)]
async fn get_user(pool: &PgPool, username: &Username) -> Result<Option<User>, Error> {
    let user = sqlx::query_as!(
        User,
        r#"SELECT id, password_hash FROM users WHERE username = $1"#,
        username.as_ref()
    )
    .fetch_optional(pool)
    .await?;

    Ok(user)
}

#[tracing::instrument(skip_all)]
async fn check_password(password: Password, hash: String) -> Result<bool, password_hash::Error> {
    let span = tracing::Span::current();

    tokio::task::spawn_blocking(move || {
        span.in_scope(move || {
            let hash =
                PasswordHash::new(&hash).expect("Failed to parse PasswordHash from database.");

            match Argon2::default().verify_password(password.as_bytes(), &hash) {
                Ok(()) => Ok(true),
                Err(password_hash::Error::Password) => Ok(false),
                Err(err) => Err(err),
            }
        })
    })
    .await
    .unwrap()
}

#[derive(Debug, Error)]
#[error(transparent)]
pub enum Error {
    Database(#[from] sqlx::Error),
    PasswordHash(#[from] password_hash::Error),

    #[error("Username does not exist")]
    DoesNotExist,

    #[error("Invalid credentials")]
    InvalidCredentials,
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Error::DoesNotExist => StatusCode::BAD_REQUEST.into_response(),
            Error::InvalidCredentials => StatusCode::UNAUTHORIZED.into_response(),

            Error::Database(_) | Error::PasswordHash(_) => {
                tracing::error!("{self}");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}
