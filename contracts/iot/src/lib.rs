#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, token, Address, Env, Symbol, Vec,
};

const FEE_DENOMINATOR: i128 = 10_000;
/// Credit TTL: 30 days in ledgers (~5 s/ledger). In tests use a short TTL.
#[cfg(not(test))]
const CREDIT_TTL_LEDGERS: u32 = 518_400;
#[cfg(test)]
const CREDIT_TTL_LEDGERS: u32 = 100;

/// Timelock before unpause takes effect (~1 hour at 5 s/ledger). Short in tests.
#[cfg(not(test))]
const UNPAUSE_DELAY_LEDGERS: u32 = 720;
#[cfg(test)]
const UNPAUSE_DELAY_LEDGERS: u32 = 5;

// ── Discount tiers (device count thresholds) ─────────────────────────────────
// ≥10 devices → 5 % off, ≥50 → 10 % off, ≥100 → 20 % off
const TIER2_MIN: usize = 10;
const TIER2_DISC_BPS: i128 = 500;
const TIER3_MIN: usize = 50;
const TIER3_DISC_BPS: i128 = 1_000;
const TIER4_MIN: usize = 100;
const TIER4_DISC_BPS: i128 = 2_000;

#[contracttype]
#[derive(Clone)]
pub struct DevicePrice {
    pub device_id: Symbol,
    pub price: i128,
}

/// Per-user, per-device credit record.
#[contracttype]
#[derive(Clone)]
pub struct Credit {
    pub amount: i128,
    pub expires_at: u32, // ledger sequence
}

#[contracttype]
pub enum DataKey {
    Admin,
    PlatformFeeBps,
    PlatformFeeBalance(Address),
    DevicePrice(Symbol),
    DeviceOwner(Symbol),
    /// Prepaid credits: (user, device_id) → Credit
    Credit(Address, Symbol),
    /// true when contract is paused
    Paused,
    /// ledger sequence at which unpause was scheduled
    UnpauseAt,
}

#[contract]
pub struct IotContract;

#[contractimpl]
impl IotContract {
    /// Initialize contract admin and platform fee in basis points.
    pub fn initialize(env: Env, admin: Address, platform_fee_bps: i128) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("already initialized");
        }
        admin.require_auth();
        Self::validate_platform_fee(platform_fee_bps);
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::PlatformFeeBps, &platform_fee_bps);
    }

    /// Update the platform fee. Only the contract admin can call this.
    pub fn set_platform_fee(env: Env, admin: Address, platform_fee_bps: i128) {
        Self::require_admin(env.clone(), admin);
        Self::validate_platform_fee(platform_fee_bps);
        env.storage()
            .instance()
            .set(&DataKey::PlatformFeeBps, &platform_fee_bps);
    }

    /// Get the configured platform fee in basis points.
    pub fn get_platform_fee(env: Env) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::PlatformFeeBps)
            .unwrap_or(0)
    }

    /// Get the accumulated platform fee balance for a token.
    pub fn get_platform_fee_balance(env: Env, token: Address) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::PlatformFeeBalance(token))
            .unwrap_or(0)
    }

    /// Initialize device with price and owner.
    pub fn init_device(env: Env, device_id: Symbol, price: i128, owner: Address) {
        if price <= 0 {
            panic!("price must be positive");
        }
        owner.require_auth();
        env.storage()
            .instance()
            .set(&DataKey::DevicePrice(device_id.clone()), &price);
        env.storage()
            .instance()
            .set(&DataKey::DeviceOwner(device_id), &owner);
    }

    /// Get device price.
    pub fn get_device_price(env: Env, device_id: Symbol) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::DevicePrice(device_id))
            .unwrap_or(0)
    }

    /// Get device owner.
    pub fn get_device_owner(env: Env, device_id: Symbol) -> Option<Address> {
        env.storage()
            .instance()
            .get(&DataKey::DeviceOwner(device_id))
    }

    // ── Emergency pause ──────────────────────────────────────────────────────

    /// Pause the contract immediately. Admin only. Emits a Paused event.
    pub fn pause(env: Env, admin: Address) {
        Self::require_admin(env.clone(), admin.clone());
        env.storage().instance().set(&DataKey::Paused, &true);
        // Clear any pending unpause schedule.
        env.storage().instance().remove(&DataKey::UnpauseAt);
        env.events()
            .publish((symbol_short!("paused"),), (admin, env.ledger().sequence()));
    }

    /// Schedule an unpause after UNPAUSE_DELAY_LEDGERS. Admin only.
    /// The contract stays paused until `execute_unpause` is called after the delay.
    pub fn unpause(env: Env, admin: Address) {
        Self::require_admin(env.clone(), admin.clone());
        if !Self::is_paused(env.clone()) {
            panic!("not paused");
        }
        let ready_at = env.ledger().sequence() + UNPAUSE_DELAY_LEDGERS;
        env.storage()
            .instance()
            .set(&DataKey::UnpauseAt, &ready_at);
        env.events().publish(
            (symbol_short!("upauseSch"),),
            (admin, ready_at),
        );
    }

    /// Execute the unpause after the timelock has elapsed.
    pub fn execute_unpause(env: Env, admin: Address) {
        Self::require_admin(env.clone(), admin.clone());
        let ready_at: u32 = env
            .storage()
            .instance()
            .get(&DataKey::UnpauseAt)
            .unwrap_or_else(|| panic!("unpause not scheduled"));
        if env.ledger().sequence() < ready_at {
            panic!("timelock not elapsed");
        }
        env.storage().instance().set(&DataKey::Paused, &false);
        env.storage().instance().remove(&DataKey::UnpauseAt);
        env.events()
            .publish((symbol_short!("unpaused"),), (admin, env.ledger().sequence()));
    }

    /// Returns true when the contract is paused.
    pub fn is_paused(env: Env) -> bool {
        env.storage()
            .instance()
            .get(&DataKey::Paused)
            .unwrap_or(false)
    }

    // ── Bulk access ──────────────────────────────────────────────────────────

    /// Purchase bulk access credits for multiple devices in one transaction.
    /// Volume discount applied to the total based on the number of devices.
    /// Each credit is stored per (user, device_id) and expires after CREDIT_TTL_LEDGERS.
    pub fn purchase_bulk_access(
        env: Env,
        user: Address,
        token: Address,
        device_ids: Vec<Symbol>,
        amounts: Vec<i128>,
    ) {
        user.require_auth();

        if Self::is_paused(env.clone()) {
            panic!("contract paused");
        }

        let n = device_ids.len() as usize;
        if n == 0 {
            panic!("empty device list");
        }
        if device_ids.len() != amounts.len() {
            panic!("device_ids and amounts length mismatch");
        }

        // Compute discount bps based on number of devices purchased.
        let discount_bps: i128 = if n >= TIER4_MIN {
            TIER4_DISC_BPS
        } else if n >= TIER3_MIN {
            TIER3_DISC_BPS
        } else if n >= TIER2_MIN {
            TIER2_DISC_BPS
        } else {
            0
        };

        let expires_at = env.ledger().sequence() + CREDIT_TTL_LEDGERS;
        let token_client = token::Client::new(&env, &token);
        let contract_address = env.current_contract_address();

        for i in 0..device_ids.len() {
            let device_id = device_ids.get(i).unwrap();
            let full_amount = amounts.get(i).unwrap();

            if full_amount <= 0 {
                panic!("amount must be positive");
            }

            let price = Self::get_device_price(env.clone(), device_id.clone());
            if price <= 0 {
                panic!("device not registered");
            }
            if full_amount < price {
                panic!("amount below device price");
            }

            // Apply volume discount: user pays discounted amount, credit stored = full_amount.
            let discount = full_amount * discount_bps / FEE_DENOMINATOR;
            let charge = full_amount - discount;

            // Transfer charge from user to contract (held as credit).
            token_client.transfer(&user, &contract_address, &charge);

            // Accumulate with any existing unexpired credit.
            let key = DataKey::Credit(user.clone(), device_id.clone());
            let existing: Credit = env
                .storage()
                .persistent()
                .get(&key)
                .unwrap_or(Credit { amount: 0, expires_at: 0 });

            let carry = if existing.expires_at >= env.ledger().sequence() {
                existing.amount
            } else {
                0
            };

            env.storage().persistent().set(
                &key,
                &Credit {
                    amount: carry + full_amount, // credit at face value
                    expires_at,
                },
            );
        }

        env.events().publish(
            (symbol_short!("bulk"), user.clone()),
            (n as u32, discount_bps),
        );
    }

    /// Return unexpired credit amount for (user, device_id). Returns 0 if expired.
    pub fn get_credit(env: Env, user: Address, device_id: Symbol) -> i128 {
        let key = DataKey::Credit(user, device_id);
        let credit: Credit = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or(Credit { amount: 0, expires_at: 0 });
        if credit.expires_at >= env.ledger().sequence() {
            credit.amount
        } else {
            0
        }
    }

    /// Process payment for device access.
    /// If the user has an unexpired credit ≥ device price, the credit is consumed
    /// and the stored funds are forwarded to the owner (minus platform fee);
    /// otherwise the normal on-the-fly transfer path is used.
    pub fn request_access(
        env: Env,
        device_id: Symbol,
        user: Address,
        token: Address,
        amount: i128,
    ) -> bool {
        user.require_auth();

        if Self::is_paused(env.clone()) {
            panic!("contract paused");
        }

        let price = Self::get_device_price(env.clone(), device_id.clone());
        if price <= 0 {
            return false;
        }

        let owner = Self::get_device_owner(env.clone(), device_id.clone())
            .unwrap_or_else(|| panic!("device owner not found"));
        let fee_bps = Self::get_platform_fee(env.clone());
        let contract_address = env.current_contract_address();
        let token_client = token::Client::new(&env, &token);

        // Try to redeem credit first.
        let credit_key = DataKey::Credit(user.clone(), device_id.clone());
        let credit: Credit = env
            .storage()
            .persistent()
            .get(&credit_key)
            .unwrap_or(Credit { amount: 0, expires_at: 0 });

        let use_credit = credit.expires_at >= env.ledger().sequence() && credit.amount >= price;

        let (pay_amount, from_credit) = if use_credit {
            (price, true)
        } else {
            if amount < price {
                return false;
            }
            (amount, false)
        };

        let platform_fee = pay_amount * fee_bps / FEE_DENOMINATOR;
        let owner_amount = pay_amount - platform_fee;

        if from_credit {
            // Funds are already held by the contract; forward them.
            if owner_amount > 0 {
                token_client.transfer(&contract_address, &owner, &owner_amount);
            }
            if platform_fee > 0 {
                let fee_key = DataKey::PlatformFeeBalance(token.clone());
                let bal = Self::get_platform_fee_balance(env.clone(), token.clone());
                env.storage()
                    .instance()
                    .set(&fee_key, &(bal + platform_fee));
            }
            // Consume credit.
            let remaining = credit.amount - price;
            if remaining > 0 {
                env.storage().persistent().set(
                    &credit_key,
                    &Credit {
                        amount: remaining,
                        expires_at: credit.expires_at,
                    },
                );
            } else {
                env.storage().persistent().remove(&credit_key);
            }
        } else {
            // Normal path: transfer from user.
            if owner_amount > 0 {
                token_client.transfer(&user, &owner, &owner_amount);
            }
            if platform_fee > 0 {
                token_client.transfer(&user, &contract_address, &platform_fee);
                let fee_key = DataKey::PlatformFeeBalance(token.clone());
                let bal = Self::get_platform_fee_balance(env.clone(), token.clone());
                env.storage()
                    .instance()
                    .set(&fee_key, &(bal + platform_fee));
            }
        }

        env.events().publish(
            (symbol_short!("payment"), device_id.clone()),
            (
                user.clone(),
                owner.clone(),
                pay_amount,
                owner_amount,
                platform_fee,
            ),
        );
        env.events()
            .publish((symbol_short!("access"), device_id), user);

        true
    }

    /// Backwards-compatible alias for request_access.
    pub fn pay(env: Env, device_id: Symbol, user: Address, token: Address, amount: i128) -> bool {
        Self::request_access(env, device_id, user, token, amount)
    }

    /// Withdraw accumulated platform fees. Only the contract admin can call this.
    pub fn withdraw_platform_fees(
        env: Env,
        admin: Address,
        token: Address,
        to: Address,
        amount: i128,
    ) {
        Self::require_admin(env.clone(), admin);
        if amount <= 0 {
            panic!("withdraw amount must be positive");
        }
        let fee_key = DataKey::PlatformFeeBalance(token.clone());
        let current_fee_balance = Self::get_platform_fee_balance(env.clone(), token.clone());
        if amount > current_fee_balance {
            panic!("insufficient platform fees");
        }
        env.storage()
            .instance()
            .set(&fee_key, &(current_fee_balance - amount));
        token::Client::new(&env, &token).transfer(&env.current_contract_address(), &to, &amount);
    }

    fn require_admin(env: Env, admin: Address) {
        admin.require_auth();
        let stored_admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .unwrap_or_else(|| panic!("contract not initialized"));
        if admin != stored_admin {
            panic!("admin required");
        }
    }

    fn validate_platform_fee(platform_fee_bps: i128) {
        if platform_fee_bps < 0 || platform_fee_bps > FEE_DENOMINATOR {
            panic!("invalid platform fee");
        }
    }
}

#[cfg(test)]
mod test;
