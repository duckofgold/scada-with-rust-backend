#!/usr/bin/env bash
set -e

DB_FILE="database.db"

# Build the project
echo "[INFO] Building the project..."
cargo build

echo "[INFO] Checking database file..."
if [ ! -f "$DB_FILE" ]; then
    echo "[INFO] Creating database file: $DB_FILE"
    touch "$DB_FILE"
fi
chmod 664 "$DB_FILE"
echo "[INFO] Database file is ready."

echo "[INFO] Starting the SCADA backend server..."
cargo run 