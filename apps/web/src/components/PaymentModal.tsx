'use client';

import React, { useState } from 'react';
import { useWallet } from '../providers/WalletProvider';
import { buildPaymentTransaction, submitTransaction, formatAddress, formatBalance } from '@/lib/stellar';
import { AlertCircle, CheckCircle, Loader } from 'lucide-react';

interface Device {
  id: string;
  name: string;
  pricePerUse: number;
}

interface PaymentModalProps {
  device: Device | null;
  isOpen: boolean;
  onClose: () => void;
  onSuccess: (txHash: string) => void;
}

type TransactionState = 'idle' | 'building' | 'signing' | 'submitting' | 'success' | 'error';

const PaymentModal = ({ device, isOpen, onClose, onSuccess }: PaymentModalProps) => {
  const { publicKey, connect, balance, signTransaction, isConnected } = useWallet();
  const [state, setState] = useState<TransactionState>('idle');
  const [txHash, setTxHash] = useState<string | null>(null);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  const handlePayment = async () => {
    if (!publicKey) {
      await connect();
      return;
    }

    if (!device) return;

    try {
      setState('building');
      setErrorMessage(null);

      // Check balance
      const balanceNum = parseFloat(balance);
      if (balanceNum < device.pricePerUse) {
        setErrorMessage(
          `Insufficient balance. You need ${device.pricePerUse} XLM but only have ${formatBalance(balance)} XLM`
        );
        setState('error');
        return;
      }

      // Build the transaction
      // In production, this would call the Soroban contract address
      // For now, using a test recipient address
      const testRecipientAddress = 'GBUQWP3BOUZX34ULNQG23RQ6F4YUSXHTQSXUSMIQSTBE2EURIDVXL6B';
      
      const transactionXdr = await buildPaymentTransaction(
        publicKey,
        testRecipientAddress,
        device.pricePerUse
      );

      setState('signing');

      // Sign the transaction
      const signedXdr = await signTransaction(transactionXdr);

      setState('submitting');

      // Submit the transaction
      const hash = await submitTransaction(signedXdr);
      
      setTxHash(hash);
      setState('success');

      // Call success callback after a brief delay
      setTimeout(() => {
        onSuccess(hash);
        onClose();
      }, 2000);
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Transaction failed';
      setErrorMessage(message);
      setState('error');
      console.error('Payment error:', error);
    }
  };

  if (!isOpen || !device) return null;

  const isLoading = state === 'building' || state === 'signing' || state === 'submitting';

  return (
    <div className="fixed inset-0 bg-black/70 flex items-center justify-center z-50">
      <div className="bg-white dark:bg-gray-900 rounded-3xl p-8 max-w-md w-full mx-4 shadow-2xl">
        <h2 className="text-2xl font-bold mb-6">Confirm Payment</h2>

        {/* Device Details */}
        <div className="bg-gradient-to-r from-stellar-purple/10 to-blue-500/10 dark:bg-gray-800 rounded-2xl p-5 mb-6">
          <p className="text-sm text-gray-600 dark:text-gray-400">Device</p>
          <p className="font-semibold text-lg text-gray-900 dark:text-white">{device.name}</p>
          <p className="text-3xl font-bold mt-3 text-stellar-purple">{device.pricePerUse} XLM</p>
        </div>

        {/* Wallet Info */}
        {isConnected && publicKey && (
          <div className="bg-gray-50 dark:bg-gray-800 rounded-xl p-4 mb-6 text-sm">
            <p className="text-gray-600 dark:text-gray-400 mb-1">Paying from</p>
            <p className="font-mono text-xs break-all text-gray-900 dark:text-gray-100 mb-3">
              {publicKey}
            </p>
            <div className="flex justify-between items-center">
              <span className="text-gray-600 dark:text-gray-400">Your Balance:</span>
              <span className={`font-bold ${parseFloat(balance) >= device.pricePerUse ? 'text-green-600' : 'text-red-600'}`}>
                {formatBalance(balance)} XLM
              </span>
            </div>
          </div>
        )}

        {/* Status Messages */}
        {state === 'success' && (
          <div className="bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800 rounded-xl p-4 mb-6 flex items-start space-x-3">
            <CheckCircle className="text-green-600 flex-shrink-0 mt-0.5" size={20} />
            <div>
              <p className="font-semibold text-green-800 dark:text-green-300">Payment Successful!</p>
              <p className="text-xs text-green-700 dark:text-green-400 mt-1 font-mono break-all">{txHash}</p>
            </div>
          </div>
        )}

        {state === 'error' && (
          <div className="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-xl p-4 mb-6 flex items-start space-x-3">
            <AlertCircle className="text-red-600 flex-shrink-0 mt-0.5" size={20} />
            <div>
              <p className="font-semibold text-red-800 dark:text-red-300">Transaction Failed</p>
              <p className="text-xs text-red-700 dark:text-red-400 mt-1">{errorMessage}</p>
            </div>
          </div>
        )}

        {/* Loading State */}
        {isLoading && (
          <div className="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-xl p-4 mb-6 flex items-center space-x-3">
            <Loader className="text-blue-600 animate-spin" size={20} />
            <div>
              <p className="font-semibold text-blue-800 dark:text-blue-300">
                {state === 'building' && 'Building transaction...'}
                {state === 'signing' && 'Waiting for signature...'}
                {state === 'submitting' && 'Submitting to network...'}
              </p>
              <p className="text-xs text-blue-700 dark:text-blue-400 mt-1">
                {state === 'building' && 'Preparing your payment transaction'}
                {state === 'signing' && 'Please sign with your Freighter wallet'}
                {state === 'submitting' && 'Processing on Stellar network'}
              </p>
            </div>
          </div>
        )}

        {/* Action Buttons */}
        <div className="space-y-3">
          {state !== 'success' ? (
            <>
              <button
                onClick={handlePayment}
                disabled={isLoading || state === 'error'}
                className="w-full bg-stellar-purple hover:bg-stellar-purple/90 disabled:bg-gray-400 disabled:cursor-not-allowed text-white py-3 rounded-xl font-semibold transition-all duration-200"
              >
                {isLoading ? (
                  <span className="flex items-center justify-center space-x-2">
                    <Loader size={16} className="animate-spin" />
                    <span>Processing...</span>
                  </span>
                ) : state === 'error' ? (
                  'Try Again'
                ) : !isConnected ? (
                  'Connect Freighter'
                ) : (
                  'Confirm & Pay'
                )}
              </button>
              <button
                onClick={() => {
                  onClose();
                  setState('idle');
                  setErrorMessage(null);
                  setTxHash(null);
                }}
                disabled={isLoading}
                className="w-full text-gray-700 dark:text-gray-300 hover:text-gray-900 dark:hover:text-gray-100 py-3 font-semibold disabled:opacity-50 disabled:cursor-not-allowed transition"
              >
                Cancel
              </button>
            </>
          ) : (
            <button
              onClick={() => {
                onClose();
                setState('idle');
                setErrorMessage(null);
                setTxHash(null);
              }}
              className="w-full bg-stellar-purple hover:bg-stellar-purple/90 text-white py-3 rounded-xl font-semibold transition"
            >
              Close
            </button>
          )}
        </div>

        {/* Info Text */}
        {!isLoading && state !== 'success' && state !== 'error' && (
          <p className="text-xs text-gray-500 dark:text-gray-400 mt-4 text-center">
            Payment will be processed via Stellar blockchain using XLM
          </p>
        )}
      </div>
    </div>
  );
};

export default PaymentModal;