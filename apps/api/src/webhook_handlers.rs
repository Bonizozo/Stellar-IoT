//! HTTP handlers for the webhook management API.
//!
//! Routes:
//!   POST   /webhooks                          Register a webhook
//!   DELETE /webhooks/:id                      Delete a webhook
//!   GET    /webhooks?device_id=<id>           List webhooks for a device
//!   GET    /webhooks/:id/logs                 Delivery logs for a webhook
//!   GET    /devices/:id/webhook-logs          All delivery logs for a device

use crate::webhook::{RegisterWebhookRequest, WebhookRegistration, WebhookView};
use crate::webhook_service::WebhookStore;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;

// ── Register ──────────────────────────────────────────────────────────────

/// `POST /webhooks`
///
/// Registers a new webhook for a device. Returns the created record
/// (secret is omitted from the response).
pub async fn register_webhook(
    State(store): State<WebhookStore>,
    Json(req): Json<RegisterWebhookRequest>,
) -> Result<Json<WebhookView>, (StatusCode, String)> {
    if req.url.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "url is required".into()));
    }
    if req.secret.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "secret is required".into()));
    }
    if req.device_id.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "device_id is required".into()));
    }

    let reg = WebhookRegistration::new(req);
    let view = WebhookView::from(&reg);
    store.insert(reg);
    Ok(Json(view))
}

// ── Delete ────────────────────────────────────────────────────────────────

/// `DELETE /webhooks/:id`
///
/// Deletes a registered webhook. Returns 404 if the webhook does not exist.
pub async fn delete_webhook(
    State(store): State<WebhookStore>,
    Path(id): Path<String>,
) -> StatusCode {
    if store.delete(&id) {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}

// ── List ──────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct ListWebhooksQuery {
    pub device_id: String,
}

/// `GET /webhooks?device_id=<id>`
///
/// Returns all webhooks registered for the given device.
pub async fn list_webhooks(
    State(store): State<WebhookStore>,
    Query(q): Query<ListWebhooksQuery>,
) -> Json<Vec<WebhookView>> {
    let webhooks = store.list_for_device(&q.device_id);
    Json(webhooks.iter().map(WebhookView::from).collect())
}

// ── Delivery logs ─────────────────────────────────────────────────────────

/// `GET /webhooks/:id/logs`
///
/// Returns all delivery log entries for a specific webhook.
pub async fn get_webhook_logs(
    State(store): State<WebhookStore>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if store.get(&id).is_none() {
        return Err(StatusCode::NOT_FOUND);
    }
    let logs = store.logs_for_webhook(&id);
    Ok(Json(serde_json::json!({ "webhook_id": id, "logs": logs })))
}

/// `GET /devices/:id/webhook-logs`
///
/// Returns all delivery log entries across every webhook registered for a device.
pub async fn get_device_webhook_logs(
    State(store): State<WebhookStore>,
    Path(device_id): Path<String>,
) -> Json<serde_json::Value> {
    let logs = store.logs_for_device(&device_id);
    Json(serde_json::json!({ "device_id": device_id, "logs": logs }))
}