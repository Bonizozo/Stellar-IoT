use axum::{extract::Path, http::StatusCode, Json};
use crate::models::{Device, PaymentRequest, PaymentResponse, Session};
use crate::services;

/// Get all available devices
pub async fn get_devices() -> Json<Vec<Device>> {
    Json(services::get_mock_devices())
}

/// Process payment and grant access
pub async fn process_payment(
    Json(payment): Json<PaymentRequest>,
) -> Result<Json<PaymentResponse>, StatusCode> {
    // Validate payment
    let device = services::get_mock_devices()
        .into_iter()
        .find(|d| d.id == payment.device_id)
        .ok_or(StatusCode::NOT_FOUND)?;

    if payment.amount < device.price {
        return Err(StatusCode::BAD_REQUEST);
    }

    // TODO: Call Soroban smart contract to validate payment
    // For now, simulate successful payment
    let session = Session::new(payment.device_id, payment.user_address);

    Ok(Json(PaymentResponse {
        access_granted: true,
        session_id: session.id,
        expires_at: session.expires_at.to_rfc3339(),
    }))
}

/// Get session details
pub async fn get_session(Path(id): Path<String>) -> Result<Json<Session>, StatusCode> {
    // TODO: Implement session storage
    // For now, return mock session
    let _ = id;
    Err(StatusCode::NOT_IMPLEMENTED)
}
