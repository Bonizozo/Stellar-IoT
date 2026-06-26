'use client'

import { useCallback, useEffect, useMemo, useState } from 'react'
import { ExternalLink, Download, Clock, Wallet, Receipt } from 'lucide-react'
import { PaymentHistoryEntry, PaymentStatus } from '@/types'
import { getPaymentHistory, getPaymentHistoryCsvUrl } from '@/services/api'
import { useWallet } from '@/providers/WalletProvider'
import { formatAddress, getExplorerTxUrl } from '@/lib/stellar'

const STATUS_STYLES: Record<PaymentStatus, string> = {
  active: 'bg-green-100 text-green-800',
  expired: 'bg-gray-100 text-gray-600',
  ended: 'bg-yellow-100 text-yellow-800',
}

function formatDuration(secs: number): string {
  if (secs <= 0) return '0m'
  const h = Math.floor(secs / 3600)
  const m = Math.floor((secs % 3600) / 60)
  const parts: string[] = []
  if (h) parts.push(`${h}h`)
  parts.push(`${m}m`)
  return parts.join(' ')
}

export default function PaymentHistory() {
  const { publicKey, isConnected } = useWallet()
  const [entries, setEntries] = useState<PaymentHistoryEntry[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  // Filters
  const [device, setDevice] = useState('all')
  const [status, setStatus] = useState<'all' | PaymentStatus>('all')
  const [from, setFrom] = useState('')
  const [to, setTo] = useState('')

  const load = useCallback(async () => {
    if (!publicKey) return
    setLoading(true)
    setError(null)
    try {
      const res = await getPaymentHistory(publicKey)
      setEntries(res.data)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load payment history')
    } finally {
      setLoading(false)
    }
  }, [publicKey])

  useEffect(() => {
    load()
  }, [load])

  // Unique devices for the filter dropdown, derived from the full history.
  const deviceOptions = useMemo(() => {
    const map = new Map<string, string>()
    entries.forEach((e) => map.set(e.device_id, e.device_name))
    return Array.from(map.entries())
  }, [entries])

  const filtered = useMemo(() => {
    return entries.filter((e) => {
      if (device !== 'all' && e.device_id !== device) return false
      if (status !== 'all' && e.status !== status) return false
      if (from && new Date(e.created_at) < new Date(from)) return false
      if (to && new Date(e.created_at) > new Date(`${to}T23:59:59`)) return false
      return true
    })
  }, [entries, device, status, from, to])

  const totals = useMemo(() => {
    return filtered.reduce(
      (acc, e) => {
        acc.spent += e.amount
        acc.duration += e.duration_secs
        return acc
      },
      { spent: 0, duration: 0 },
    )
  }, [filtered])

  const csvUrl = useMemo(() => {
    if (!publicKey) return '#'
    return getPaymentHistoryCsvUrl(publicKey, {
      device_id: device !== 'all' ? device : undefined,
      status: status !== 'all' ? status : undefined,
      from: from || undefined,
      to: to || undefined,
    })
  }, [publicKey, device, status, from, to])

  if (!isConnected || !publicKey) {
    return (
      <div className="text-center py-16 text-gray-500">
        <Wallet className="mx-auto mb-3" />
        Connect your wallet to view your payment history.
      </div>
    )
  }

  if (loading) return <div className="text-center py-12">Loading payment history...</div>
  if (error) return <div className="bg-red-50 text-red-700 p-4 rounded-lg">{error}</div>

  return (
    <div>
      {/* Summary cards */}
      <div className="grid grid-cols-1 sm:grid-cols-3 gap-4 mb-8">
        <div className="bg-white dark:bg-gray-800 rounded-lg shadow p-5">
          <p className="text-sm text-gray-500 flex items-center gap-1"><Wallet size={14} /> Total Spent</p>
          <p className="text-2xl font-bold text-stellar-purple mt-1">{totals.spent.toFixed(2)} XLM</p>
        </div>
        <div className="bg-white dark:bg-gray-800 rounded-lg shadow p-5">
          <p className="text-sm text-gray-500 flex items-center gap-1"><Receipt size={14} /> Transactions</p>
          <p className="text-2xl font-bold mt-1">{filtered.length}</p>
        </div>
        <div className="bg-white dark:bg-gray-800 rounded-lg shadow p-5">
          <p className="text-sm text-gray-500 flex items-center gap-1"><Clock size={14} /> Total Session Time</p>
          <p className="text-2xl font-bold mt-1">{formatDuration(totals.duration)}</p>
        </div>
      </div>

      {/* Filters */}
      <div className="flex flex-wrap items-end gap-3 mb-6">
        <div>
          <label className="block text-xs text-gray-500 mb-1">Device</label>
          <select value={device} onChange={(e) => setDevice(e.target.value)}
            className="border rounded-lg px-3 py-2 bg-white dark:bg-gray-700 dark:border-gray-600 text-sm">
            <option value="all">All devices</option>
            {deviceOptions.map(([id, name]) => (
              <option key={id} value={id}>{name}</option>
            ))}
          </select>
        </div>
        <div>
          <label className="block text-xs text-gray-500 mb-1">Status</label>
          <select value={status} onChange={(e) => setStatus(e.target.value as 'all' | PaymentStatus)}
            className="border rounded-lg px-3 py-2 bg-white dark:bg-gray-700 dark:border-gray-600 text-sm">
            <option value="all">All</option>
            <option value="active">Active</option>
            <option value="ended">Ended</option>
            <option value="expired">Expired</option>
          </select>
        </div>
        <div>
          <label className="block text-xs text-gray-500 mb-1">From</label>
          <input type="date" value={from} onChange={(e) => setFrom(e.target.value)}
            className="border rounded-lg px-3 py-2 bg-white dark:bg-gray-700 dark:border-gray-600 text-sm" />
        </div>
        <div>
          <label className="block text-xs text-gray-500 mb-1">To</label>
          <input type="date" value={to} onChange={(e) => setTo(e.target.value)}
            className="border rounded-lg px-3 py-2 bg-white dark:bg-gray-700 dark:border-gray-600 text-sm" />
        </div>
        {(device !== 'all' || status !== 'all' || from || to) && (
          <button onClick={() => { setDevice('all'); setStatus('all'); setFrom(''); setTo('') }}
            className="text-sm text-stellar-purple hover:underline py-2">Clear</button>
        )}
        <a href={csvUrl} download
          className="ml-auto bg-stellar-purple text-white px-4 py-2 rounded-lg text-sm font-medium hover:bg-opacity-90 transition flex items-center gap-2">
          <Download size={16} /> Export CSV
        </a>
      </div>

      {/* Table */}
      {filtered.length === 0 ? (
        <div className="text-center py-12 text-gray-500">No transactions found.</div>
      ) : (
        <div className="overflow-x-auto bg-white dark:bg-gray-800 rounded-lg shadow">
          <table className="w-full text-sm">
            <thead>
              <tr className="text-left text-gray-500 border-b dark:border-gray-700">
                <th className="px-4 py-3 font-medium">Date</th>
                <th className="px-4 py-3 font-medium">Device</th>
                <th className="px-4 py-3 font-medium">Amount</th>
                <th className="px-4 py-3 font-medium">Status</th>
                <th className="px-4 py-3 font-medium">Duration</th>
                <th className="px-4 py-3 font-medium">Transaction</th>
              </tr>
            </thead>
            <tbody>
              {filtered.map((e) => (
                <tr key={e.id} className="border-b last:border-0 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-700/50">
                  <td className="px-4 py-3 whitespace-nowrap">{new Date(e.created_at).toLocaleString()}</td>
                  <td className="px-4 py-3">{e.device_name}</td>
                  <td className="px-4 py-3 font-medium">{e.amount.toFixed(2)} XLM</td>
                  <td className="px-4 py-3">
                    <span className={`text-xs px-2 py-1 rounded capitalize ${STATUS_STYLES[e.status]}`}>{e.status}</span>
                  </td>
                  <td className="px-4 py-3">{formatDuration(e.duration_secs)}</td>
                  <td className="px-4 py-3">
                    <a href={getExplorerTxUrl(e.tx_hash)} target="_blank" rel="noopener noreferrer"
                      className="text-stellar-purple hover:underline inline-flex items-center gap-1 font-mono">
                      {formatAddress(e.tx_hash, 5)} <ExternalLink size={12} />
                    </a>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  )
}
