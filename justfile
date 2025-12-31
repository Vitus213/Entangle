# Justfile - Command runner for development tasks
# Install just: https://github.com/casey/just

# List available commands
default:
    @just --list

# Install dependencies and setup environment
setup:
    @echo "🔧 Setting up development environment..."
    @if [ ! -f .env ]; then \
        echo "📝 Creating .env file..."; \
        cp .env.example .env; \
        echo "⚠️  Please update .env with your secrets"; \
    fi
    @mkdir -p uploads migrations
    @echo "✅ Setup complete"

# Generate random secrets for .env
secrets:
    @echo "🔐 Generating random secrets..."
    @echo ""
    @echo "APP_SECRET_KEY=$(openssl rand -hex 32)"
    @echo "JWT_SECRET=$(openssl rand -hex 32)"
    @echo ""
    @echo "Copy these to your .env file"

# Start databases with Docker Compose
db-up:
    docker-compose -f docker-compose.dev.yml up -d

# Stop databases
db-down:
    docker-compose -f docker-compose.dev.yml down

# View database logs
db-logs:
    docker-compose -f docker-compose.dev.yml logs -f

# Create a new migration
migrate-create NAME:
    sqlx migrate add {{NAME}}

# Run migrations
migrate:
    sqlx migrate run

# Revert last migration
migrate-revert:
    sqlx migrate revert

# Run the API server
run:
    cargo run --bin entangle-api

# Run with auto-reload on file changes
watch:
    watchexec -r -e rs,toml cargo run --bin entangle-api

# Run tests
test:
    cargo test

# Run linter
lint:
    cargo clippy -- -D warnings

# Format code
fmt:
    cargo fmt

# Check formatting without making changes
fmt-check:
    cargo fmt -- --check

# Build release version
build:
    cargo build --release

# Clean build artifacts
clean:
    cargo clean
    rm -rf target/

# Full check (fmt, clippy, test)
check: fmt-check lint test

# Development workflow (start databases and run server)
dev: db-up
    @echo "⏳ Waiting for openGauss to initialize (this takes ~30-40 seconds on first run)..."
    @sleep 35
    @echo "🔍 Checking container status..."
    @docker ps | grep entangle
    @just migrate
    @just run
