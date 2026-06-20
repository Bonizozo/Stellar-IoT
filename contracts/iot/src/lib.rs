#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, token, Address, Env, Symbol,
};

const FEE_DENOMINATOR: i128 = 10_000;

#[contracttype]
#[derive(Clone)]
pub struct DevicePrice {
    pub device_id: Symbol,
    pub price: i128,
}

#[contracttype]
pub enum DataKey {
    Admin,
    PlatformFeeBps,
    PlatformFeeBalance(Address),
    DevicePrice(Symbol),
    DeviceOwner(Symbol),
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

    /// Process payment for device access, sending net revenue to the owner and retaining platform fees.
    pub fn request_access(
        env: Env,
        device_id: Symbol,
        user: Address,
        token: Address,
        amount: i128,
    ) -> bool {
        user.require_auth();

        let price = Self::get_device_price(env.clone(), device_id.clone());
        if price <= 0 || amount < price {
            return false;
        }

        let owner = Self::get_device_owner(env.clone(), device_id.clone())
            .unwrap_or_else(|| panic!("device owner not found"));
        let fee_bps = Self::get_platform_fee(env.clone());
        let platform_fee = amount * fee_bps / FEE_DENOMINATOR;
        let owner_amount = amount - platform_fee;
        let contract_address = env.current_contract_address();
        let token_client = token::Client::new(&env, &token);

        if owner_amount > 0 {
            token_client.transfer(&user, &owner, &owner_amount);
        }
        if platform_fee > 0 {
            token_client.transfer(&user, &contract_address, &platform_fee);
            let fee_key = DataKey::PlatformFeeBalance(token.clone());
            let current_fee_balance = Self::get_platform_fee_balance(env.clone(), token.clone());
            env.storage()
                .instance()
                .set(&fee_key, &(current_fee_balance + platform_fee));
        }

        env.events().publish(
            (symbol_short!("payment"), device_id.clone()),
            (
                user.clone(),
                owner.clone(),
                amount,
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

