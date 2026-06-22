'use client'

import { useState } from 'react'
import { Device } from '@/types'
import { useWallet } from '@/providers/WalletProvider'
import PaymentModal from './PaymentModal'
import { AlertCircle, CheckCircle } from 'lucide-react'

interface PayButtonProps {
  device: Device
}

export default function PayButton({ device }: PayButtonProps) {
  const { isConnected, publicKey } = useWallet()
  const [isModalOpen, setIsModalOpen] = useState(false)
  const [success, setSuccess] = useState(false)
  const [successTxHash, setSuccessTxHash] = useState<string | null>(null)
  const [error, setError] = useState<string | null>(null)

  const handlePaymentSuccess = (txHash: string) => {
    setSuccess(true)
    setSuccessTxHash(txHash)
    setError(null)
    
    // Reset success state after 5 seconds
    setTimeout(() => {
      setSuccess(false)
      setSuccessTxHash(null)
    }, 5000)
  }

  if (success) {
    return (
      <div className="bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800 rounded-xl p-4 flex items-start space-x-3">
        <CheckCircle className="text-green-600 flex-shrink-0 mt-0.5" size={20} />
        <div className="flex-1">
          <p className="font-semibold text-green-800 dark:text-green-300">
            ✓ Access Granted!
          </p>
          <p className="text-xs text-green-700 dark:text-green-400 mt-1">
            Device is now unlocked. Transaction: {successTxHash?.slice(0, 20)}...
          </p>
        </div>
      </div>
    )
  }

  return (
    <div>
      {error && (
        <div className="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-xl p-4 mb-4 flex items-start space-x-3">
          <AlertCircle className="text-red-600 flex-shrink-0 mt-0.5" size={20} />
          <div>
            <p className="font-semibold text-red-800 dark:text-red-300">Payment Failed</p>
            <p className="text-sm text-red-700 dark:text-red-400 mt-1">{error}</p>
          </div>
        </div>
      )}
      
      <button
        onClick={() => setIsModalOpen(true)}
        disabled={!device.available}
        className="w-full bg-stellar-purple text-white py-3 rounded-lg font-semibold hover:bg-opacity-90 disabled:bg-gray-400 disabled:cursor-not-allowed transition-colors duration-200"
      >
        {device.available ? `Pay ${device.price} XLM to Unlock` : 'Device Unavailable'}
      </button>
      
      <p className="text-xs text-gray-500 dark:text-gray-400 mt-3 text-center">
        {isConnected && publicKey ? (
          <>Payment will be processed via Stellar blockchain using your Freighter wallet</>
        ) : (
          <>Connect your Freighter wallet first to make a payment</>
        )}
      </p>

      <PaymentModal
        device={{
          id: device.id,
          name: device.name,
          pricePerUse: device.price,
        }}
        isOpen={isModalOpen}
        onClose={() => setIsModalOpen(false)}
        onSuccess={(txHash) => {
          setIsModalOpen(false)
          handlePaymentSuccess(txHash)
        }}
      />
    </div>
  )
}
