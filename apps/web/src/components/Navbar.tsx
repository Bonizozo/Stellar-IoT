'use client'

import Link from 'next/link'
import { useState } from 'react'
import { useWallet } from '@/providers/WalletProvider'
import { formatAddress, formatBalance } from '@/lib/stellar'
import { LogOut } from 'lucide-react'

export default function Navbar() {
  const { publicKey, isConnected, balance, loading, connect, disconnect, error } = useWallet()
  const [showDropdown, setShowDropdown] = useState(false)

  const handleConnect = async () => {
    try {
      await connect()
    } catch (err) {
      console.error('Failed to connect wallet:', err)
    }
  }

  return (
    <nav className="bg-white dark:bg-gray-900 shadow-md">
      <div className="container mx-auto px-4">
        <div className="flex justify-between items-center h-16">
          <Link href="/" className="flex items-center space-x-2">
            <span className="text-2xl font-bold text-stellar-purple">Stellar IoT</span>
          </Link>
          
          <div className="flex items-center space-x-6">
            <Link href="/" className="text-gray-700 dark:text-gray-300 hover:text-stellar-purple transition">
              Devices
            </Link>
            <Link href="/sessions" className="text-gray-700 dark:text-gray-300 hover:text-stellar-purple transition">
              Sessions
            </Link>
            <Link href="/register" className="text-gray-700 dark:text-gray-300 hover:text-stellar-purple transition">
              Register Device
            </Link>
            
            {/* Wallet Connection Section */}
            {isConnected && publicKey ? (
              <div className="relative">
                <button
                  onClick={() => setShowDropdown(!showDropdown)}
                  className="bg-stellar-purple text-white px-4 py-2 rounded-lg hover:bg-opacity-90 transition flex items-center space-x-2"
                >
                  <div className="text-sm">
                    <p className="font-semibold">{formatAddress(publicKey)}</p>
                    <p className="text-xs opacity-90">{formatBalance(balance)} XLM</p>
                  </div>
                </button>

                {/* Dropdown Menu */}
                {showDropdown && (
                  <div className="absolute right-0 mt-2 w-64 bg-white dark:bg-gray-800 rounded-lg shadow-xl z-50">
                    <div className="p-4 border-b border-gray-200 dark:border-gray-700">
                      <p className="text-xs text-gray-600 dark:text-gray-400 mb-1">Connected Wallet</p>
                      <p className="font-mono text-sm break-all">{publicKey}</p>
                      <p className="text-sm font-semibold mt-3 text-stellar-purple">
                        Balance: {formatBalance(balance)} XLM
                      </p>
                    </div>
                    <button
                      onClick={() => {
                        disconnect()
                        setShowDropdown(false)
                      }}
                      className="w-full text-left px-4 py-3 hover:bg-gray-100 dark:hover:bg-gray-700 flex items-center space-x-2 text-red-600"
                    >
                      <LogOut size={16} />
                      <span>Disconnect Wallet</span>
                    </button>
                  </div>
                )}
              </div>
            ) : (
              <button
                onClick={handleConnect}
                disabled={loading}
                className="bg-stellar-purple text-white px-4 py-2 rounded-lg hover:bg-opacity-90 disabled:bg-gray-400 disabled:cursor-not-allowed transition"
              >
                {loading ? 'Connecting...' : 'Connect Wallet'}
              </button>
            )}

            {error && (
              <div className="text-red-600 text-xs">
                Connection error
              </div>
            )}
          </div>
        </div>
      </div>
    </nav>
  )
}
