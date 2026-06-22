import {
  Account,
  Asset,
  BASE_FEE,
  Keypair,
  Networks,
  Operation,
  Horizon,
  TransactionBuilder,
  Transaction,
  FeeBumpTransaction,
} from '@stellar/stellar-sdk';

const server = new Horizon.Server('https://horizon-testnet.stellar.org');
const NETWORK_PASSPHRASE = Networks.TESTNET;

/**
 * Builds a payment transaction for device access
 * In a real implementation, this would call the Soroban smart contract
 * For now, returns a basic payment transaction that can be signed
 */
export async function buildPaymentTransaction(
  sourceAddress: string,
  destinationAddress: string,
  amount: number
): Promise<string> {
  try {
    // Load the account to get the correct sequence number
    const account = await server.loadAccount(sourceAddress);
    
    // Create a transaction builder
    const builder = new TransactionBuilder(account, {
      fee: BASE_FEE,
      networkPassphrase: NETWORK_PASSPHRASE,
    });

    // Add a payment operation
    builder.addOperation(
      Operation.payment({
        destination: destinationAddress,
        asset: Asset.native(),
        amount: amount.toString(),
      })
    );

    // Set the timeout to 5 minutes
    builder.setTimeout(300);

    // Build the transaction
    const transaction = builder.build();

    // Return as XDR string
    return transaction.toXDR();
  } catch (error) {
    console.error('Error building transaction:', error);
    throw new Error('Failed to build payment transaction');
  }
}

/**
 * Validates if a Stellar address is valid
 */
export function isValidStellarAddress(address: string): boolean {
  try {
    Keypair.fromPublicKey(address);
    return true;
  } catch {
    return false;
  }
}

/**
 * Formats a Stellar address for display (truncated)
 */
export function formatAddress(address: string, chars = 6): string {
  if (!address || address.length < chars * 2) return address;
  return `${address.slice(0, chars)}...${address.slice(-chars)}`;
}

/**
 * Formats balance to a fixed number of decimal places
 */
export function formatBalance(balance: string | number, decimals = 2): string {
  const num = typeof balance === 'string' ? parseFloat(balance) : balance;
  return num.toFixed(decimals);
}

/**
 * Submits a signed transaction to the network
 */
export async function submitTransaction(signedXdr: string): Promise<string> {
  try {
    // Convert XDR string to Transaction object
    const transaction = TransactionBuilder.fromXDR(signedXdr, Networks.TESTNET);
    const result = await server.submitTransaction(transaction);
    return result.hash;
  } catch (error) {
    console.error('Error submitting transaction:', error);
    throw new Error('Failed to submit transaction to network');
  }
}

/**
 * Gets transaction details by hash
 */
export async function getTransactionDetails(hash: string) {
  try {
    return await server.transactions().transaction(hash).call();
  } catch (error) {
    console.error('Error fetching transaction details:', error);
    throw new Error('Failed to fetch transaction details');
  }
}

/**
 * Waits for a transaction to be confirmed
 */
export async function waitForTransaction(
  hash: string,
  maxAttempts = 30,
  delayMs = 1000
): Promise<boolean> {
  let attempts = 0;

  while (attempts < maxAttempts) {
    try {
      await getTransactionDetails(hash);
      return true;
    } catch {
      attempts++;
      if (attempts >= maxAttempts) {
        throw new Error('Transaction confirmation timeout');
      }
      await new Promise(resolve => setTimeout(resolve, delayMs));
    }
  }

  return false;
}
