use axum::{
    extract::ws::{WebSocket, WebSocketUpgrade},
    response::Response,
    routing::get,
    Router,
    Extension,
};
use std::net::SocketAddr;
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

    tracing::info!("==! starting Qwew backend !==");

    dotenvy::dotenv().ok();

    let config = config::AppConfig::from_env();

    let pool = db::connection::create_pool().await
        .expect("failed to connect to PostgreSQL. Is the Docker container running?");

    tracing::info!("database pool created successfully");

    let app = Router::new()
        .route("/", get(|| async { "Qwew backend is running" }))
        .route("/ws", get(ws_handler))
        .route("/auth/register", axum::routing::post(handlers::auth::register))

        .layer(Extension(pool))
        .layer(Extension(config))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    Extension(pool): Extension<db::PgPool>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, pool))
}

async fn handle_socket(mut socket: WebSocket, pool: db::PgPool) {
    tracing::info!("new WebSocket client connected");

    // TODO: auth

    while let Some(msg) = socket.recv().await {
        match msg {
            Ok(msg) => {
                if let Err(e) = socket.send(msg).await {
                    tracing::error!("WebSocket send error: {}", e);
                    break;
                }
            }
            Err(e) => {
                tracing::error!("WebSocket error: {}", e);
                break;
            }
        }
    }

    tracing::info!("WebSocket client disconnected");
}
