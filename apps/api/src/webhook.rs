//! Webhook data models.
//!
//! A *webhook* is a URL registered by a device owner to receive
//! real-time HTTP POST notifications when key device events occur.
//! Every delivery attempt is logged so owners can diagnose failures.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── Event types ──────────────────────────────────────────────────────────

/// The set of device events that can trigger a webhook delivery.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum WebhookEventType {
    /// A Stellar payment was received for the device.
    PaymentReceived,
    /// A session was created and access has been granted to the device.
    AccessGranted,
    /// A session has expired and access is no longer valid.
    AccessExpired,
    /// The device has not sent a heartbeat within the expected window.
    DeviceOffline,
}

impl std::fmt::Display for WebhookEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::PaymentReceived => "payment_received",
            Self::AccessGranted => "access_granted",
            Self::AccessExpired => "access_expired",
            Self::DeviceOffline => "device_offline",
        };
        write!(f, "{}", s)
    }
}

// ── Registration ─────────────────────────────────────────────────────────

/// Request body for `POST /webhooks`.
#[derive(Debug, Deserialize)]
pub struct RegisterWebhookRequest {
    /// Device ID this webhook is scoped to.
    pub device_id: String,
    /// HTTPS URL that will receive POST deliveries.
    pub url: String,
    /// Caller-supplied secret used to generate the HMAC-SHA256 signature
    /// included in every delivery (`X-Webhook-Signature` header).
    pub secret: String,
    /// Subset of event types to deliver. If empty, all events are delivered.
    #[serde(default)]
    pub events: Vec<WebhookEventType>,
}

/// Persisted webhook record returned by the API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookRegistration {
    pub id: String,
    pub device_id: String,
    pub url: String,
    /// Secret is stored but never returned in API responses (see `WebhookView`).
    #[serde(skip_serializing)]
    pub secret: String,
    /// Empty vec means "all events".
    pub events: Vec<WebhookEventType>,
    pub created_at: DateTime<Utc>,
    pub active: bool,
}

impl WebhookRegistration {
    pub fn new(req: RegisterWebhookRequest) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            device_id: req.device_id,
            url: req.url,
            secret: req.secret,
            events: req.events,
            created_at: Utc::now(),
            active: true,
        }
    }

    /// Returns true if this webhook should fire for the given event type.
    pub fn matches(&self, event_type: &WebhookEventType) -> bool {
        self.active && (self.events.is_empty() || self.events.contains(event_type))
    }
}

/// Public-facing view of a `WebhookRegistration` (secret omitted).
#[derive(Debug, Serialize)]
pub struct WebhookView {
    pub id: String,
    pub device_id: String,
    pub url: String,
    pub events: Vec<WebhookEventType>,
    pub created_at: DateTime<Utc>,
    pub active: bool,
}

impl From<&WebhookRegistration> for WebhookView {
    fn from(r: &WebhookRegistration) -> Self {
        Self {
            id: r.id.clone(),
            device_id: r.device_id.clone(),
            url: r.url.clone(),
            events: r.events.clone(),
            created_at: r.created_at,
            active: r.active,
        }
    }
}

// ── Event payload ─────────────────────────────────────────────────────────

/// The JSON body POSTed to the webhook URL on every delivery attempt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookPayload {
    /// Unique ID for this specific delivery (stable across retries).
    pub delivery_id: String,
    pub event: WebhookEventType,
    pub device_id: String,
    pub timestamp: DateTime<Utc>,
    /// Arbitrary event-specific data.
    pub data: serde_json::Value,
}

impl WebhookPayload {
    pub fn new(event: WebhookEventType, device_id: String, data: serde_json::Value) -> Self {
        Self {
            delivery_id: Uuid::new_v4().to_string(),
            event,
            device_id,
            timestamp: Utc::now(),
            data,
        }
    }
}

// ── Delivery log ──────────────────────────────────────────────────────────

/// Outcome of a single HTTP delivery attempt.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DeliveryStatus {
    /// Remote endpoint returned a 2xx status code.
    Success,
    /// Remote endpoint returned a non-2xx status code.
    Failed,
    /// The HTTP request itself could not be completed (timeout, DNS, etc.).
    Error,
}

/// One row in the delivery log — one row per attempt, not per event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookDeliveryLog {
    pub id: String,
    pub webhook_id: String,
    pub delivery_id: String,
    pub event: WebhookEventType,
    pub attempt: u32,
    pub status: DeliveryStatus,
    /// HTTP status code returned by the remote endpoint, if we got one.
    pub http_status: Option<u16>,
    /// First 500 chars of the response body (for debugging).
    pub response_body: Option<String>,
    /// Error message if `status == Error`.
    pub error_message: Option<String>,
    pub attempted_at: DateTime<Utc>,
}
