mod middleware;
mod routes;
mod ws;

use axum::{routing::get, Router};
use entangle_db::create_pool;
use sqlx::PgPool;
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use ws::WsHub;

/// Application state
#[derive(Clone)]
struct AppState {
    pool: PgPool,
    ws_hub: WsHub,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "entangle_api=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load environment variables
    dotenvy::dotenv().ok();

    // Get database URL from environment
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // Create database pool
    tracing::info!("Connecting to database...");
    let pool = create_pool(&database_url).await?;
    tracing::info!("Database connected successfully");

    // Create WebSocket hub
    let ws_hub = WsHub::new();

    // Create application state
    let state = AppState {
        pool: pool.clone(),
        ws_hub: ws_hub.clone(),
    };

    // Setup CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build application router
    let app = Router::new()
        .route("/", get(|| async { "Entangle API Server" }))
        .route("/health", get(|| async { "OK" }))
        // WebSocket route
        .route("/ws/documents/:id", get(ws::handlers::websocket_handler))
        // Public routes (no auth required)
        .nest("/api/auth", routes::user_public_routes())
        // Protected routes (auth required)
        .nest("/api", routes::user_protected_routes())
        .nest("/api", routes::document_routes())
        .nest("/api", routes::folder_routes())
        .nest("/api", routes::tag_routes())
        .layer(cors)
        .layer(axum::Extension(ws_hub))
        .with_state(pool);

    // Start server
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
