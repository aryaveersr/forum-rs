use argon2::{
    Argon2, PasswordHash, PasswordHasher,
    password_hash::{self, SaltString},
};
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

use crate::domain::user::{
    display_name::{DisplayName, DisplayNameError},
    password::{Password, PasswordError},
    username::{Username, UsernameError},
};

#[derive(Deserialize)]
pub struct Body {
    username: String,
    display_name: String,
    password: String,
}

#[tracing::instrument(
    name = "Register User",
    skip_all,
    fields(
        username = body.username,
        display_name = body.display_name,
    )
)]
pub async fn handler(
    State(pool): State<PgPool>,
    Json(body): Json<Body>,
) -> Result<StatusCode, Error> {
    let username = Username::try_from(body.username)?;
    let display_name = DisplayName::try_from(body.display_name)?;
    let password = Password::try_from(body.password)?;

    if check_if_username_exists(&pool, &username).await? {
        return Err(Error::AlreadyExists);
    }

    let id = Uuid::new_v4();
    let salt = SaltString::encode_b64(id.as_bytes())?;
    let password_hash = Argon2::default().hash_password(password.as_bytes(), &salt)?;

    insert_user(&pool, id, username, display_name, password_hash).await?;

    Ok(StatusCode::CREATED)
}

#[tracing::instrument(name = "Check if username exists", skip_all)]
async fn check_if_username_exists(pool: &PgPool, username: &Username) -> Result<bool, sqlx::Error> {
    let exists = sqlx::query_scalar!(
        r#"SELECT EXISTS(SELECT 1 FROM users WHERE username = $1) AS "exists!""#,
        username.as_ref()
    )
    .fetch_one(pool)
    .await?;

    Ok(exists)
}

#[tracing::instrument(name = "Insert User", skip_all)]
async fn insert_user(
    pool: &PgPool,
    id: Uuid,
    username: Username,
    display_name: DisplayName,
    password_hash: PasswordHash<'_>,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"INSERT INTO users (id, username, display_name, password_hash) VALUES ($1, $2, $3, $4)"#,
        id,
        username.as_ref(),
        display_name.as_ref(),
        password_hash.to_string()
    )
    .execute(pool)
    .await?;

    Ok(())
}

#[derive(Debug, Error)]
#[error(transparent)]
pub enum Error {
    Username(#[from] UsernameError),
    DisplayName(#[from] DisplayNameError),
    Password(#[from] PasswordError),
    Database(#[from] sqlx::Error),
    PasswordHash(#[from] password_hash::Error),

    #[error("Username already exists")]
    AlreadyExists,
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Error::AlreadyExists => StatusCode::CONFLICT.into_response(),

            Error::Username(_) | Error::DisplayName(_) | Error::Password(_) => {
                let err = self.to_string();
                (StatusCode::BAD_REQUEST, err).into_response()
            }

            Error::Database(_) | Error::PasswordHash(_) => {
                tracing::error!("{self}");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}
