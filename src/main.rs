use axum::{
    routing::{get, post, put},
    Router,
};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tokio::signal;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod auth;
mod database;
mod handlers;
mod models;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing with proper configuration
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();
    
    // Initialize database
    let db = match database::init_database().await {
        Ok(pool) => pool,
        Err(e) => {
            eprintln!("Failed to initialize database: {}", e);
            return Err(e);
        }
    };
    
    // Build routes
    let app = Router::new()
        .route("/api/login", post(handlers::login))
        .route("/api/machines", get(handlers::list_machines).post(handlers::create_machine))
        .route("/api/machines/update", post(handlers::update_machine_speed))
        .route("/api/machines/{id}/comments", get(handlers::get_comments).post(handlers::add_comment))
        .route("/api/machines/{id}/history", get(handlers::get_history))
        .route("/api/machines/{id}", put(handlers::update_machine))
        .route("/api/users", get(handlers::list_users).post(handlers::create_user))
        .route("/api/users/{id}", put(handlers::update_user))
        .layer(CorsLayer::permissive())
        .with_state(db);

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    println!("Server running on http://{}", addr);
    
    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Failed to bind to address {}: {}", addr, e);
            return Err(e.into());
        }
    };
    
    // Handle graceful shutdown
    let server = axum::serve(listener, app);
    
    if let Err(e) = server.with_graceful_shutdown(shutdown_signal()).await {
        eprintln!("Server error: {}", e);
        return Err(e.into());
    }
    
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    println!("\nShutting down gracefully...");
}