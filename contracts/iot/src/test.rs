#![cfg(test)]

use super::*;
use soroban_sdk::{symbol_short, testutils::Address as _, token, Address, Env};

// ── helpers ──────────────────────────────────────────────────────────────────

/// Returns (env, contract_client, token_id, asset_client, admin).
/// Caller must keep `env` alive and re-derive clients from it when needed.
fn setup_with_fee(fee_bps: i128) -> (Env, Address, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, IotContract);
    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_id = env
        .register_stellar_asset_contract_v2(token_admin)
        .address();

    IotContractClient::new(&env, &contract_id).initialize(&admin, &fee_bps);

    (env, contract_id, token_id, admin)
}

fn setup() -> (Env, Address, Address, Address) {
    setup_with_fee(500) // 5 %
}

// ── initialization ────────────────────────────────────────────────────────────

#[test]
fn test_initialize_stores_fee() {
    let (env, cid, _, _) = setup();
    assert_eq!(IotContractClient::new(&env, &cid).get_platform_fee(), 500);
}

#[test]
#[should_panic(expected = "already initialized")]
fn test_initialize_twice_panics() {
    let (env, cid, _, admin) = setup();
    IotContractClient::new(&env, &cid).initialize(&admin, &100);
}

#[test]
#[should_panic(expected = "invalid platform fee")]
fn test_initialize_negative_fee_panics() {
    let env = Env::default();
    env.mock_all_auths();
    let cid = env.register_contract(None, IotContract);
    let admin = Address::generate(&env);
    IotContractClient::new(&env, &cid).initialize(&admin, &-1);
}

#[test]
#[should_panic(expected = "invalid platform fee")]
fn test_initialize_fee_exceeds_denominator_panics() {
    let env = Env::default();
    env.mock_all_auths();
    let cid = env.register_contract(None, IotContract);
    let admin = Address::generate(&env);
    IotContractClient::new(&env, &cid).initialize(&admin, &10_001);
}

// ── platform fee management ───────────────────────────────────────────────────

#[test]
fn test_set_platform_fee() {
    let (env, cid, _, admin) = setup();
    let c = IotContractClient::new(&env, &cid);
    c.set_platform_fee(&admin, &200);
    assert_eq!(c.get_platform_fee(), 200);
}

#[test]
#[should_panic(expected = "admin required")]
fn test_set_platform_fee_non_admin_panics() {
    let (env, cid, _, _) = setup();
    let impostor = Address::generate(&env);
    IotContractClient::new(&env, &cid).set_platform_fee(&impostor, &200);
}

#[test]
fn test_zero_fee_no_split() {
    let (env, cid, token_id, _) = setup_with_fee(0);
    let c = IotContractClient::new(&env, &cid);
    let asset = token::StellarAssetClient::new(&env, &token_id);
    let tok = token::Client::new(&env, &token_id);
    let owner = Address::generate(&env);
    let user = Address::generate(&env);
    let price = 1_000i128;

    c.init_device(&symbol_short!("d"), &price, &owner);
    asset.mint(&user, &price);
    c.request_access(&symbol_short!("d"), &user, &token_id, &price);

    assert_eq!(tok.balance(&owner), price);
    assert_eq!(tok.balance(&cid), 0);
    assert_eq!(c.get_platform_fee_balance(&token_id), 0);
}

// ── device registration ───────────────────────────────────────────────────────

#[test]
fn test_init_device_stores_price_and_owner() {
    let (env, cid, _, _) = setup();
    let c = IotContractClient::new(&env, &cid);
    let owner = Address::generate(&env);
    c.init_device(&symbol_short!("d1"), &2_000, &owner);
    assert_eq!(c.get_device_price(&symbol_short!("d1")), 2_000);
    assert_eq!(c.get_device_owner(&symbol_short!("d1")).unwrap(), owner);
}

#[test]
#[should_panic(expected = "price must be positive")]
fn test_init_device_zero_price_panics() {
    let (env, cid, _, _) = setup();
    IotContractClient::new(&env, &cid)
        .init_device(&symbol_short!("d1"), &0, &Address::generate(&env));
}

#[test]
#[should_panic(expected = "price must be positive")]
fn test_init_device_negative_price_panics() {
    let (env, cid, _, _) = setup();
    IotContractClient::new(&env, &cid)
        .init_device(&symbol_short!("d1"), &-1, &Address::generate(&env));
}

#[test]
fn test_init_device_re_registration_overwrites() {
    let (env, cid, _, _) = setup();
    let c = IotContractClient::new(&env, &cid);
    let owner2 = Address::generate(&env);
    c.init_device(&symbol_short!("d1"), &100, &Address::generate(&env));
    c.init_device(&symbol_short!("d1"), &999, &owner2);
    assert_eq!(c.get_device_price(&symbol_short!("d1")), 999);
    assert_eq!(c.get_device_owner(&symbol_short!("d1")).unwrap(), owner2);
}

#[test]
fn test_get_nonexistent_device_returns_defaults() {
    let (env, cid, _, _) = setup();
    let c = IotContractClient::new(&env, &cid);
    assert_eq!(c.get_device_price(&symbol_short!("nope")), 0);
    assert!(c.get_device_owner(&symbol_short!("nope")).is_none());
}

// ── payment and access flow ───────────────────────────────────────────────────

#[test]
fn test_request_access_splits_revenue_correctly() {
    // 5 % fee → owner gets 950, contract keeps 50
    let (env, cid, token_id, _) = setup();
    let c = IotContractClient::new(&env, &cid);
    let asset = token::StellarAssetClient::new(&env, &token_id);
    let tok = token::Client::new(&env, &token_id);
    let owner = Address::generate(&env);
    let user = Address::generate(&env);

    c.init_device(&symbol_short!("d1"), &1_000, &owner);
    asset.mint(&user, &1_000);

    assert!(c.request_access(&symbol_short!("d1"), &user, &token_id, &1_000));
    assert_eq!(tok.balance(&owner), 950);
    assert_eq!(tok.balance(&cid), 50);
    assert_eq!(c.get_platform_fee_balance(&token_id), 50);
    assert_eq!(tok.balance(&user), 0);
}

#[test]
fn test_pay_alias_is_equivalent() {
    let (env, cid, token_id, _) = setup();
    let c = IotContractClient::new(&env, &cid);
    let asset = token::StellarAssetClient::new(&env, &token_id);
    let tok = token::Client::new(&env, &token_id);
    let owner = Address::generate(&env);
    let user = Address::generate(&env);

    c.init_device(&symbol_short!("d1"), &500, &owner);
    asset.mint(&user, &500);

    assert!(c.pay(&symbol_short!("d1"), &user, &token_id, &500));
    assert_eq!(tok.balance(&owner), 475); // 500 - 5%
}

#[test]
fn test_overpayment_is_accepted() {
    // paying more than price is allowed; full amount is transferred
    let (env, cid, token_id, _) = setup_with_fee(0);
    let c = IotContractClient::new(&env, &cid);
    let asset = token::StellarAssetClient::new(&env, &token_id);
    let tok = token::Client::new(&env, &token_id);
    let owner = Address::generate(&env);
    let user = Address::generate(&env);

    c.init_device(&symbol_short!("d1"), &100, &owner);
    asset.mint(&user, &200);

    assert!(c.request_access(&symbol_short!("d1"), &user, &token_id, &200));
    assert_eq!(tok.balance(&owner), 200);
}

// ── edge case: insufficient amount ───────────────────────────────────────────

#[test]
fn test_insufficient_amount_returns_false_no_transfer() {
    let (env, cid, token_id, _) = setup();
    let c = IotContractClient::new(&env, &cid);
    let asset = token::StellarAssetClient::new(&env, &token_id);
    let tok = token::Client::new(&env, &token_id);
    let owner = Address::generate(&env);
    let user = Address::generate(&env);

    c.init_device(&symbol_short!("d1"), &1_000, &owner);
    asset.mint(&user, &999);

    assert!(!c.request_access(&symbol_short!("d1"), &user, &token_id, &999));
    assert_eq!(tok.balance(&user), 999); // no tokens moved
    assert_eq!(tok.balance(&owner), 0);
}

#[test]
fn test_zero_amount_returns_false() {
    let (env, cid, token_id, _) = setup();
    let c = IotContractClient::new(&env, &cid);
    c.init_device(&symbol_short!("d1"), &1_000, &Address::generate(&env));
    assert!(!c.request_access(
        &symbol_short!("d1"),
        &Address::generate(&env),
        &token_id,
        &0
    ));
}

// ── edge case: unregistered device ───────────────────────────────────────────

#[test]
fn test_request_access_unregistered_device_returns_false() {
    let (env, cid, token_id, _) = setup();
    let c = IotContractClient::new(&env, &cid);
    let user = Address::generate(&env);
    token::StellarAssetClient::new(&env, &token_id).mint(&user, &1_000);
    // price == 0 for unknown device → returns false without panic
    assert!(!c.request_access(&symbol_short!("nope"), &user, &token_id, &1_000));
}

// ── edge case: double access ──────────────────────────────────────────────────

#[test]
fn test_double_access_both_succeed() {
    // Contract does not prevent re-payment; each call is independent.
    let (env, cid, token_id, _) = setup_with_fee(0);
    let c = IotContractClient::new(&env, &cid);
    let asset = token::StellarAssetClient::new(&env, &token_id);
    let tok = token::Client::new(&env, &token_id);
    let owner = Address::generate(&env);
    let user = Address::generate(&env);

    c.init_device(&symbol_short!("d1"), &100, &owner);
    asset.mint(&user, &200);

    assert!(c.request_access(&symbol_short!("d1"), &user, &token_id, &100));
    assert!(c.request_access(&symbol_short!("d1"), &user, &token_id, &100));
    assert_eq!(tok.balance(&owner), 200);
    assert_eq!(tok.balance(&user), 0);
}

// ── platform fee accumulation ─────────────────────────────────────────────────

#[test]
fn test_platform_fee_accumulates_across_payments() {
    let (env, cid, token_id, _) = setup(); // 5 %
    let c = IotContractClient::new(&env, &cid);
    let asset = token::StellarAssetClient::new(&env, &token_id);
    let owner = Address::generate(&env);

    c.init_device(&symbol_short!("d1"), &1_000, &owner);
    for _ in 0..3 {
        let user = Address::generate(&env);
        asset.mint(&user, &1_000);
        c.request_access(&symbol_short!("d1"), &user, &token_id, &1_000);
    }

    assert_eq!(c.get_platform_fee_balance(&token_id), 150); // 50 × 3
}

// ── platform fee withdrawal ───────────────────────────────────────────────────

#[test]
fn test_admin_withdraw_full_balance() {
    let (env, cid, token_id, admin) = setup();
    let c = IotContractClient::new(&env, &cid);
    let asset = token::StellarAssetClient::new(&env, &token_id);
    let tok = token::Client::new(&env, &token_id);
    let owner = Address::generate(&env);
    let user = Address::generate(&env);
    let recipient = Address::generate(&env);

    c.init_device(&symbol_short!("d1"), &1_000, &owner);
    asset.mint(&user, &1_000);
    c.request_access(&symbol_short!("d1"), &user, &token_id, &1_000);

    c.withdraw_platform_fees(&admin, &token_id, &recipient, &50);
    assert_eq!(tok.balance(&recipient), 50);
    assert_eq!(c.get_platform_fee_balance(&token_id), 0);
}

#[test]
fn test_admin_withdraw_partial_balance() {
    let (env, cid, token_id, admin) = setup();
    let c = IotContractClient::new(&env, &cid);
    let asset = token::StellarAssetClient::new(&env, &token_id);
    let tok = token::Client::new(&env, &token_id);
    let owner = Address::generate(&env);
    let user = Address::generate(&env);
    let recipient = Address::generate(&env);

    c.init_device(&symbol_short!("d1"), &1_000, &owner);
    asset.mint(&user, &1_000);
    c.request_access(&symbol_short!("d1"), &user, &token_id, &1_000);

    c.withdraw_platform_fees(&admin, &token_id, &recipient, &20);
    assert_eq!(tok.balance(&recipient), 20);
    assert_eq!(c.get_platform_fee_balance(&token_id), 30);
}

#[test]
#[should_panic(expected = "insufficient platform fees")]
fn test_withdraw_more_than_balance_panics() {
    let (env, cid, token_id, admin) = setup();
    let recipient = Address::generate(&env);
    IotContractClient::new(&env, &cid)
        .withdraw_platform_fees(&admin, &token_id, &recipient, &1);
}

#[test]
#[should_panic(expected = "withdraw amount must be positive")]
fn test_withdraw_zero_panics() {
    let (env, cid, token_id, admin) = setup();
    let recipient = Address::generate(&env);
    IotContractClient::new(&env, &cid)
        .withdraw_platform_fees(&admin, &token_id, &recipient, &0);
}

#[test]
#[should_panic(expected = "admin required")]
fn test_withdraw_non_admin_panics() {
    let (env, cid, token_id, _) = setup();
    let impostor = Address::generate(&env);
    let recipient = Address::generate(&env);
    IotContractClient::new(&env, &cid)
        .withdraw_platform_fees(&impostor, &token_id, &recipient, &1);
}

// ── full integration: register → pay → withdraw ───────────────────────────────

#[test]
fn test_full_integration_flow() {
    let (env, cid, token_id, admin) = setup_with_fee(1_000); // 10 %
    let c = IotContractClient::new(&env, &cid);
    let asset = token::StellarAssetClient::new(&env, &token_id);
    let tok = token::Client::new(&env, &token_id);

    let owner1 = Address::generate(&env);
    let owner2 = Address::generate(&env);
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let treasury = Address::generate(&env);

    c.init_device(&symbol_short!("s1"), &1_000, &owner1);
    c.init_device(&symbol_short!("s2"), &2_000, &owner2);

    asset.mint(&user1, &1_000);
    asset.mint(&user2, &2_000);

    assert!(c.request_access(&symbol_short!("s1"), &user1, &token_id, &1_000));
    assert!(c.request_access(&symbol_short!("s2"), &user2, &token_id, &2_000));

    assert_eq!(tok.balance(&owner1), 900);   // 1000 - 10%
    assert_eq!(tok.balance(&owner2), 1_800); // 2000 - 10%
    assert_eq!(c.get_platform_fee_balance(&token_id), 300); // 100+200

    c.withdraw_platform_fees(&admin, &token_id, &treasury, &300);
    assert_eq!(tok.balance(&treasury), 300);
    assert_eq!(c.get_platform_fee_balance(&token_id), 0);
}
