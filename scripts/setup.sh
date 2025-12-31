#!/usr/bin/env bash
# Setup script for development environment

set -e

echo "🔧 Entangle Development Environment Setup"
echo "=========================================="
echo ""

# Check if Nix is installed
if ! command -v nix &> /dev/null; then
    echo "❌ Nix is not installed"
    echo "Please install Nix: https://nixos.org/download.html"
    exit 1
fi

echo "✅ Nix found: $(nix --version)"

# Enable flakes if not already enabled
if ! nix flake metadata &> /dev/null; then
    echo "⚙️  Enabling Nix flakes..."
    mkdir -p ~/.config/nix
    echo "experimental-features = nix-command flakes" >> ~/.config/nix/nix.conf
fi

# Create .env from .env.example if it doesn't exist
if [ ! -f .env ]; then
    echo "📝 Creating .env file..."
    cat > .env << 'EOF'
# Secrets file - DO NOT COMMIT TO GIT
# Generate random secrets with: openssl rand -hex 32

APP_SECRET_KEY=$(openssl rand -hex 32)
JWT_SECRET=$(openssl rand -hex 32)
EOF
    echo "✅ Created .env with generated secrets"
else
    echo "ℹ️  .env already exists, skipping..."
fi

# Create necessary directories
echo "📁 Creating directories..."
mkdir -p uploads migrations

# Install direnv if not already installed (optional but recommended)
if ! command -v direnv &> /dev/null; then
    echo "⚠️  direnv not found. Install it for automatic environment loading:"
    echo "   nix profile install nixpkgs#direnv"
else
    echo "✅ direnv found"
    direnv allow .
fi

echo ""
echo "✅ Setup complete!"
echo ""
echo "Next steps:"
echo "1. Enter the development environment:"
echo "   nix develop"
echo ""
echo "2. Or use direnv for automatic loading:"
echo "   direnv allow"
echo ""
echo "3. Start databases:"
echo "   just db-up"
echo ""
echo "4. Run the development server:"
echo "   just dev"
echo ""
