use crate::handlers;
use axum::{
    routing::{get, post, delete},
    Router,
};

pub fn device_routes() -> Router {
    Router::new()
        .route("/devices", get(handlers::get_devices))
        .route("/devices/search", get(handlers::search_devices))
        .route("/devices/:id/heartbeat", post(handlers::device_heartbeat))
        .route("/sessions", get(handlers::get_sessions))
        .route("/session/:id", get(handlers::get_session).delete(handlers::end_session))
        .route("/session/:id/extend", post(handlers::extend_session))
        .route("/session/:id/telemetry", get(handlers::telemetry_ws))
        .route("/devices/:id/analytics", get(handlers::get_device_analytics))
        .route("/session/:id", get(handlers::get_session))
}

pub fn payment_routes() -> Router {
    Router::new().route("/pay", post(handlers::process_payment))
}
