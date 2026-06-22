use crate::config::StellarConfig;
use lazy_static::lazy_static;
use serde::Deserialize;
use std::collections::HashSet;
use std::sync::Mutex;

lazy_static! {
    static ref PROCESSED_TX_HASHES: Mutex<HashSet<String>> = Mutex::new(HashSet::new());
}

#[derive(Debug, Deserialize)]
struct HorizonOperation {
    #[serde(rename = "type")]
    operation_type: String,
    #[allow(dead_code)]
    source_account: String,
    #[serde(rename = "to")]
    destination_account: String,
    amount: String,
    #[serde(rename = "transaction_hash")]
    #[allow(dead_code)]
    tx_hash: String,
}

#[derive(Debug, Deserialize)]
struct HorizonTransaction {
    #[allow(dead_code)]
    id: String,
    successful: bool,
    #[allow(dead_code)]
    memo: Option<String>,
    operations: Vec<HorizonOperation>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct HorizonResponse {
    _embedded: Embedded,
}

#[derive(Debug, Deserialize)]
struct Embedded {
    #[allow(dead_code)]
    records: Vec<HorizonTransaction>,
}

pub struct StellarService {
    config: StellarConfig,
    client: reqwest::Client,
}

impl StellarService {
    pub fn new() -> Self {
        Self {
            config: StellarConfig::from_env(),
            client: reqwest::Client::new(),
        }
    }

    /// Verify a payment transaction on Stellar.
    ///
    /// Checks:
    /// 1. Transaction exists and is successful
    /// 2. Amount matches expected price
    /// 3. Destination matches device owner wallet
    /// 4. Prevents replay attacks via transaction hash deduplication
    pub async fn verify_payment(
        &self,
        tx_hash: &str,
        expected_amount: f64,
        expected_destination: &str,
    ) -> Result<bool, String> {
        // 1. Prevent replay attacks
        {
            let processed = PROCESSED_TX_HASHES
                .lock()
                .map_err(|_| "Lock poisoned".to_string())?;
            if processed.contains(tx_hash) {
                return Ok(false);
            }
        }

        // 2. Fetch transaction from Horizon
        let url = format!("{}/transactions/{}", self.config.horizon_url, tx_hash);
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Horizon request failed: {}", e))?;

        if response.status() == 404 {
            return Err("Transaction not found".to_string());
        }

        let tx: HorizonTransaction = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        // 3. Check transaction successful
        if !tx.successful {
            return Err("Transaction failed".to_string());
        }

        // 4. Find payment operation
        let payment_op = tx
            .operations
            .iter()
            .find(|op| op.operation_type == "payment")
            .ok_or_else(|| "No payment operation found".to_string())?;

        // 5. Validate amount
        let amount: f64 = payment_op
            .amount
            .parse()
            .map_err(|_| "Invalid amount format".to_string())?;
        if (amount - expected_amount).abs() > 0.0001 {
            return Err(format!(
                "Amount mismatch: expected {}, got {}",
                expected_amount, amount
            ));
        }

        // 6. Validate destination
        if payment_op.destination_account != expected_destination {
            return Err("Destination mismatch".to_string());
        }

        // 7. Mark as processed (replay protection)
        {
            let mut processed = PROCESSED_TX_HASHES
                .lock()
                .map_err(|_| "Lock poisoned".to_string())?;
            processed.insert(tx_hash.to_string());
        }

        Ok(true)
    }
}
