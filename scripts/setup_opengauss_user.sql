-- Create database user for Entangle application
-- This script creates a new user since OpenGauss forbids remote connections with initial user

CREATE USER entangle WITH PASSWORD 'Entangle@2024' SYSID 600;
ALTER USER entangle WITH CREATEDB CREATEROLE;

-- Create database
CREATE DATABASE entangle_db OWNER entangle;
\c entangle_db

-- Grant necessary permissions
GRANT ALL PRIVILEGES ON DATABASE entangle_db TO entangle;

-- Show created user
\du entangle
