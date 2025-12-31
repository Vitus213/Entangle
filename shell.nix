# Legacy shell.nix for compatibility with nix-shell
# Use 'nix develop' with flakes for the full experience
let
  pkgs = import <nixpkgs> {
    overlays = [
      (import (builtins.fetchTarball "https://github.com/oxalica/rust-overlay/archive/master.tar.gz"))
    ];
  };

  rustToolchain = pkgs.rust-bin.stable.latest.default.override {
    extensions = [ "rust-src" "rust-analyzer" ];
  };
in
pkgs.mkShell {
  buildInputs = with pkgs; [
    rustToolchain
    pkg-config
    openssl
    postgresql
    redis
    docker
    docker-compose
    sqlx-cli
  ];

  shellHook = ''
    echo "🚀 Entangle Development Environment (legacy nix-shell)"
    echo "For better experience, use: nix develop"

    if [ -f .env ]; then
      export $(grep -v '^#' .env | xargs)
    fi
  '';
}
