use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Machine {
    pub id: i64,
    pub name: String,
    pub code: String,
    pub location: Option<String>,
    pub machine_type: Option<String>,
    pub current_speed: f64,
    pub status_message: String,
    pub is_online: bool,
    pub last_update: i64,
}

#[derive(Debug, Serialize)]
pub struct MachineResponse {
    pub id: i64,
    pub name: String,
    pub code: String,
    pub api_key: String, // Only for create response
    pub location: Option<String>,
    pub machine_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateMachineRequest {
    pub name: String,
    pub code: String,
    pub location: Option<String>,
    pub machine_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SpeedUpdateRequest {
    pub speed: f64,
    pub message: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub role: String,
    pub username: String,
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    #[serde(flatten)]
    pub data: T,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub role: String,
    pub token: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub password: String,
    pub role: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct MaintenanceComment {
    pub id: i64,
    pub machine_id: i64,
    pub comment: String,
    pub priority: String,
    pub username: String,
    pub created_at: i64,
}

#[derive(Debug, Deserialize)]
pub struct AddCommentRequest {
    pub comment: String,
    pub priority: Option<String>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct SpeedHistory {
    pub speed: f64,
    pub message: Option<String>,
    pub timestamp: i64,
}

#[derive(Debug, Serialize)]
pub struct MachineListResponse {
    pub machines: Vec<Machine>,
}

#[derive(Debug, Serialize)]
pub struct CommentListResponse {
    pub comments: Vec<MaintenanceComment>,
}

#[derive(Debug, Serialize)]
pub struct HistoryResponse {
    pub history: Vec<SpeedHistory>,
}

#[derive(Debug, Serialize)]
pub struct UpdateResponse {
    pub success: bool,
    pub timestamp: i64,
}