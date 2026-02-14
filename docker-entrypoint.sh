#!/bin/sh
set -e

DB_URL="${DATABASE_URL:-sqlite:/app/data/splitify.db}"
DB_PATH=$(echo "$DB_URL" | sed 's|^sqlite:||')
DB_DIR=$(dirname "$DB_PATH")

if [ -f "$DB_PATH" ]; then
    echo "Database file found - will run migrations to ensure schema is up-to-date"
    DB_EXISTS=true
else
    echo "Database file not found - will create new database and run initial migrations"
    DB_EXISTS=false
fi

echo "========================================="
echo "Running database setup..."
echo "========================================="

cd /app

if [ "$DB_EXISTS" = false ]; then
    echo "Creating new database..."
    sqlx database create
    echo "Database created successfully"
    
    echo "Running initial migrations..."
    sqlx migrate run
    echo "Initial migrations completed"
else
    echo "Database already exists, checking for new migrations..."
    sqlx migrate run
    echo "Migrations up to date"
fi

echo "========================================="
echo "Starting Splitify application..."
echo "========================================="

# Execute the application (replaces this shell process)
exec /app/rustify-app
