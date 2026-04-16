mod handlers;
mod host_handlers;
mod models;
mod player_handlers;
mod readers;
mod state;

use axum::{routing::get, Router};
use std::env;
use std::sync::{Arc, Mutex};

use handlers::ws_handler;
use state::GameState;

#[tokio::main]
async fn main() {
    let admin_password =
        env::var("FAMILY_FEUD_ADMIN_PASSWORD").unwrap_or_else(|_| "admin123".to_string());

    let state = Arc::new(Mutex::new(GameState::new(admin_password)));

    let app = Router::new()
        .route("/ws", get(ws_handler))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!("Server running on port 3000 (WebSocket at ws://0.0.0.0:3000/ws)");
    println!("Admin password loaded from FAMILY_FEUD_ADMIN_PASSWORD (default applied if unset)");

    axum::serve(listener, app).await.unwrap();
}
