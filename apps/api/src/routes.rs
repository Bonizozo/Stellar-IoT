use crate::auth;
use crate::auth_middleware::require_auth;
use crate::handlers;
use crate::webhook_handlers;
use crate::webhook_service::WebhookStore;
use axum::{
    middleware,
    routing::{get, post, delete},
    Router,
};

pub fn auth_routes() -> Router {
    Router::new()
        .route("/auth/challenge", get(auth::get_challenge))
        .route("/auth/verify", post(auth::verify_challenge))
}

pub fn device_routes() -> Router {
    let protected = Router::new()
        .route("/devices/:id/analytics", get(handlers::get_device_analytics))
        .layer(middleware::from_fn(require_auth));

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
        .merge(protected)
}

pub fn payment_routes() -> Router {
    Router::new()
        .route("/pay", post(handlers::process_payment))
        .route("/payments", get(handlers::get_payment_history))
}

pub fn earnings_routes() -> Router {
    Router::new()
        .route("/earnings", get(handlers::get_owner_earnings))
        .route("/earnings/devices", get(handlers::get_owner_devices))
        .route("/earnings/withdraw", post(handlers::withdraw_earnings))
}

pub fn webhook_routes() -> Router<WebhookStore> {
    Router::new()
        .route("/webhooks", post(webhook_handlers::register_webhook).get(webhook_handlers::list_webhooks))
        .route("/webhooks/:id", delete(webhook_handlers::delete_webhook))
        .route("/webhooks/:id/logs", get(webhook_handlers::get_webhook_logs))
        .route("/devices/:id/webhook-logs", get(webhook_handlers::get_device_webhook_logs))
}
