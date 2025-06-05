use axum::{
    extract::{Path, State, Query},
    http::{StatusCode, HeaderMap},
    response::Json,
};
use serde::Deserialize;

use crate::{
    auth::{self, AuthResult},
    database::{DbPool, current_timestamp},
    models::*,
};

// Helper function to extract token from headers
fn extract_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .map(|s| s.to_string())
}

// Helper function for admin-only routes
async fn require_admin(headers: &HeaderMap, pool: &DbPool) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let token = extract_token(headers)
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, Json(ErrorResponse { error: "Missing token".to_string() })))?;
    
    match auth::validate_token(&token, pool).await {
        Some(AuthResult::Admin) => Ok(()),
        _ => Err((StatusCode::UNAUTHORIZED, Json(ErrorResponse { error: "Admin access required".to_string() }))),
    }
}

// POST /api/login
pub async fn login(
    State(pool): State<DbPool>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, (StatusCode, Json<ErrorResponse>)> {
    match auth::authenticate_user(&payload.username, &payload.password, &pool).await {
        Some(user) => Ok(Json(LoginResponse {
            token: user.token,
            role: user.role,
            username: user.username,
        })),
        None => Err((StatusCode::UNAUTHORIZED, Json(ErrorResponse {
            error: "Invalid credentials".to_string(),
        }))),
    }
}

// POST /api/machines
pub async fn create_machine(
    headers: HeaderMap,
    State(pool): State<DbPool>,
    Json(payload): Json<CreateMachineRequest>,
) -> Result<(StatusCode, Json<MachineResponse>), (StatusCode, Json<ErrorResponse>)> {
    require_admin(&headers, &pool).await?;
    
    let api_key = auth::generate_machine_api_key();
    
    match sqlx::query(
        "INSERT INTO machines (name, code, api_key, location, machine_type) VALUES (?, ?, ?, ?, ?)"
    )
    .bind(&payload.name)
    .bind(&payload.code)
    .bind(&api_key)
    .bind(&payload.location)
    .bind(&payload.machine_type)
    .execute(&pool)
    .await
    {
        Ok(result) => {
            let machine_id = result.last_insert_rowid();
            Ok((StatusCode::CREATED, Json(MachineResponse {
                id: machine_id,
                name: payload.name,
                code: payload.code,
                api_key,
                location: payload.location,
                machine_type: payload.machine_type,
            })))
        },
        Err(_) => Err((StatusCode::BAD_REQUEST, Json(ErrorResponse {
            error: "Machine code already exists".to_string(),
        }))),
    }
}

// GET /api/machines
pub async fn list_machines(
    headers: HeaderMap,
    State(pool): State<DbPool>,
) -> Result<Json<MachineListResponse>, (StatusCode, Json<ErrorResponse>)> {
    let token = extract_token(&headers)
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, Json(ErrorResponse { error: "Missing token".to_string() })))?;
    
    // Verify token is valid (admin or user)
    match auth::validate_token(&token, &pool).await {
        Some(AuthResult::Admin) | Some(AuthResult::User(_)) => {},
        _ => return Err((StatusCode::UNAUTHORIZED, Json(ErrorResponse { error: "Invalid token".to_string() }))),
    }
    
    match sqlx::query_as::<_, Machine>("SELECT * FROM machines ORDER BY name").fetch_all(&pool).await {
        Ok(machines) => Ok(Json(MachineListResponse { machines })),
        Err(_) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse {
            error: "Database error".to_string(),
        }))),
    }
}

// POST /api/machines/update
pub async fn update_machine_speed(
    headers: HeaderMap,
    State(pool): State<DbPool>,
    Json(payload): Json<SpeedUpdateRequest>,
) -> Result<Json<UpdateResponse>, (StatusCode, Json<ErrorResponse>)> {
    let token = extract_token(&headers)
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, Json(ErrorResponse { error: "Missing token".to_string() })))?;
    
    // Only machine API keys can update speed
    let machine_id = match auth::validate_token(&token, &pool).await {
        Some(AuthResult::Machine(id)) => id,
        _ => return Err((StatusCode::UNAUTHORIZED, Json(ErrorResponse { error: "Invalid machine API key".to_string() }))),
    };
    
    let timestamp = current_timestamp();
    let message = payload.message.unwrap_or_else(|| "".to_string());
    
    // Update machine status
    match sqlx::query(
        "UPDATE machines SET current_speed = ?, status_message = ?, last_update = ?, is_online = 1 WHERE id = ?"
    )
    .bind(payload.speed)
    .bind(&message)
    .bind(timestamp)
    .bind(machine_id)
    .execute(&pool)
    .await
    {
        Ok(_) => {
            // Insert into history
            let _ = sqlx::query(
                "INSERT INTO speed_history (machine_id, speed, message, timestamp) VALUES (?, ?, ?, ?)"
            )
            .bind(machine_id)
            .bind(payload.speed)
            .bind(&message)
            .bind(timestamp)
            .execute(&pool)
            .await;
            
            Ok(Json(UpdateResponse {
                success: true,
                timestamp,
            }))
        },
        Err(_) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse {
            error: "Failed to update machine".to_string(),
        }))),
    }
}

// POST /api/machines/{id}/comments
pub async fn add_comment(
    headers: HeaderMap,
    Path(machine_id): Path<i64>,
    State(pool): State<DbPool>,
    Json(payload): Json<AddCommentRequest>,
) -> Result<(StatusCode, Json<MaintenanceComment>), (StatusCode, Json<ErrorResponse>)> {
    let token = extract_token(&headers)
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, Json(ErrorResponse { error: "Missing token".to_string() })))?;
    
    let username = match auth::validate_token(&token, &pool).await {
        Some(AuthResult::Admin) => "admin".to_string(),
        Some(AuthResult::User(username)) => username,
        _ => return Err((StatusCode::UNAUTHORIZED, Json(ErrorResponse { error: "Invalid token".to_string() }))),
    };
    
    // Check if machine exists
    if let Err(_) = sqlx::query("SELECT id FROM machines WHERE id = ?")
        .bind(machine_id)
        .fetch_one(&pool)
        .await
    {
        return Err((StatusCode::NOT_FOUND, Json(ErrorResponse {
            error: "Machine not found".to_string(),
        })));
    }
    
    let priority = payload.priority.unwrap_or_else(|| "normal".to_string());
    let timestamp = current_timestamp();
    
    match sqlx::query(
        "INSERT INTO maintenance_comments (machine_id, username, comment, priority, created_at) VALUES (?, ?, ?, ?, ?)"
    )
    .bind(machine_id)
    .bind(&username)
    .bind(&payload.comment)
    .bind(&priority)
    .bind(timestamp)
    .execute(&pool)
    .await
    {
        Ok(result) => {
            let comment_id = result.last_insert_rowid();
            Ok((StatusCode::CREATED, Json(MaintenanceComment {
                id: comment_id,
                machine_id,
                comment: payload.comment,
                priority,
                username,
                created_at: timestamp,
            })))
        },
        Err(_) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse {
            error: "Failed to add comment".to_string(),
        }))),
    }
}

// GET /api/machines/{id}/comments
pub async fn get_comments(
    headers: HeaderMap,
    Path(machine_id): Path<i64>,
    State(pool): State<DbPool>,
) -> Result<Json<CommentListResponse>, (StatusCode, Json<ErrorResponse>)> {
    let token = extract_token(&headers)
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, Json(ErrorResponse { error: "Missing token".to_string() })))?;
    
    // Verify token is valid (admin or user)
    match auth::validate_token(&token, &pool).await {
        Some(AuthResult::Admin) | Some(AuthResult::User(_)) => {},
        _ => return Err((StatusCode::UNAUTHORIZED, Json(ErrorResponse { error: "Invalid token".to_string() }))),
    }
    
    // Check if machine exists
    if let Err(_) = sqlx::query("SELECT id FROM machines WHERE id = ?")
        .bind(machine_id)
        .fetch_one(&pool)
        .await
    {
        return Err((StatusCode::NOT_FOUND, Json(ErrorResponse {
            error: "Machine not found".to_string(),
        })));
    }
    
    match sqlx::query_as::<_, MaintenanceComment>(
        "SELECT * FROM maintenance_comments WHERE machine_id = ? ORDER BY created_at DESC"
    )
    .bind(machine_id)
    .fetch_all(&pool)
    .await
    {
        Ok(comments) => Ok(Json(CommentListResponse { comments })),
        Err(_) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse {
            error: "Database error".to_string(),
        }))),
    }
}

// GET /api/machines/{id}/history
#[derive(Deserialize)]
pub struct HistoryQuery {
    limit: Option<i64>,
}

pub async fn get_history(
    headers: HeaderMap,
    Path(machine_id): Path<i64>,
    Query(params): Query<HistoryQuery>,
    State(pool): State<DbPool>,
) -> Result<Json<HistoryResponse>, (StatusCode, Json<ErrorResponse>)> {
    let token = extract_token(&headers)
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, Json(ErrorResponse { error: "Missing token".to_string() })))?;
    
    // Verify token is valid (admin or user)
    match auth::validate_token(&token, &pool).await {
        Some(AuthResult::Admin) | Some(AuthResult::User(_)) => {},
        _ => return Err((StatusCode::UNAUTHORIZED, Json(ErrorResponse { error: "Invalid token".to_string() }))),
    }
    
    // Check if machine exists
    if let Err(_) = sqlx::query("SELECT id FROM machines WHERE id = ?")
        .bind(machine_id)
        .fetch_one(&pool)
        .await
    {
        return Err((StatusCode::NOT_FOUND, Json(ErrorResponse {
            error: "Machine not found".to_string(),
        })));
    }
    
    let limit = params.limit.unwrap_or(100);
    
    match sqlx::query_as::<_, SpeedHistory>(
        "SELECT speed, message, timestamp FROM speed_history WHERE machine_id = ? ORDER BY timestamp DESC LIMIT ?"
    )
    .bind(machine_id)
    .bind(limit)
    .fetch_all(&pool)
    .await
    {
        Ok(history) => Ok(Json(HistoryResponse { history })),
        Err(_) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse {
            error: "Database error".to_string(),
        }))),
    }
}

// POST /api/users
pub async fn create_user(
    headers: HeaderMap,
    State(pool): State<DbPool>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<User>), (StatusCode, Json<ErrorResponse>)> {
    require_admin(&headers, &pool).await?;
    
    let token = auth::generate_user_token();
    
    match sqlx::query(
        "INSERT INTO users (username, password, role, token) VALUES (?, ?, ?, ?)"
    )
    .bind(&payload.username)
    .bind(&payload.password)
    .bind(&payload.role)
    .bind(&token)
    .execute(&pool)
    .await
    {
        Ok(result) => {
            let user_id = result.last_insert_rowid();
            Ok((StatusCode::CREATED, Json(User {
                id: user_id,
                username: payload.username,
                role: payload.role,
                token,
            })))
        },
        Err(_) => Err((StatusCode::BAD_REQUEST, Json(ErrorResponse {
            error: "Username already exists".to_string(),
        }))),
    }
}