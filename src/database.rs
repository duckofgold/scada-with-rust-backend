use sqlx::SqlitePool;
use std::time::{SystemTime, UNIX_EPOCH};
use std::path::Path;
use std::fs;

pub type DbPool = SqlitePool;

pub async fn init_database() -> anyhow::Result<DbPool> {
    let db_path = "database.db";
    
    // Check if database file exists and is writable
    if Path::new(db_path).exists() {
        // Check if file is writable
        if let Err(e) = fs::OpenOptions::new()
            .write(true)
            .open(db_path)
        {
            return Err(anyhow::anyhow!("Database file exists but is not writable: {}", e));
        }
    }
    
    let pool = SqlitePool::connect(&format!("sqlite:{}", db_path)).await?;
    
    // Create tables
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS machines (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE,
            code TEXT NOT NULL UNIQUE,
            api_key TEXT NOT NULL UNIQUE,
            location TEXT,
            machine_type TEXT,
            current_speed REAL DEFAULT 0.0,
            status_message TEXT DEFAULT '',
            last_update INTEGER DEFAULT 0,
            is_online BOOLEAN DEFAULT 0,
            created_at INTEGER DEFAULT (strftime('%s', 'now'))
        )
    "#).execute(&pool).await?;

    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT NOT NULL UNIQUE,
            password TEXT NOT NULL,
            role TEXT NOT NULL CHECK (role IN ('admin', 'manager', 'technician')),
            token TEXT UNIQUE,
            created_at INTEGER DEFAULT (strftime('%s', 'now'))
        )
    "#).execute(&pool).await?;

    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS maintenance_comments (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            machine_id INTEGER NOT NULL,
            username TEXT NOT NULL,
            comment TEXT NOT NULL,
            priority TEXT DEFAULT 'normal' CHECK (priority IN ('low', 'normal', 'high', 'critical')),
            created_at INTEGER DEFAULT (strftime('%s', 'now')),
            FOREIGN KEY (machine_id) REFERENCES machines (id)
        )
    "#).execute(&pool).await?;

    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS speed_history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            machine_id INTEGER NOT NULL,
            speed REAL NOT NULL,
            message TEXT,
            timestamp INTEGER DEFAULT (strftime('%s', 'now')),
            FOREIGN KEY (machine_id) REFERENCES machines (id)
        )
    "#).execute(&pool).await?;

    // Insert hardcoded admin user
    sqlx::query(r#"
        INSERT OR IGNORE INTO users (username, password, role, token) 
        VALUES ('admin', 'admin123', 'admin', 'admin_token_12345')
    "#).execute(&pool).await?;

    // Create indexes
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_machines_api_key ON machines(api_key)").execute(&pool).await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_speed_history_machine ON speed_history(machine_id)").execute(&pool).await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_maintenance_machine ON maintenance_comments(machine_id)").execute(&pool).await?;

    Ok(pool)
}

pub fn current_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}