use axum::{extract::Query, http::StatusCode, Json};
use chrono::Utc;
use ed25519_dalek::{Signature, VerifyingKey};
use jsonwebtoken::{encode, EncodingKey, Header};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    env,
    sync::Mutex,
};

// ─── Challenge store ──────────────────────────────────────────────────────────

struct ChallengeEntry {
    nonce: String,
    expires_at: i64, // unix seconds
}

lazy_static::lazy_static! {
    static ref CHALLENGES: Mutex<HashMap<String, ChallengeEntry>> = Mutex::new(HashMap::new());
}

const CHALLENGE_TTL_SECS: i64 = 60;

// ─── JWT claims ───────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // wallet address
    pub exp: usize,
}

fn jwt_secret() -> String {
    env::var("JWT_SECRET").expect("JWT_SECRET must be set")
}

// ─── GET /auth/challenge?address=<wallet> ─────────────────────────────────────

#[derive(Deserialize)]
pub struct ChallengeQuery {
    pub address: String,
}

#[derive(Serialize)]
pub struct ChallengeResponse {
    pub nonce: String,
    pub expires_at: i64,
}

pub async fn get_challenge(
    Query(q): Query<ChallengeQuery>,
) -> Json<ChallengeResponse> {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    let nonce = hex::encode(bytes);
    let expires_at = Utc::now().timestamp() + CHALLENGE_TTL_SECS;

    CHALLENGES.lock().unwrap().insert(
        q.address,
        ChallengeEntry { nonce: nonce.clone(), expires_at },
    );

    Json(ChallengeResponse { nonce, expires_at })
}

// ─── POST /auth/verify ────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct VerifyRequest {
    /// Stellar wallet address (Ed25519 public key, base64url or hex, 32 bytes).
    pub address: String,
    /// Hex-encoded Ed25519 signature over the nonce bytes.
    pub signature: String,
}

#[derive(Serialize)]
pub struct TokenResponse {
    pub token: String,
}

pub async fn verify_challenge(
    Json(body): Json<VerifyRequest>,
) -> Result<Json<TokenResponse>, (StatusCode, &'static str)> {
    // 1. Retrieve and validate the challenge
    let nonce = {
        let mut store = CHALLENGES.lock().unwrap();
        let entry = store
            .remove(&body.address)
            .ok_or((StatusCode::NOT_FOUND, "No challenge found for this address"))?;

        if Utc::now().timestamp() > entry.expires_at {
            return Err((StatusCode::GONE, "Challenge expired"));
        }
        entry.nonce
    };

    // 2. Decode public key (32-byte hex or base64)
    let pk_bytes: [u8; 32] = hex::decode(&body.address)
        .or_else(|_| {
            use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
            URL_SAFE_NO_PAD.decode(&body.address)
        })
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid address encoding"))?
        .try_into()
        .map_err(|_| (StatusCode::BAD_REQUEST, "Address must be 32 bytes"))?;

    let verifying_key = VerifyingKey::from_bytes(&pk_bytes)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid public key"))?;

    // 3. Decode signature (64-byte hex)
    let sig_bytes: [u8; 64] = hex::decode(&body.signature)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid signature encoding"))?
        .try_into()
        .map_err(|_| (StatusCode::BAD_REQUEST, "Signature must be 64 bytes"))?;

    let signature = Signature::from_bytes(&sig_bytes);

    // 4. Verify: client must have signed the raw nonce bytes
    use ed25519_dalek::Verifier;
    verifying_key
        .verify(nonce.as_bytes(), &signature)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Signature verification failed"))?;

    // 5. Issue JWT (24h expiry)
    let exp = (Utc::now() + chrono::Duration::hours(24)).timestamp() as usize;
    let claims = Claims { sub: body.address, exp };
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret().as_bytes()),
    )
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Token generation failed"))?;

    Ok(Json(TokenResponse { token }))
}
