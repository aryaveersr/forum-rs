use argon2::{Argon2, PasswordHash, PasswordVerifier, password_hash};
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

#[derive(Deserialize)]
pub struct Body {
    username: String,
    password: String,
}

#[tracing::instrument(
    name = "Login User",
    fields(username = body.username)
    skip_all,
)]
pub async fn handler(
    State(pool): State<PgPool>,
    Json(body): Json<Body>,
) -> Result<StatusCode, Error> {
    let user = match get_user(&pool, &body.username).await? {
        Some(user) => user,
        None => return Err(Error::DoesNotExist),
    };

    if !check_password(body.password, user.password_hash).await? {
        return Err(Error::InvalidCredentials);
    }

    tracing::info!("Login successful for {}", user.id);

    Ok(StatusCode::OK)
}

struct User {
    id: Uuid,
    password_hash: String,
}

#[tracing::instrument(name = "Get User", skip_all)]
async fn get_user(pool: &PgPool, username: &str) -> Result<Option<User>, Error> {
    let user = sqlx::query_as!(
        User,
        r#"SELECT id, password_hash FROM users WHERE username = $1"#,
        username
    )
    .fetch_optional(pool)
    .await?;

    Ok(user)
}

#[tracing::instrument(name = "Check Password", skip_all)]
async fn check_password(password: String, hash: String) -> Result<bool, password_hash::Error> {
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
