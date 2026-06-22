mod analytics;
mod config;
mod handlers;
mod models;
mod routes;
mod services;
mod stellar_service;
mod webhook;
mod webhook_handlers;
mod webhook_service;

use axum::{http::Method, Router};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use webhook_service::WebhookStore;

#[tokio::main]
async fn main() {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::DELETE])
        .allow_headers(Any);

    let store = WebhookStore::new();

    let app = Router::new()
        .merge(routes::device_routes())
        .merge(routes::payment_routes())
        .merge(routes::webhook_routes())
        .with_state(store)
        .layer(cors);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    println!("🚀 Server running on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}