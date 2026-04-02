use axum::{
    routing::{get, post},
    Router, Extension,
};
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::sync::RwLock;
use ws::tickets::WsTickets;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod db;
mod handlers;
mod models;
mod routes;
mod ws;
mod utils;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("== starting Qwew backend ==");

    dotenvy::dotenv().ok();

    let config = config::AppConfig::from_env();
    let pool = db::connection::create_pool().await
        .expect("failed to connect to PostgreSQL. make sure to start the container");

    tracing::info!("database pool created successfully");

    let connected_users: ws::ConnectedUsers = Arc::new(RwLock::new(HashMap::new()));
    let ws_tickets: WsTickets = Arc::new(RwLock::new(HashMap::new()));

    let app = Router::new()
        .route("/", get(|| async { "Qwew backend is running" }))

        // auth routes
        .route("/auth/register", post(handlers::auth::register))
        .route("/auth/login", post(handlers::auth::login))
        .route("/auth/me", get(handlers::auth::get_me))

        // invites
        .route("/invites", post(handlers::invites::create_invite))

        // messages
        .route("/conversations", get(handlers::messages::get_conversations))
        .route("/conversations/:id/messages", get(handlers::messages::get_history))

        // WebSocket
        .route("/ws/ticket", post(handlers::ws::issue_ticket))
        .route("/ws", get(ws::ws_handler))

        .layer(Extension(ws_tickets))
        .layer(Extension(connected_users))
        .layer(Extension(pool))
        .layer(Extension(config))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    axum::serve(listener, app)
        .await
        .unwrap();
}


