use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ─── Device ──────────────────────────────────────────────────────────────────

/// Known device categories used for filtering.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DeviceCategory {
    Security,
    Environmental,
    Climate,
    Utility,
    Access,
}

/// Core device record, now enriched with discovery metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub id: String,
    pub name: String,
    pub description: String,
    pub price: f64,
    pub available: bool,
    pub location: String,
    pub category: DeviceCategory,
    /// Average user rating, 0.0–5.0.
    pub rating: f64,
    /// Cumulative access count used as a popularity signal.
    pub popularity: u64,
    /// WGS-84 latitude for geospatial queries.
    pub latitude: f64,
    /// WGS-84 longitude for geospatial queries.
    pub longitude: f64,
}

// ─── Search / filter ─────────────────────────────────────────────────────────

/// Sort field for device search results.
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SortField {
    Price,
    Rating,
    Popularity,
}

/// Sort direction.
#[derive(Debug, Clone, Deserialize, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum SortOrder {
    #[default]
    Asc,
    Desc,
}

/// Query parameters accepted by `GET /devices/search`.
///
/// All fields are optional; omitting a field disables that filter.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct DeviceSearchQuery {
    // ── Full-text ──────────────────────────────────────────────────────────
    /// Case-insensitive substring match against name and description.
    pub q: Option<String>,

    // ── Filters ────────────────────────────────────────────────────────────
    pub category: Option<DeviceCategory>,
    /// When true return only available devices; false returns only unavailable.
    /// Omit to return all.
    pub available: Option<bool>,
    pub min_price: Option<f64>,
    pub max_price: Option<f64>,

    // ── Geospatial ─────────────────────────────────────────────────────────
    /// Centre-point latitude for proximity search.
    pub lat: Option<f64>,
    /// Centre-point longitude for proximity search.
    pub lng: Option<f64>,
    /// Maximum distance in kilometres from (lat, lng).
    pub radius_km: Option<f64>,

    // ── Sorting ────────────────────────────────────────────────────────────
    pub sort_by: Option<SortField>,
    #[serde(default)]
    pub sort_order: SortOrder,

    // ── Cursor pagination ──────────────────────────────────────────────────
    /// Maximum number of results to return (1–100, default 20).
    pub limit: Option<usize>,
    /// Opaque cursor returned by the previous page (the last device id).
    pub cursor: Option<String>,
}

/// A single page of search results.
#[derive(Debug, Serialize)]
pub struct DeviceSearchResponse {
    pub data: Vec<Device>,
    /// Total number of devices that match the query (before pagination).
    pub total: usize,
    /// Cursor to pass as `cursor=` on the next request; `null` when no more pages.
    pub next_cursor: Option<String>,
    pub limit: usize,
}

// ─── Payment / Session ───────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct PaymentRequest {
    pub device_id: String,
    pub user_address: String,
    #[allow(dead_code)]
    pub amount: f64,
    pub tx_hash: String, // Stellar transaction hash to verify
}

#[derive(Debug, Serialize)]
pub struct PaymentResponse {
    pub access_granted: bool,
    pub session_id: String,
    pub expires_at: String,
}

#[allow(dead_code)]
#[derive(Debug, Serialize)]
pub struct PaymentError {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Session {
    pub id: String,
    pub device_id: String,
    pub user_address: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub active: bool,
}

impl Session {
    pub fn new(device_id: String, user_address: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            device_id,
            user_address,
            created_at: now,
            expires_at: now + Duration::hours(1),
            active: true,
        }
    }
}
