mod extract;

use chrono::{Duration, Utc};
pub use extract::Session;
use sqlx::PgPool;
use uuid::Uuid;

#[tracing::instrument(skip_all)]
pub async fn create_session(pool: &PgPool, user_id: Uuid) -> Result<Uuid, sqlx::Error> {
    let session_id = Uuid::new_v4();

    sqlx::query!(
        "INSERT INTO sessions (id, user_id, expires_at) VALUES ($1, $2, $3)",
        session_id,
        user_id,
        Utc::now() + Duration::days(7)
    )
    .execute(pool)
    .await?;

    delete_expired_sessions(pool).await?;
    Ok(session_id)
}

#[tracing::instrument(skip_all)]
async fn delete_expired_sessions(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query!("DELETE FROM sessions WHERE expires_at < $1", Utc::now())
        .execute(pool)
        .await?;

    Ok(())
}
