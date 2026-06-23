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

// ─── Device Heartbeat ────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct HeartbeatRequest {
    pub health_metrics: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DeviceStatus {
    pub device_id: String,
    pub online: bool,
    pub last_seen: Option<DateTime<Utc>>,
    pub missed_heartbeats: u32,
    pub health_metrics: Option<serde_json::Value>,
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
    pub device_name: String,
    pub user_address: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub active: bool,
}

impl Session {
    pub fn new(device_id: String, device_name: String, user_address: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            device_id,
            device_name,
            user_address,
            created_at: now,
            expires_at: now + Duration::hours(1),
            active: true,
        }
    }
}

// ─── Telemetry Data ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct TelemetryData {
    pub timestamp: String,
    pub numeric_readings: std::collections::HashMap<String, f64>,
    pub boolean_readings: std::collections::HashMap<String, bool>,
    pub string_readings: std::collections::HashMap<String, String>,
    pub is_abnormal: bool,
}

// ─── Analytics ───────────────────────────────────────────────────────────────

/// Granularity for revenue / session time-series.
#[derive(Debug, Clone, Deserialize, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ReportPeriod {
    #[default]
    Daily,
    Weekly,
    Monthly,
}

/// Query parameters for `GET /devices/:id/analytics`.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct AnalyticsQuery {
    /// Time-series granularity: daily | weekly | monthly  (default: daily)
    #[serde(default)]
    pub period: ReportPeriod,
    /// How many periods to look back (default: 30 for daily, 12 for weekly/monthly).
    pub lookback: Option<usize>,
    /// Export format: json | csv  (default: json)
    pub format: Option<String>,
}

/// One data point in a revenue or session time-series.
#[derive(Debug, Clone, Serialize)]
pub struct TimeSeriesPoint {
    /// ISO-8601 date label for the period start (YYYY-MM-DD).
    pub date: String,
    pub revenue: f64,
    pub session_count: u64,
    pub unique_users: u64,
}

/// Aggregated peak-usage hour row.
#[derive(Debug, Clone, Serialize)]
pub struct PeakHour {
    /// Hour of the day in UTC (0–23).
    pub hour: u8,
    pub session_count: u64,
}

/// Retention cohort: for users who first accessed the device N periods ago,
/// how many returned in subsequent periods?
#[derive(Debug, Clone, Serialize)]
pub struct RetentionRow {
    /// Cohort label (e.g., "2025-01-06").
    pub cohort: String,
    /// Number of users who first accessed the device in this cohort.
    pub new_users: u64,
    /// Number who came back at least once after the cohort period.
    pub returning_users: u64,
    pub retention_rate: f64,
}

/// Full analytics report for a single device.
#[derive(Debug, Serialize)]
pub struct DeviceAnalyticsReport {
    pub device_id: String,
    pub period: String,
    /// Total revenue over the lookback window (XLM).
    pub total_revenue: f64,
    /// Total sessions in the lookback window.
    pub total_sessions: u64,
    /// Total unique users in the lookback window.
    pub total_unique_users: u64,
    /// Average session duration in seconds.
    pub avg_session_duration_secs: f64,
    /// Revenue / sessions / unique-users time-series.
    pub time_series: Vec<TimeSeriesPoint>,
    /// Top-5 peak usage hours (UTC).
    pub peak_hours: Vec<PeakHour>,
    /// Simple two-period retention cohort analysis.
    pub retention: Vec<RetentionRow>,
}
