use crate::analytics;
use crate::models::{
    AnalyticsQuery, Device, DeviceSearchQuery, DeviceSearchResponse,
    PaymentRequest, PaymentResponse, Session, HeartbeatRequest,
};
use crate::services;
use axum::{
    extract::{Path, Query, ws::{WebSocketUpgrade, WebSocket, Message}},
    http::StatusCode,
    response::IntoResponse,
    extract::{Path, Query},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;

/// Get all available devices (unchanged — keeps backwards compatibility).
pub async fn get_devices() -> Json<Vec<Device>> {
    Json(services::get_mock_devices())
}

/// Search and filter devices.
pub async fn search_devices(Query(query): Query<DeviceSearchQuery>) -> Json<DeviceSearchResponse> {
    Json(services::search_devices(&query))
}

/// Process payment and grant access with Stellar verification.
pub async fn process_payment(
    Json(payment): Json<PaymentRequest>,
) -> Result<Json<PaymentResponse>, StatusCode> {
    // 1. Verify payment on Stellar
    match services::verify_payment(&payment.tx_hash, &payment.device_id, &payment.user_address)
        .await
    {
        Ok(true) => {
            // Payment verified - grant access and store session in global store
            let session = services::create_session(payment.device_id, payment.user_address);
            Ok(Json(PaymentResponse {
                access_granted: true,
                session_id: session.id,
                expires_at: session.expires_at.to_rfc3339(),
            }))
        }
        Ok(false) => {
            // Replay attack detected
            Err(StatusCode::CONFLICT)
        }
        Err(msg) => {
            // Verification failed
            eprintln!("Payment verification failed: {}", msg);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

/// Get session details.
pub async fn get_session(Path(id): Path<String>) -> Result<Json<Session>, StatusCode> {
    match services::get_session(&id) {
        Some(session) => Ok(Json(session)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

#[derive(Debug, Deserialize)]
pub struct SessionsQuery {
    pub user: String,
}

/// Get sessions by user.
pub async fn get_sessions(Query(query): Query<SessionsQuery>) -> Json<Vec<Session>> {
    Json(services::get_sessions_by_user(&query.user))
}

/// Extend session.
pub async fn extend_session(
    Path(id): Path<String>,
    Json(payment): Json<PaymentRequest>,
) -> Result<Json<PaymentResponse>, StatusCode> {
    match services::verify_payment(&payment.tx_hash, &payment.device_id, &payment.user_address)
        .await
    {
        Ok(true) => {
            match services::extend_session(&id, 1) {
                Ok(session) => Ok(Json(PaymentResponse {
                    access_granted: true,
                    session_id: session.id,
                    expires_at: session.expires_at.to_rfc3339(),
                })),
                Err(_) => Err(StatusCode::NOT_FOUND),
            }
        }
        Ok(false) => Err(StatusCode::CONFLICT),
        Err(_) => Err(StatusCode::BAD_REQUEST),
    }
}

/// End session.
pub async fn end_session(Path(id): Path<String>) -> Result<StatusCode, StatusCode> {
    match services::end_session(&id) {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

/// Upgrade HTTP GET request to WebSocket.
pub async fn telemetry_ws(
    ws: WebSocketUpgrade,
    Path(id): Path<String>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_telemetry_socket(socket, id))
}

/// Send live telemetry data packets to the client.
async fn handle_telemetry_socket(mut socket: WebSocket, session_id: String) {
    // Verify session exists
    let session = match services::get_session(&session_id) {
        Some(s) => s,
        None => {
            let _ = socket.send(Message::Text("Session not found".to_string())).await;
            return;
        }
    };

    // Find the device category for simulation behavior
    let device_category = services::get_mock_devices()
        .into_iter()
        .find(|d| d.id == session.device_id)
        .map(|d| d.category)
        .unwrap_or(crate::models::DeviceCategory::Climate);

    let mut interval = tokio::time::interval(std::time::Duration::from_millis(1500));
    let mut ticks = 0;

    loop {
        tokio::select! {
            _ = interval.tick() => {
                // Check if session remains active and not expired
                let current_session = match services::get_session(&session_id) {
                    Some(s) => s,
                    None => break,
                };

                let now = chrono::Utc::now();
                if !current_session.active || current_session.expires_at < now {
                    let _ = socket.send(Message::Text(serde_json::json!({
                        "error": "Session expired or terminated",
                        "active": false
                    }).to_string())).await;
                    break;
                }

                ticks += 1;
                let data = services::generate_telemetry_data(&device_category, ticks);
                if let Ok(msg_text) = serde_json::to_string(&data) {
                    if socket.send(Message::Text(msg_text)).await.is_err() {
                        break; // Connection dropped
                    }
                } else {
                    break;
                }
            }
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Close(_))) | None => break,
                    _ => {}
                }
            }
        }
    }
}

/// `GET /devices/:id/analytics`
///
/// Query params:
/// - `period`   – daily | weekly | monthly  (default: daily)
/// - `lookback` – number of periods to include (default: 30/12/12)
/// - `format`   – json | csv  (default: json)
///
/// Returns a full analytics report for the device owner.
/// Use `format=csv` for an exportable spreadsheet.
pub async fn get_device_analytics(
    Path(id): Path<String>,
    Query(query): Query<AnalyticsQuery>,
) -> Response {
    let report = match analytics::generate_report(&id, &query) {
        Some(r) => r,
        None => {
            return (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Device not found" })))
                .into_response();
        }
    };

    let want_csv = query
        .format
        .as_deref()
        .map(|f| f.eq_ignore_ascii_case("csv"))
        .unwrap_or(false);

    if want_csv {
        match analytics::report_to_csv(&report) {
            Ok(csv) => (
                StatusCode::OK,
                [
                    (
                        header::CONTENT_TYPE,
                        "text/csv; charset=utf-8",
                    ),
                    (
                        header::CONTENT_DISPOSITION,
                        "attachment; filename=\"analytics.csv\"",
                    ),
                ],
                csv,
            )
                .into_response(),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e })),
            )
                .into_response(),
        }
    } else {
        Json(report).into_response()
    }
}

/// Process device heartbeat.
pub async fn device_heartbeat(
    Path(id): Path<String>,
    Json(payload): Json<HeartbeatRequest>,
) -> Result<StatusCode, StatusCode> {
    match services::record_heartbeat(&id, payload.health_metrics) {
        Ok(_) => Ok(StatusCode::OK),
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}
