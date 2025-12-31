{
  description = "Entangle - Multi-user collaborative document editing system";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, fenix, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };

        # Use fenix for Rust toolchain - faster and more modern
        rustToolchain = fenix.packages.${system}.complete.withComponents [
          "cargo"
          "clippy"
          "rust-src"
          "rustc"
          "rustfmt"
          "rust-analyzer"
        ];

        # Environment variables
        env = {
          # Application
          APP_NAME = "Entangle";
          APP_ENV = "development";
          APP_PORT = "3000";
          APP_HOST = "127.0.0.1";

          # Database
          DATABASE_URL = "postgres://omm:Entangle@2024@localhost:5432/postgres";
          DATABASE_MAX_CONNECTIONS = "10";

          # Redis
          REDIS_URL = "redis://localhost:6379";
          REDIS_POOL_SIZE = "10";

          # JWT (use secrets management for production)
          JWT_ACCESS_EXPIRY = "3600";
          JWT_REFRESH_EXPIRY = "604800";

          # Storage
          STORAGE_TYPE = "local";
          STORAGE_PATH = "./uploads";
          MAX_UPLOAD_SIZE = "10485760";

          # Logging
          LOG_LEVEL = "info";
          LOG_FORMAT = "pretty";
          RUST_LOG = "entangle_api=debug,tower_http=debug";

          # CORS
          CORS_ALLOWED_ORIGINS = "http://localhost:5173,http://localhost:3000";
          CORS_ALLOWED_METHODS = "GET,POST,PUT,DELETE,OPTIONS";
          CORS_ALLOWED_HEADERS = "Content-Type,Authorization";

          # WebSocket
          WS_HEARTBEAT_INTERVAL = "30";
          WS_CLIENT_TIMEOUT = "60";
        };

      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            # Rust toolchain
            rustToolchain

            # Build dependencies
            pkg-config
            openssl

            # Database tools
            postgresql
            sqlx-cli

            # Redis
            redis

            # Docker for containers
            docker
            docker-compose

            # Development tools
            just
            watchexec

            # Optional: Node.js for frontend
            nodejs_20
            nodePackages.pnpm
          ];

          shellHook = ''
            echo "🚀 Entangle Development Environment"
            echo "=================================="
            echo "Rust: $(rustc --version)"
            echo "Cargo: $(cargo --version)"
            echo ""

            # Set environment variables
            ${pkgs.lib.concatStringsSep "\n"
              (pkgs.lib.mapAttrsToList (name: value: "export ${name}=\"${value}\"") env)}

            # Check Docker mirror configuration
            if [ -f /etc/docker/daemon.json ]; then
              if grep -q "registry-mirrors" /etc/docker/daemon.json 2>/dev/null; then
                echo "✅ Docker 镜像加速器已配置"
              else
                echo "⚠️  Docker 镜像加速器未配置"
                echo "   运行: ./scripts/setup-docker-mirror.sh"
              fi
            else
              echo "⚠️  Docker 镜像加速器未配置"
              echo "   运行: ./scripts/setup-docker-mirror.sh"
            fi
            echo ""

            # Load secrets from .env if exists (for sensitive data)
            if [ -f .env ]; then
              echo "📝 Loading additional secrets from .env"
              export $(grep -v '^#' .env | xargs)
            else
              echo "⚠️  No .env file found. Create one for secrets (APP_SECRET_KEY, JWT_SECRET)"
              echo "   You can copy from .env.example and add your secrets"
            fi

            # Create necessary directories
            mkdir -p uploads
            mkdir -p migrations

            echo ""
            echo "📦 Available commands:"
            echo "  just dev               - Start complete dev environment"
            echo "  just db-up             - Start databases only"
            echo "  cargo run              - Run the API server"
            echo "  cargo test             - Run tests"
            echo "  cargo clippy           - Run linter"
            echo ""
          '';

          # Set library path for linked libraries
          LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath [ pkgs.openssl ]}";
        };

        # Package definition (optional, for building the project)
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "entangle";
          version = "0.1.0";
          src = ./.;
          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          buildInputs = with pkgs; [
            openssl
            pkg-config
          ];
        };
      }
    );
}
