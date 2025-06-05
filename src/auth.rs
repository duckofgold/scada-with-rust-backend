use crate::database::DbPool;
use uuid::Uuid;
use sqlx::Row;

#[derive(Debug, Clone)]
pub enum AuthResult {
    Admin,
    User(String), // username
    Machine(i64), // machine_id
}

pub async fn validate_token(token: &str, pool: &DbPool) -> Option<AuthResult> {
    // Check hardcoded admin token
    if token == "admin_token_12345" {
        return Some(AuthResult::Admin);
    }
    
    // Check if it's a machine API key
    if token.starts_with("machine_") {
        if let Ok(row) = sqlx::query("SELECT id FROM machines WHERE api_key = ?")
            .bind(token)
            .fetch_one(pool)
            .await
        {
            let machine_id: i64 = row.get("id");
            return Some(AuthResult::Machine(machine_id));
        }
    }
    
    // Check user tokens
    if let Ok(row) = sqlx::query("SELECT username FROM users WHERE token = ?")
        .bind(token)
        .fetch_one(pool)
        .await
    {
        let username: String = row.get("username");
        return Some(AuthResult::User(username));
    }
    
    None
}

pub fn generate_machine_api_key() -> String {
    format!("machine_{}", Uuid::new_v4().simple())
}

pub fn generate_user_token() -> String {
    format!("user_{}", Uuid::new_v4().simple())
}

pub async fn authenticate_user(username: &str, password: &str, pool: &DbPool) -> Option<crate::models::User> {
    sqlx::query_as::<_, crate::models::User>("SELECT * FROM users WHERE username = ? AND password = ?")
        .bind(username)
        .bind(password)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
}