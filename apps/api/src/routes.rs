use axum::{Router, routing::{get, post}};
use crate::handlers;

pub fn device_routes() -> Router {
    Router::new()
        .route("/devices", get(handlers::get_devices))
        .route("/devices/search", get(handlers::search_devices))
        .route("/session/:id", get(handlers::get_session))
}

pub fn payment_routes() -> Router {
    Router::new()
        .route("/pay", post(handlers::process_payment))
}
