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

    // Create WebSocket hub with database pool for persistence
    let ws_hub = WsHub::with_pool(pool.clone());
    tracing::info!("WebSocket hub initialized with auto-save enabled");

    // Create application state
    let _state = AppState {
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
        // WebSocket route (需要 pool extension 用于权限检查)
        .route("/ws/documents/:id", get(ws::handlers::websocket_handler))
        .layer(axum::Extension(pool.clone()))  // Pool extension for WebSocket
        .layer(axum::Extension(ws_hub.clone())) // WsHub extension
        // Public routes (no auth required)
        .nest("/api/auth", routes::user_public_routes())
        // Protected routes (auth required)
        .nest("/api", routes::user_protected_routes())
        .nest("/api", routes::document_routes())
        .nest("/api", routes::folder_routes())
        .nest("/api", routes::tag_routes())
        .nest("/api", routes::comment_routes())
        .nest("/api", routes::notification_routes())
        .nest("/api", routes::task_routes())
        .layer(cors)
        .with_state(pool);

    // Start server
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;

    // Handle graceful shutdown
    let ws_hub_shutdown = ws_hub.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        tracing::info!("Shutdown signal received, saving all documents...");
        ws_hub_shutdown.save_all_dirty().await;
        tracing::info!("All documents saved");
        std::process::exit(0);
    });

    axum::serve(listener, app).await?;

    Ok(())
}
