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

use crate::{
    domain::user::{display_name::DisplayName, password::Password, username::Username},
    session::Session,
};

#[derive(Deserialize, Debug)]
pub struct Body {
    username: Username,
    display_name: DisplayName,
    password: Password,
}

#[tracing::instrument(name = "Register User", skip(pool))]
pub async fn handler(
    State(pool): State<PgPool>,
    Json(body): Json<Body>,
) -> Result<(StatusCode, String), Error> {
    if check_if_username_exists(&pool, &body.username).await? {
        return Err(Error::AlreadyExists);
    }

    let id = Uuid::new_v4();
    let salt = SaltString::encode_b64(id.as_bytes())?;
    let password_hash = Argon2::default().hash_password(body.password.as_bytes(), &salt)?;

    insert_user(&pool, id, body.username, body.display_name, password_hash).await?;

    let session = Session::new(&pool, id).await?;

    Ok((StatusCode::CREATED, session.id.to_string()))
}

#[tracing::instrument(skip_all)]
async fn check_if_username_exists(pool: &PgPool, username: &Username) -> Result<bool, sqlx::Error> {
    let exists = sqlx::query_scalar!(
        r#"SELECT EXISTS(SELECT 1 FROM users WHERE username = $1) AS "exists!""#,
        username.as_ref()
    )
    .fetch_one(pool)
    .await?;

    Ok(exists)
}

#[tracing::instrument(skip_all)]
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
    Database(#[from] sqlx::Error),
    PasswordHash(#[from] password_hash::Error),

    #[error("Username already exists")]
    AlreadyExists,
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Error::AlreadyExists => StatusCode::CONFLICT.into_response(),

            Error::Database(_) | Error::PasswordHash(_) => {
                tracing::error!("{self}");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}
