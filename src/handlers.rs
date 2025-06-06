use axum::{
    extract::{Path, State, Query},
    http::{StatusCode, HeaderMap},
    response::Json,
};
use serde::Deserialize;
use sqlx::Row;

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
    println!("[LOG] Login request received for user: {}", payload.username);
    match auth::authenticate_user(&payload.username, &payload.password, &pool).await {
        Some(user) => {
            println!("[LOG] Login successful for user: {}", user.username);
            Ok(Json(LoginResponse {
                token: user.token,
                role: user.role,
                username: user.username,
            }))
        },
        None => {
            println!("[LOG] Login failed for user: {}", payload.username);
            Err((StatusCode::UNAUTHORIZED, Json(ErrorResponse {
                error: "Invalid credentials".to_string(),
            })))
        },
    }
}

// POST /api/machines
pub async fn create_machine(
    headers: HeaderMap,
    State(pool): State<DbPool>,
    Json(payload): Json<CreateMachineRequest>,
) -> Result<(StatusCode, Json<MachineResponse>), (StatusCode, Json<ErrorResponse>)> {
    println!("[LOG] Create machine request received: {}", payload.name);
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
            println!("[LOG] Machine created successfully: {}", payload.name);
            Ok((StatusCode::CREATED, Json(MachineResponse {
                id: machine_id,
                name: payload.name,
                code: payload.code,
                api_key,
                location: payload.location,
                machine_type: payload.machine_type,
            })))
        },
        Err(_) => {
            println!("[LOG] Failed to create machine: {}", payload.name);
            Err((StatusCode::BAD_REQUEST, Json(ErrorResponse {
                error: "Machine code already exists".to_string(),
            })))
        },
    }
}

// GET /api/machines
pub async fn list_machines(
    headers: HeaderMap,
    State(pool): State<DbPool>,
) -> Result<Json<MachineListResponse>, (StatusCode, Json<ErrorResponse>)> {
    println!("[LOG] List machines request received");
    let token = extract_token(&headers)
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, Json(ErrorResponse { error: "Missing token".to_string() })))?;
    
    // Verify token is valid (admin or user)
    match auth::validate_token(&token, &pool).await {
        Some(AuthResult::Admin) | Some(AuthResult::User(_)) => {},
        _ => return Err((StatusCode::UNAUTHORIZED, Json(ErrorResponse { error: "Invalid token".to_string() }))),
    }
    
    match sqlx::query_as::<_, Machine>("SELECT * FROM machines ORDER BY name").fetch_all(&pool).await {
        Ok(machines) => {
            println!("[LOG] Machines listed successfully");
            Ok(Json(MachineListResponse { machines }))
        },
        Err(_) => {
            println!("[LOG] Failed to list machines");
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse {
                error: "Database error".to_string(),
            })))
        },
    }
}

// POST /api/machines/update
pub async fn update_machine_speed(
    headers: HeaderMap,
    State(pool): State<DbPool>,
    Json(payload): Json<SpeedUpdateRequest>,
) -> Result<Json<UpdateResponse>, (StatusCode, Json<ErrorResponse>)> {
    println!("[LOG] Update machine speed request received");
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
            
            println!("[LOG] Machine speed updated successfully for machine ID: {}", machine_id);
            Ok(Json(UpdateResponse {
                success: true,
                timestamp,
            }))
        },
        Err(_) => {
            println!("[LOG] Failed to update machine speed for machine ID: {}", machine_id);
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse {
                error: "Failed to update machine".to_string(),
            })))
        },
    }
}

// POST /api/machines/{id}/comments
pub async fn add_comment(
    headers: HeaderMap,
    Path(machine_id): Path<i64>,
    State(pool): State<DbPool>,
    Json(payload): Json<AddCommentRequest>,
) -> Result<(StatusCode, Json<MaintenanceComment>), (StatusCode, Json<ErrorResponse>)> {
    println!("[LOG] Add comment request received for machine ID: {}", machine_id);
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
            println!("[LOG] Comment added successfully for machine ID: {}", machine_id);
            Ok((StatusCode::CREATED, Json(MaintenanceComment {
                id: comment_id,
                machine_id,
                comment: payload.comment,
                priority,
                username,
                created_at: timestamp,
            })))
        },
        Err(_) => {
            println!("[LOG] Failed to add comment for machine ID: {}", machine_id);
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse {
                error: "Failed to add comment".to_string(),
            })))
        },
    }
}

// GET /api/machines/{id}/comments
pub async fn get_comments(
    headers: HeaderMap,
    Path(machine_id): Path<i64>,
    State(pool): State<DbPool>,
) -> Result<Json<CommentListResponse>, (StatusCode, Json<ErrorResponse>)> {
    println!("[LOG] Get comments request received for machine ID: {}", machine_id);
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
        Ok(comments) => {
            println!("[LOG] Comments retrieved successfully for machine ID: {}", machine_id);
            Ok(Json(CommentListResponse { comments }))
        },
        Err(_) => {
            println!("[LOG] Failed to retrieve comments for machine ID: {}", machine_id);
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse {
                error: "Database error".to_string(),
            })))
        },
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
    println!("[LOG] Create user request received for user: {}", payload.username);
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
            println!("[LOG] User created successfully: {}", payload.username);
            Ok((StatusCode::CREATED, Json(User {
                id: user_id,
                username: payload.username,
                role: payload.role,
                token,
            })))
        },
        Err(_) => {
            println!("[LOG] Failed to create user: {}", payload.username);
            Err((StatusCode::BAD_REQUEST, Json(ErrorResponse {
                error: "Username already exists".to_string(),
            })))
        },
    }
}

// PUT /api/users/{id}
pub async fn update_user(
    headers: HeaderMap,
    Path(user_id): Path<i64>,
    State(pool): State<DbPool>,
    Json(payload): Json<UpdateUserRequest>,
) -> Result<Json<User>, (StatusCode, Json<ErrorResponse>)> {
    println!("[LOG] Update user request received for user ID: {}", user_id);
    require_admin(&headers, &pool).await?;

    // Check if user exists
    if let Err(_) = sqlx::query("SELECT id FROM users WHERE id = ?")
        .bind(user_id)
        .fetch_one(&pool)
        .await
    {
        return Err((StatusCode::NOT_FOUND, Json(ErrorResponse {
            error: "User not found".to_string(),
        })));
    }

    // Build update query dynamically based on provided fields
    let mut query = String::from("UPDATE users SET ");
    let mut params: Vec<String> = Vec::new();
    let mut query_builder = sqlx::query("");

    if let Some(password) = &payload.password {
        params.push("password = ?".to_string());
        query_builder = query_builder.bind(password);
    }

    if let Some(role) = &payload.role {
        if !["admin", "manager", "technician"].contains(&role.as_str()) {
            return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse {
                error: "Invalid role. Must be one of: admin, manager, technician".to_string(),
            })));
        }
        params.push("role = ?".to_string());
        query_builder = query_builder.bind(role);
    }

    if let Some(is_active) = &payload.is_active {
        params.push("is_active = ?".to_string());
        query_builder = query_builder.bind(is_active);
    }

    if params.is_empty() {
        return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse {
            error: "No fields to update".to_string(),
        })));
    }

    query.push_str(&params.join(", "));
    query.push_str(" WHERE id = ?");
    query_builder = query_builder.bind(user_id);

    // Execute update
    match query_builder.execute(&pool).await {
        Ok(_) => {
            // Fetch updated user
            match sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
                .bind(user_id)
                .fetch_one(&pool)
                .await
            {
                Ok(user) => {
                    println!("[LOG] User updated successfully: {}", user.username);
                    Ok(Json(user))
                },
                Err(_) => {
                    println!("[LOG] Failed to fetch updated user: {}", user_id);
                    Err((StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse {
                        error: "Failed to fetch updated user".to_string(),
                    })))
                },
            }
        },
        Err(_) => {
            println!("[LOG] Failed to update user: {}", user_id);
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse {
                error: "Failed to update user".to_string(),
            })))
        },
    }
}

// PUT /api/machines/{id}
pub async fn update_machine(
    headers: HeaderMap,
    Path(machine_id): Path<i64>,
    State(pool): State<DbPool>,
    Json(payload): Json<UpdateMachineRequest>,
) -> Result<Json<MachineResponse>, (StatusCode, Json<ErrorResponse>)> {
    println!("[LOG] Update machine request received for machine ID: {}", machine_id);
    require_admin(&headers, &pool).await?;

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

    // Build update query dynamically based on provided fields
    let mut query = String::from("UPDATE machines SET ");
    let mut params: Vec<String> = Vec::new();
    let mut query_builder = sqlx::query("");

    if let Some(name) = &payload.name {
        params.push("name = ?".to_string());
        query_builder = query_builder.bind(name);
    }

    if let Some(code) = &payload.code {
        params.push("code = ?".to_string());
        query_builder = query_builder.bind(code);
    }

    if let Some(location) = &payload.location {
        params.push("location = ?".to_string());
        query_builder = query_builder.bind(location);
    }

    if let Some(machine_type) = &payload.machine_type {
        params.push("machine_type = ?".to_string());
        query_builder = query_builder.bind(machine_type);
    }

    if let Some(true) = payload.regenerate_api_key {
        params.push("api_key = ?".to_string());
        query_builder = query_builder.bind(auth::generate_machine_api_key());
    }

    if params.is_empty() {
        return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse {
            error: "No fields to update".to_string(),
        })));
    }

    query.push_str(&params.join(", "));
    query.push_str(" WHERE id = ?");
    query_builder = query_builder.bind(machine_id);

    // Execute update
    match query_builder.execute(&pool).await {
        Ok(_) => {
            // Fetch updated machine and its API key
            match sqlx::query("SELECT m.*, m.api_key FROM machines m WHERE m.id = ?")
                .bind(machine_id)
                .fetch_one(&pool)
                .await
            {
                Ok(row) => {
                    let machine = Machine {
                        id: row.get("id"),
                        name: row.get("name"),
                        code: row.get("code"),
                        location: row.get("location"),
                        machine_type: row.get("machine_type"),
                        current_speed: row.get("current_speed"),
                        status_message: row.get("status_message"),
                        is_online: row.get("is_online"),
                        last_update: row.get("last_update"),
                    };
                    let api_key: String = row.get("api_key");
                    
                    println!("[LOG] Machine updated successfully: {}", machine.name);
                    Ok(Json(MachineResponse {
                        id: machine.id,
                        name: machine.name,
                        code: machine.code,
                        api_key,
                        location: machine.location,
                        machine_type: machine.machine_type,
                    }))
                },
                Err(_) => {
                    println!("[LOG] Failed to fetch updated machine: {}", machine_id);
                    Err((StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse {
                        error: "Failed to fetch updated machine".to_string(),
                    })))
                },
            }
        },
        Err(e) => {
            println!("[LOG] Failed to update machine: {}", machine_id);
            if e.to_string().contains("UNIQUE constraint failed") {
                Err((StatusCode::BAD_REQUEST, Json(ErrorResponse {
                    error: "Machine name or code already exists".to_string(),
                })))
            } else {
                Err((StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse {
                    error: "Failed to update machine".to_string(),
                })))
            }
        },
    }
}

// GET /api/users
pub async fn list_users(
    headers: HeaderMap,
    State(pool): State<DbPool>,
) -> Result<Json<UserListResponse>, (StatusCode, Json<ErrorResponse>)> {
    println!("[LOG] List users request received");
    require_admin(&headers, &pool).await?;

    match sqlx::query_as::<_, User>("SELECT * FROM users ORDER BY username").fetch_all(&pool).await {
        Ok(users) => {
            println!("[LOG] Users listed successfully");
            Ok(Json(UserListResponse { users }))
        },
        Err(_) => {
            println!("[LOG] Failed to list users");
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse {
                error: "Database error".to_string(),
            })))
        },
    }
}