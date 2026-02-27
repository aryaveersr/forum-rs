use crate::auth::Session;

pub async fn handler(session: Session) -> String {
    format!("{} -> {}", session.user_id, session.session_id)
}
