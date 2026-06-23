use crate::webhook_service::WebhookStore;
use crate::{handlers, webhook_handlers};
use axum::{
    routing::{delete, get, post},
    Router,
};

pub fn device_routes() -> Router<WebhookStore> {
    Router::new()
        .route("/devices/search", get(handlers::search_devices))
        .route("/devices/:id/heartbeat", post(handlers::device_heartbeat))
        .route("/sessions", get(handlers::get_sessions))
        .route(
            "/session/:id",
            get(handlers::get_session).delete(handlers::end_session),
        )
        .route("/session/:id/extend", post(handlers::extend_session))
        .route("/session/:id/telemetry", get(handlers::telemetry_ws))
        .route(
            "/devices/:id/analytics",
            get(handlers::get_device_analytics),
        )
        .route(
            "/devices/:id/webhook-logs",
            get(webhook_handlers::get_device_webhook_logs),
        )
}

pub fn payment_routes() -> Router<WebhookStore> {
    Router::new().route("/pay", post(handlers::process_payment))
}

pub fn webhook_routes() -> Router<WebhookStore> {
    Router::new()
        .route("/webhooks", post(webhook_handlers::register_webhook))
        .route("/webhooks", get(webhook_handlers::list_webhooks))
        .route("/webhooks/:id", delete(webhook_handlers::delete_webhook))
        .route(
            "/webhooks/:id/logs",
            get(webhook_handlers::get_webhook_logs),
        )
}
