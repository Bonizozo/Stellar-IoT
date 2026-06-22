use dotenv::dotenv;
use std::env;

#[derive(Clone)]
pub struct StellarConfig {
    pub horizon_url: String,
    #[allow(dead_code)]
    pub network_passphrase: String,
}

impl StellarConfig {
    pub fn from_env() -> Self {
        dotenv().ok();
        let horizon_url = env::var("HORIZON_URL")
            .unwrap_or_else(|_| "https://horizon-testnet.stellar.org".to_string());
        let network_passphrase = env::var("STELLAR_NETWORK_PASSPHRASE")
            .unwrap_or_else(|_| "Test SDF Network ; September 2015".to_string());

        Self {
            horizon_url,
            network_passphrase,
        }
    }
}
