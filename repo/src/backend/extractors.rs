/// Authenticated user stored in request extensions by the session middleware.
#[derive(Clone, Debug)]
pub struct SessionUser {
    pub user_id: String,
    pub username: String,
    pub role: String,
    pub session_id: String,
}
