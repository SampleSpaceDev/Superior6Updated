#!/bin/bash

# Superior 6 Setup Script

set -e

echo "🚀 Setting up Superior 6..."

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust is not installed. Please install Rust first:"
    echo "   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# Check if PostgreSQL is running
if ! command -v psql &> /dev/null; then
    echo "❌ PostgreSQL is not installed. Please install PostgreSQL first."
    exit 1
fi

# Create .env file if it doesn't exist
if [ ! -f .env ]; then
    echo "📝 Creating .env file..."
    cp .env.example .env
    echo "⚠️  Please edit .env file with your database credentials before continuing."
    echo "   Press any key to continue after editing .env..."
    read -n 1 -s
fi

# Install SQLx CLI if not installed
if ! command -v sqlx &> /dev/null; then
    echo "📦 Installing SQLx CLI..."
    cargo install sqlx-cli --no-default-features --features rustls,postgres
fi

# Load environment variables
if [ -f .env ]; then
    export $(cat .env | xargs)
fi

# Create database if it doesn't exist
echo "🗄️  Setting up database..."
DB_NAME=$(echo $DATABASE_URL | sed 's/.*\///')
DB_URL_WITHOUT_NAME=$(echo $DATABASE_URL | sed 's/\/[^/]*$//')

# Try to create the database (will fail silently if it exists)
psql "$DB_URL_WITHOUT_NAME/postgres" -c "CREATE DATABASE $DB_NAME;" 2>/dev/null || true

# Run migrations
echo "🔄 Running database migrations..."
sqlx migrate run --source src/migrations

# Create static directories
echo "📁 Creating static directories..."
mkdir -p static/css static/js static/images
mkdir -p templates/auth templates/user templates/admin

# Build the project
echo "🔨 Building project..."
cargo build --release

# Create admin user script
echo "👤 Creating admin user helper script..."
cat > create_admin.sql << EOF
-- Run this to make a user an admin (replace email with actual email)
-- psql \$DATABASE_URL -c "UPDATE users SET is_admin = true WHERE email = 'your-email@example.com';"
EOF

echo "✅ Setup complete!"
echo ""
echo "🎯 Next steps:"
echo "1. Start the server: cargo run"
echo "2. Visit http://localhost:3000"
echo "3. Register a user account"
echo "4. Make yourself admin by running: psql \$DATABASE_URL -f create_admin.sql"
echo "   (Edit the SQL file first with your email)"
echo ""
echo "🏆 Happy predicting!"