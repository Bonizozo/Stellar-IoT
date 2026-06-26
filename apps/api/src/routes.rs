use crate::handlers;
use crate::webhook_handlers;
use crate::webhook_service::WebhookStore;
use axum::{
    routing::{get, post, delete},
    Router,
};

pub fn device_routes() -> Router {
    Router::new()
        .route("/devices", get(handlers::list_devices).post(handlers::register_device))
        .route(
            "/devices/:id",
            get(handlers::get_managed_device)
                .put(handlers::update_device)
                .delete(handlers::delete_device),
        )
        .route("/devices/search", get(handlers::search_devices))
        .route("/devices/:id/heartbeat", post(handlers::device_heartbeat))
        .route("/devices/:id/telemetry", post(handlers::upload_telemetry))
        .route("/devices/:id/reviews", post(handlers::add_device_review).get(handlers::get_device_reviews))
        .route("/devices/:id/qr-scan", post(handlers::record_qr_scan))
        .route("/devices/:id/qr-analytics", get(handlers::get_qr_analytics))
        .route("/sessions", get(handlers::get_sessions))
        .route("/session/:id", get(handlers::get_session).delete(handlers::end_session))
        .route("/session/:id/extend", post(handlers::extend_session))
        .route("/session/:id/telemetry", get(handlers::telemetry_ws))
        .route("/devices/:id/analytics", get(handlers::get_device_analytics))
}

pub fn payment_routes() -> Router {
    Router::new()
        .route("/pay", post(handlers::process_payment))
        .route("/payments", get(handlers::get_payment_history))
}

pub fn webhook_routes() -> Router<WebhookStore> {
    Router::new()
        .route("/webhooks", post(webhook_handlers::register_webhook).get(webhook_handlers::list_webhooks))
        .route("/webhooks/:id", delete(webhook_handlers::delete_webhook))
        .route("/webhooks/:id/logs", get(webhook_handlers::get_webhook_logs))
        .route("/devices/:id/webhook-logs", get(webhook_handlers::get_device_webhook_logs))
}
