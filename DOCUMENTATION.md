# SCADA Backend Documentation

## API Structure

### Endpoints

- **POST /api/login**
  - **Description:** Authenticate a user and receive a token.
  - **Request Body:**
    ```json
    {
      "username": "admin",
      "password": "admin123"
    }
    ```
  - **Response:**
    ```json
    {
      "token": "admin_token_12345",
      "role": "admin",
      "username": "admin"
    }
    ```

- **POST /api/machines**
  - **Description:** Create a new machine.
  - **Request Body:**
    ```json
    {
      "name": "Test Machine",
      "code": "MCH-001",
      "location": "Plant 1",
      "machine_type": "Conveyor"
    }
    ```
  - **Response:**
    ```json
    {
      "id": 1,
      "name": "Test Machine",
      "code": "MCH-001",
      "api_key": "machine_12345",
      "location": "Plant 1",
      "machine_type": "Conveyor"
    }
    ```

- **GET /api/machines**
  - **Description:** List all machines.
  - **Response:**
    ```json
    {
      "machines": [
        {
          "id": 1,
          "name": "Test Machine",
          "code": "MCH-001",
          "location": "Plant 1",
          "machine_type": "Conveyor",
          "current_speed": 0.0,
          "status_message": "",
          "is_online": false,
          "last_update": 0
        }
      ]
    }
    ```

- **POST /api/machines/update**
  - **Description:** Update a machine's speed.
  - **Request Body:**
    ```json
    {
      "speed": 123.45,
      "message": "Running at test speed"
    }
    ```
  - **Response:**
    ```json
    {
      "success": true,
      "timestamp": 1234567890
    }
    ```

- **POST /api/machines/{id}/comments**
  - **Description:** Add a comment to a machine.
  - **Request Body:**
    ```json
    {
      "comment": "Routine check completed.",
      "priority": "normal"
    }
    ```
  - **Response:**
    ```json
    {
      "id": 1,
      "machine_id": 1,
      "comment": "Routine check completed.",
      "priority": "normal",
      "username": "admin",
      "created_at": 1234567890
    }
    ```

- **GET /api/machines/{id}/comments**
  - **Description:** Get comments for a machine.
  - **Response:**
    ```json
    {
      "comments": [
        {
          "id": 1,
          "machine_id": 1,
          "comment": "Routine check completed.",
          "priority": "normal",
          "username": "admin",
          "created_at": 1234567890
        }
      ]
    }
    ```

- **POST /api/users**
  - **Description:** Create a new user.
  - **Request Body:**
    ```json
    {
      "username": "user1",
      "password": "userpass",
      "role": "manager"
    }
    ```
  - **Response:**
    ```json
    {
      "id": 1,
      "username": "user1",
      "role": "manager",
      "token": "user_token_12345"
    }
    ```

## Data Models

### Machine
```rust
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
```

### User
```rust
pub struct User {
    pub id: i64,
    pub username: String,
    pub role: String,
    pub token: String,
}
```

### MaintenanceComment
```rust
pub struct MaintenanceComment {
    pub id: i64,
    pub machine_id: i64,
    pub comment: String,
    pub priority: String,
    pub username: String,
    pub created_at: i64,
}
```

## Example Interactions

### Creating a Machine
1. **Login** to get a token.
2. **Create a machine** using the token.
3. **List machines** to verify creation.

### Updating Machine Speed
1. **Login** to get a token.
2. **Update machine speed** using the machine's API key.

### Adding a Comment
1. **Login** to get a token.
2. **Add a comment** to a machine using the token.
3. **Get comments** to verify the comment was added.

## Technical Details

- **Database:** SQLite
- **Server:** Axum (Rust)
- **Authentication:** Token-based
- **API Format:** JSON

## Troubleshooting

- If you encounter issues, check the logs for detailed error messages.
- Ensure the database file exists and is writable. 