//! Webhook delivery service.
//!
//! Responsibilities:
//! - In-memory store of registered webhooks and delivery logs
//! - HMAC-SHA256 payload signing (`X-Webhook-Signature` header)
//! - HTTP delivery with exponential backoff (up to 5 attempts)
//! - Delivery log persistence

use crate::webhook::{
    DeliveryStatus, WebhookDeliveryLog, WebhookEventType, WebhookPayload, WebhookRegistration,
};
use hmac::{Hmac, Mac};
use reqwest::Client;
use sha2::Sha256;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::time::{sleep, Duration};
use uuid::Uuid;
use chrono::Utc;

// ── Store ─────────────────────────────────────────────────────────────────

/// Thread-safe in-memory store shared across all request handlers.
#[derive(Clone)]
pub struct WebhookStore {
    inner: Arc<RwLock<WebhookStoreInner>>,
}

struct WebhookStoreInner {
    webhooks: HashMap<String, WebhookRegistration>,
    logs: Vec<WebhookDeliveryLog>,
}

impl WebhookStore {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(WebhookStoreInner {
                webhooks: HashMap::new(),
                logs: Vec::new(),
            })),
        }
    }

    pub fn insert(&self, reg: WebhookRegistration) {
        let mut store = self.inner.write().unwrap();
        store.webhooks.insert(reg.id.clone(), reg);
    }

    pub fn get(&self, id: &str) -> Option<WebhookRegistration> {
        self.inner.read().unwrap().webhooks.get(id).cloned()
    }

    pub fn delete(&self, id: &str) -> bool {
        self.inner.write().unwrap().webhooks.remove(id).is_some()
    }

    pub fn list_for_device(&self, device_id: &str) -> Vec<WebhookRegistration> {
        self.inner
            .read()
            .unwrap()
            .webhooks
            .values()
            .filter(|w| w.device_id == device_id)
            .cloned()
            .collect()
    }

    pub fn matching_webhooks(&self, device_id: &str, event: &WebhookEventType) -> Vec<WebhookRegistration> {
        self.inner
            .read()
            .unwrap()
            .webhooks
            .values()
            .filter(|w| w.device_id == device_id && w.matches(event))
            .cloned()
            .collect()
    }

    pub fn append_log(&self, log: WebhookDeliveryLog) {
        self.inner.write().unwrap().logs.push(log);
    }

    pub fn logs_for_webhook(&self, webhook_id: &str) -> Vec<WebhookDeliveryLog> {
        self.inner
            .read()
            .unwrap()
            .logs
            .iter()
            .filter(|l| l.webhook_id == webhook_id)
            .cloned()
            .collect()
    }

    pub fn logs_for_device(&self, device_id: &str) -> Vec<WebhookDeliveryLog> {
        let store = self.inner.read().unwrap();
        let device_webhook_ids: std::collections::HashSet<String> = store
            .webhooks
            .values()
            .filter(|w| w.device_id == device_id)
            .map(|w| w.id.clone())
            .collect();
        store
            .logs
            .iter()
            .filter(|l| device_webhook_ids.contains(&l.webhook_id))
            .cloned()
            .collect()
    }
}

impl Default for WebhookStore {
    fn default() -> Self {
        Self::new()
    }
}

// ── HMAC signing ──────────────────────────────────────────────────────────

/// Computes `HMAC-SHA256(secret, body)` and returns a lowercase hex string.
/// This value is sent in the `X-Webhook-Signature` request header so that
/// the receiver can verify the payload has not been tampered with.
pub fn compute_signature(secret: &str, body: &[u8]) -> String {
    type HmacSha256 = Hmac<Sha256>;
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .expect("HMAC can accept keys of any length");
    mac.update(body);
    let result = mac.finalize();
    hex::encode(result.into_bytes())
}

// ── Delivery with retry ───────────────────────────────────────────────────

const MAX_ATTEMPTS: u32 = 5;
const BASE_DELAY_MS: u64 = 1_000; // 1 s → 2 s → 4 s → 8 s → 16 s

/// Delivers `payload` to every webhook registered for `device_id` that
/// subscribes to `event_type`. Runs all deliveries concurrently; each
/// individual webhook delivery retries up to `MAX_ATTEMPTS` times with
/// exponential backoff. Delivery logs are written to `store` after every
/// attempt regardless of outcome.
pub async fn dispatch_event(
    store: WebhookStore,
    device_id: String,
    event_type: WebhookEventType,
    data: serde_json::Value,
) {
    let webhooks = store.matching_webhooks(&device_id, &event_type);
    if webhooks.is_empty() {
        return;
    }

    let payload = WebhookPayload::new(event_type, device_id, data);
    let client = Client::new();

    let tasks: Vec<_> = webhooks
        .into_iter()
        .map(|webhook| {
            let store = store.clone();
            let payload = payload.clone();
            let client = client.clone();
            tokio::spawn(async move {
                deliver_with_retry(&client, &store, &webhook, &payload).await;
            })
        })
        .collect();

    for task in tasks {
        let _ = task.await;
    }
}

/// Attempts delivery of `payload` to `webhook.url` with up to `MAX_ATTEMPTS`
/// retries. Writes a `WebhookDeliveryLog` row after every attempt.
async fn deliver_with_retry(
    client: &Client,
    store: &WebhookStore,
    webhook: &WebhookRegistration,
    payload: &WebhookPayload,
) {
    let body_bytes = match serde_json::to_vec(payload) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("[webhook] Failed to serialise payload: {}", e);
            return;
        }
    };
    let signature = compute_signature(&webhook.secret, &body_bytes);

    for attempt in 1..=MAX_ATTEMPTS {
        let result = client
            .post(&webhook.url)
            .header("Content-Type", "application/json")
            .header("X-Webhook-Signature", format!("sha256={}", signature))
            .header("X-Delivery-Id", &payload.delivery_id)
            .header("X-Event", payload.event.to_string())
            .body(body_bytes.clone())
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await;

        let log = match result {
            Ok(resp) => {
                let http_status = resp.status().as_u16();
                let success = resp.status().is_success();
                let body_preview = resp
                    .text()
                    .await
                    .unwrap_or_default()
                    .chars()
                    .take(500)
                    .collect::<String>();

                WebhookDeliveryLog {
                    id: Uuid::new_v4().to_string(),
                    webhook_id: webhook.id.clone(),
                    delivery_id: payload.delivery_id.clone(),
                    event: payload.event.clone(),
                    attempt,
                    status: if success {
                        DeliveryStatus::Success
                    } else {
                        DeliveryStatus::Failed
                    },
                    http_status: Some(http_status),
                    response_body: Some(body_preview),
                    error_message: None,
                    attempted_at: Utc::now(),
                }
            }
            Err(e) => WebhookDeliveryLog {
                id: Uuid::new_v4().to_string(),
                webhook_id: webhook.id.clone(),
                delivery_id: payload.delivery_id.clone(),
                event: payload.event.clone(),
                attempt,
                status: DeliveryStatus::Error,
                http_status: None,
                response_body: None,
                error_message: Some(e.to_string()),
                attempted_at: Utc::now(),
            },
        };

        let succeeded = log.status == DeliveryStatus::Success;
        store.append_log(log);

        if succeeded {
            return;
        }

        if attempt < MAX_ATTEMPTS {
            let delay_ms = BASE_DELAY_MS * (1u64 << (attempt - 1));
            sleep(Duration::from_millis(delay_ms)).await;
        }
    }
}