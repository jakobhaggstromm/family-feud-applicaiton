mod models;
mod state;
mod handlers;

use axum::{
    routing::{get, post},
    Router,
};
use std::sync::{Arc, Mutex};

use handlers::{start_round, buzz, join, get_state, leave};
use state::RoundState;

#[tokio::main]
async fn main() {

    let state = Arc::new(Mutex::new(RoundState::new()));

    let app = Router::new()
        .route("/join", post(join))
        .route("/start", post(start_round))
        .route("/buzz", post(buzz))
        .route("/state", get(get_state))
        .route("/leave",post(leave))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();

    println!("Server running on port 3000");

    axum::serve(listener, app)
        .await
        .unwrap();
}