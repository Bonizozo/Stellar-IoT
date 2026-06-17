# IoT Payment Smart Contract

Soroban smart contract for managing IoT device payments on Stellar with automatic revenue sharing.

## Functions

### initialize(admin, platform_fee_bps)
Initialize the contract admin and configurable platform fee in basis points. For example, `500` is a 5% platform fee.

### set_platform_fee(admin, platform_fee_bps)
Update the platform fee. Only the configured admin can call this function.

### get_platform_fee()
Get the current platform fee in basis points.

### get_platform_fee_balance(token)
Get the accumulated platform fees retained by the contract for a token.

### init_device(device_id, price, owner)
Initialize a device with its price and owner address. The owner authorizes the registration.

### get_device_price(device_id)
Get the price for a specific device.

### get_device_owner(device_id)
Get the owner address for a specific device.

### request_access(device_id, user, token, amount)
Process payment and grant device access. The contract transfers the net amount to the device owner immediately and retains the platform fee in the contract.

### pay(device_id, user, token, amount)
Backwards-compatible alias for `request_access`.

### withdraw_platform_fees(admin, token, to, amount)
Withdraw accumulated platform fees to a recipient. Only the configured admin can call this function.

## Building

```bash
cargo build --target wasm32-unknown-unknown --release
```

## Testing

```bash
cargo test
```

## Deploying

```bash
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/iot_contract.wasm \
  --network testnet \
  --source default
```

## Events

The contract emits two types of events:

- `payment` - When a payment is processed, including user, owner, gross amount, owner net amount, and platform fee.
- `access` - When device access is granted.

## Storage

The contract stores admin configuration, platform fee basis points, per-token platform fee balances, device prices, and device owners in contract instance storage.
