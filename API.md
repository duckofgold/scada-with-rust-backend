# SCADA API Documentation

## Authentication

### Login
Authenticates a user and returns a token.

**Endpoint:** `POST /api/login`

**Request Headers:**
```
Content-Type: application/json
```

**Request Body:**
```json
{
    "username": "admin",
    "password": "admin123"
}
```

**Success Response:**
- **Code:** 200 OK
- **Content:**
```json
{
    "token": "user_123456789",
    "role": "admin",
    "username": "admin"
}
```

**Error Response:**
- **Code:** 401 Unauthorized
- **Content:**
```json
{
    "error": "Invalid credentials"
}
```

## Machine Management

### List Machines
Retrieves a list of all machines.

**Endpoint:** `GET /api/machines`

**Authentication:** Required (Admin or User)

**Request Headers:**
```
Authorization: Bearer <token>
```

**Success Response:**
- **Code:** 200 OK
- **Content:**
```json
{
    "machines": [
        {
            "id": 1,
            "name": "Machine 1",
            "code": "M001",
            "location": "Factory A",
            "machine_type": "Type A",
            "current_speed": 100.0,
            "status_message": "Running",
            "is_online": true,
            "last_update": 1234567890
        }
    ]
}
```

### Create Machine
Creates a new machine.

**Endpoint:** `POST /api/machines`

**Authentication:** Required (Admin only)

**Request Headers:**
```
Authorization: Bearer <token>
Content-Type: application/json
```

**Request Body:**
```json
{
    "name": "New Machine",
    "code": "M002",
    "location": "Factory B",
    "machine_type": "Type B"
}
```

**Success Response:**
- **Code:** 201 Created
- **Content:**
```json
{
    "id": 2,
    "name": "New Machine",
    "code": "M002",
    "api_key": "machine_123456789",
    "location": "Factory B",
    "machine_type": "Type B"
}
```

### Update Machine
Updates an existing machine's information.

**Endpoint:** `PUT /api/machines/{id}`

**Authentication:** Required (Admin only)

**Request Headers:**
```
Authorization: Bearer <token>
Content-Type: application/json
```

**Request Body:**
```json
{
    "name": "New Machine Name",     // Optional
    "code": "NEW_CODE",            // Optional
    "location": "New Location",     // Optional
    "machine_type": "New Type",     // Optional
    "regenerate_api_key": true      // Optional, if true generates a new API key
}
```

**Success Response:**
- **Code:** 200 OK
- **Content:**
```json
{
    "id": 1,
    "name": "New Machine Name",
    "code": "NEW_CODE",
    "api_key": "machine_123456789",
    "location": "New Location",
    "machine_type": "New Type"
}
```

### Update Machine Speed
Updates a machine's speed and status.

**Endpoint:** `POST /api/machines/update`

**Authentication:** Required (Machine API Key)

**Request Headers:**
```
Authorization: Bearer <machine_api_key>
Content-Type: application/json
```

**Request Body:**
```json
{
    "speed": 150.0,
    "message": "Running at full capacity"  // Optional
}
```

**Success Response:**
- **Code:** 200 OK
- **Content:**
```json
{
    "success": true,
    "timestamp": 1234567890
}
```

### Get Machine Comments
Retrieves comments for a specific machine.

**Endpoint:** `GET /api/machines/{id}/comments`

**Authentication:** Required (Admin or User)

**Request Headers:**
```
Authorization: Bearer <token>
```

**Success Response:**
- **Code:** 200 OK
- **Content:**
```json
{
    "comments": [
        {
            "id": 1,
            "machine_id": 1,
            "comment": "Maintenance required",
            "priority": "high",
            "username": "admin",
            "created_at": 1234567890
        }
    ]
}
```

### Add Machine Comment
Adds a comment to a specific machine.

**Endpoint:** `POST /api/machines/{id}/comments`

**Authentication:** Required (Admin or User)

**Request Headers:**
```
Authorization: Bearer <token>
Content-Type: application/json
```

**Request Body:**
```json
{
    "comment": "Maintenance required",
    "priority": "high"  // Optional, defaults to "normal"
}
```

**Success Response:**
- **Code:** 201 Created
- **Content:**
```json
{
    "id": 1,
    "machine_id": 1,
    "comment": "Maintenance required",
    "priority": "high",
    "username": "admin",
    "created_at": 1234567890
}
```

### Get Machine History
Retrieves speed history for a specific machine.

**Endpoint:** `GET /api/machines/{id}/history`

**Authentication:** Required (Admin or User)

**Request Headers:**
```
Authorization: Bearer <token>
```

**Query Parameters:**
- `limit`: Optional, number of history entries to return (default: 100)

**Success Response:**
- **Code:** 200 OK
- **Content:**
```json
{
    "history": [
        {
            "speed": 150.0,
            "message": "Running at full capacity",
            "timestamp": 1234567890
        }
    ]
}
```

## User Management

### List Users
Retrieves a list of all users.

**Endpoint:** `GET /api/users`

**Authentication:** Required (Admin only)

**Request Headers:**
```
Authorization: Bearer <token>
```

**Success Response:**
- **Code:** 200 OK
- **Content:**
```json
{
    "users": [
        {
            "id": 1,
            "username": "admin",
            "role": "admin",
            "token": "user_123456789"
        }
    ]
}
```

### Create User
Creates a new user.

**Endpoint:** `POST /api/users`

**Authentication:** Required (Admin only)

**Request Headers:**
```
Authorization: Bearer <token>
Content-Type: application/json
```

**Request Body:**
```json
{
    "username": "new_user",
    "password": "password123",
    "role": "manager"  // Must be one of: "admin", "manager", "technician"
}
```

**Success Response:**
- **Code:** 201 Created
- **Content:**
```json
{
    "id": 2,
    "username": "new_user",
    "role": "manager",
    "token": "user_123456789"
}
```

### Update User
Updates an existing user's information.

**Endpoint:** `PUT /api/users/{id}`

**Authentication:** Required (Admin only)

**Request Headers:**
```
Authorization: Bearer <token>
Content-Type: application/json
```

**Request Body:**
```json
{
    "password": "new_password",  // Optional
    "role": "manager",          // Optional, must be one of: "admin", "manager", "technician"
    "is_active": true          // Optional
}
```

**Success Response:**
- **Code:** 200 OK
- **Content:**
```json
{
    "id": 1,
    "username": "john_doe",
    "role": "manager",
    "token": "user_123456789"
}
```

## Common Error Responses

### Unauthorized (401)
```json
{
    "error": "Missing token"
}
```
or
```json
{
    "error": "Admin access required"
}
```
or
```json
{
    "error": "Invalid token"
}
```

### Not Found (404)
```json
{
    "error": "Machine not found"
}
```
or
```json
{
    "error": "User not found"
}
```

### Bad Request (400)
```json
{
    "error": "Machine name or code already exists"
}
```
or
```json
{
    "error": "Username already exists"
}
```
or
```json
{
    "error": "Invalid role. Must be one of: admin, manager, technician"
}
```
or
```json
{
    "error": "No fields to update"
}
```

### Internal Server Error (500)
```json
{
    "error": "Database error"
}
```
or
```json
{
    "error": "Failed to update machine"
}
```
or
```json
{
    "error": "Failed to fetch updated machine"
}
```
or
```json
{
    "error": "Failed to update user"
}
```
or
```json
{
    "error": "Failed to fetch updated user"
}
```

## Notes
- All endpoints except `/api/login` require authentication
- Admin-only endpoints require the user to have the "admin" role
- Machine API keys are only used for speed updates
- User tokens are used for all other authenticated endpoints
- The API key will only be regenerated if explicitly requested
- All timestamps are Unix timestamps (seconds since epoch)
- Comments can have priorities: "low", "normal", "high", "critical"
- User roles can be: "admin", "manager", "technician" 