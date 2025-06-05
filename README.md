# SCADA Backend

## Overview
This is a simple SCADA backend built with Rust. It provides APIs for managing machines, updating their speeds, and adding maintenance comments.

## Quickstart

1. **Clone the repo and install dependencies:**
   - Requires Rust (https://rustup.rs/) and SQLite3

2. **Run the development script:**
   ```bash
   ./run-dev.sh
   ```
   This will build the project, ensure the database file exists, and start the server on port 8080.

## API Examples

### Login
```http
POST http://localhost:8080/api/login
Content-Type: application/json

{
  "username": "admin",
  "password": "admin123"
}
```

### Create a Machine
```http
POST http://localhost:8080/api/machines
Authorization: Bearer TOKEN
Content-Type: application/json

{
  "name": "Test Machine",
  "code": "MCH-001",
  "location": "Plant 1",
  "machine_type": "Conveyor"
}
```

### List Machines
```http
GET http://localhost:8080/api/machines
Authorization: Bearer TOKEN
```

### Update Machine Speed
```http
POST http://localhost:8080/api/machines/update
Authorization: Bearer MACHINE_API_KEY
Content-Type: application/json

{
  "speed": 123.45,
  "message": "Running at test speed"
}
```

### Add a Comment
```http
POST http://localhost:8080/api/machines/{id}/comments
Authorization: Bearer TOKEN
Content-Type: application/json

{
  "comment": "Routine check completed.",
  "priority": "normal"
}
```

### Get Comments
```http
GET http://localhost:8080/api/machines/{id}/comments
Authorization: Bearer TOKEN
```

### Create a User
```http
POST http://localhost:8080/api/users
Authorization: Bearer TOKEN
Content-Type: application/json

{
  "username": "user1",
  "password": "userpass",
  "role": "manager"
}
```

## ESP32 Integration Example

Here's a simple example of how to integrate an ESP32 with this backend:

```cpp
#include <WiFi.h>
#include <HTTPClient.h>

const char* ssid = "YOUR_WIFI_SSID";
const char* password = "YOUR_WIFI_PASSWORD";
const char* serverUrl = "http://localhost:8080/api/machines/update";
const char* apiKey = "YOUR_MACHINE_API_KEY";

void setup() {
  Serial.begin(115200);
  WiFi.begin(ssid, password);
  while (WiFi.status() != WL_CONNECTED) {
    delay(1000);
    Serial.println("Connecting to WiFi...");
  }
  Serial.println("Connected to WiFi");
}

void loop() {
  if (WiFi.status() == WL_CONNECTED) {
    HTTPClient http;
    http.begin(serverUrl);
    http.addHeader("Authorization", "Bearer " + String(apiKey));
    http.addHeader("Content-Type", "application/json");

    String jsonData = "{\"speed\": 123.45, \"message\": \"ESP32 update\"}";
    int httpResponseCode = http.POST(jsonData);

    if (httpResponseCode > 0) {
      String response = http.getString();
      Serial.println(httpResponseCode);
      Serial.println(response);
    } else {
      Serial.println("Error on sending POST: " + String(httpResponseCode));
    }
    http.end();
  }
  delay(5000);
}
```

## Troubleshooting

- If you see `unable to open database file`, ensure the working directory is writable and the file exists (the script will create it if missing).
- Check logs for detailed error messages.
