'use client'

import { useEffect, useState, useCallback } from 'react'
import { OwnerEarningsResponse, OwnerDeviceStatus, ReportPeriod } from '@/types'
import { getOwnerEarnings, getOwnerDevices, withdrawEarnings } from '@/services/api'
import { useWallet } from '@/providers/WalletProvider'
import { TrendingUp, Clock, Download, AlertCircle, CheckCircle, Loader2 } from 'lucide-react'

function StatCard({ label, value, sub, icon }: { label: string; value: string; sub?: string; icon?: React.ReactNode }) {
  return (
    <div className="bg-white dark:bg-gray-800 rounded-xl shadow p-5 flex flex-col gap-1">
      <div className="flex items-center justify-between">
        <span className="text-xs text-gray-500 uppercase tracking-wide">{label}</span>
        {icon && <span className="text-gray-400">{icon}</span>}
      </div>
      <span className="text-2xl font-bold">{value}</span>
      {sub && <span className="text-xs text-gray-400">{sub}</span>}
    </div>
  )
}

function BarChart({
  data,
  valueKey,
  labelKey,
  color = 'bg-indigo-500',
}: {
  data: Record<string, unknown>[]
  valueKey: string
  labelKey: string
  color?: string
}) {
  const max = Math.max(...data.map((d) => Number(d[valueKey])), 1)
  return (
    <div className="flex items-end gap-1 h-32">
      {data.map((d, i) => {
        const pct = (Number(d[valueKey]) / max) * 100
        return (
          <div key={i} className="flex flex-col items-center flex-1 gap-1">
            <div
              className={`w-full rounded-t ${color} transition-all`}
              style={{ height: `${pct}%` }}
              title={`${String(d[labelKey])}: ${Number(d[valueKey]).toFixed(2)}`}
            />
            {data.length <= 12 && (
              <span className="text-[9px] text-gray-400 truncate w-full text-center">
                {String(d[labelKey]).slice(-5)}
              </span>
            )}
          </div>
        )
      })}
    </div>
  )
}

export default function EarningsDashboard() {
  const { publicKey, isConnected } = useWallet()
  const [report, setReport] = useState<OwnerEarningsResponse | null>(null)
  const [devices, setDevices] = useState<OwnerDeviceStatus[]>([])
  const [period, setPeriod] = useState<ReportPeriod>('daily')
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [withdrawAmount, setWithdrawAmount] = useState('')
  const [withdrawDest, setWithdrawDest] = useState('')
  const [withdrawing, setWithdrawing] = useState(false)
  const [withdrawResult, setWithdrawResult] = useState<{ success: boolean; message: string } | null>(null)

  const addr = publicKey || ''

  const loadData = useCallback(async () => {
    if (!addr) {
      setLoading(false)
      return
    }
    setLoading(true)
    setError(null)
    try {
      const [earnings, devs] = await Promise.all([
        getOwnerEarnings(addr, period),
        getOwnerDevices(addr),
      ])
      setReport(earnings)
      setDevices(devs)
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : 'Failed to load earnings data')
    } finally {
      setLoading(false)
    }
  }, [addr, period])

  useEffect(() => { loadData() }, [loadData])

  const handleWithdraw = async () => {
    if (!addr || !withdrawAmount || !withdrawDest) return
    setWithdrawing(true)
    setWithdrawResult(null)
    try {
      const res = await withdrawEarnings({
        owner_address: addr,
        amount: parseFloat(withdrawAmount),
        destination_address: withdrawDest,
      })
      setWithdrawResult({ success: res.success, message: `${res.message} — Tx: ${res.tx_hash.slice(0, 16)}… Fee: ${res.fee} XLM` })
      setWithdrawAmount('')
      setWithdrawDest('')
    } catch (e: unknown) {
      setWithdrawResult({ success: false, message: e instanceof Error ? e.message : 'Withdrawal failed' })
    } finally {
      setWithdrawing(false)
    }
  }

  if (!isConnected || !addr) {
    return (
      <div className="text-center py-20">
        <div className="inline-flex items-center justify-center w-16 h-16 rounded-full bg-gray-100 dark:bg-gray-700 mb-4">
          <TrendingUp className="w-8 h-8 text-gray-400" />
        </div>
        <h2 className="text-xl font-semibold mb-2">Connect Your Wallet</h2>
        <p className="text-gray-500">Connect your Freighter wallet to view your earnings dashboard.</p>
      </div>
    )
  }

  if (loading) {
    return (
      <div className="text-center py-20">
        <Loader2 className="w-8 h-8 animate-spin mx-auto text-indigo-500" />
        <p className="mt-4 text-gray-400">Loading earnings data…</p>
      </div>
    )
  }

  if (error) {
    return (
      <div className="rounded-lg bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 p-4 text-red-600 dark:text-red-400">
        <div className="flex items-center gap-2">
          <AlertCircle className="w-5 h-5" />
          <span>{error}</span>
        </div>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      {/* Period selector */}
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div className="flex gap-2">
          {(['daily', 'weekly', 'monthly'] as ReportPeriod[]).map((p) => (
            <button
              key={p}
              onClick={() => setPeriod(p)}
              className={`px-3 py-1 rounded-full text-sm capitalize transition-colors ${
                period === p
                  ? 'bg-indigo-600 text-white'
                  : 'bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-300 hover:bg-indigo-100'
              }`}
            >
              {p}
            </button>
          ))}
        </div>
      </div>

      {report && (
        <>
          {/* Summary stats */}
          <div className="grid grid-cols-2 sm:grid-cols-4 gap-4">
            <StatCard
              label="Total Earnings"
              value={`${report.total_earnings_xlm.toLocaleString()} XLM`}
              icon={<TrendingUp className="w-4 h-4" />}
            />
            <StatCard
              label="Pending"
              value={`${report.pending_earnings_xlm.toLocaleString()} XLM`}
              sub="awaiting confirmation"
              icon={<Clock className="w-4 h-4" />}
            />
            <StatCard
              label="Devices"
              value={report.total_devices.toLocaleString()}
              sub={`${report.uptime_avg.toFixed(1)}% avg uptime`}
            />
            <StatCard
              label="Total Sessions"
              value={report.total_sessions.toLocaleString()}
              sub={`period: ${report.period}`}
            />
          </div>

          {/* Earnings chart */}
          <div className="bg-white dark:bg-gray-800 rounded-xl shadow p-5">
            <h3 className="text-sm font-semibold mb-4 text-gray-700 dark:text-gray-200">
              Revenue Over Time ({report.period})
            </h3>
            {report.time_series.length > 0 ? (
              <BarChart
                data={report.time_series as unknown as Record<string, unknown>[]}
                valueKey="revenue"
                labelKey="date"
                color="bg-indigo-500"
              />
            ) : (
              <p className="text-center text-gray-400 py-8">No time-series data available</p>
            )}
          </div>

          {/* Session count chart */}
          <div className="bg-white dark:bg-gray-800 rounded-xl shadow p-5">
            <h3 className="text-sm font-semibold mb-4 text-gray-700 dark:text-gray-200">
              Sessions Over Time ({report.period})
            </h3>
            {report.time_series.length > 0 ? (
              <BarChart
                data={report.time_series as unknown as Record<string, unknown>[]}
                valueKey="session_count"
                labelKey="date"
                color="bg-violet-400"
              />
            ) : (
              <p className="text-center text-gray-400 py-8">No session data available</p>
            )}
          </div>

          {/* Top performing devices */}
          <div className="bg-white dark:bg-gray-800 rounded-xl shadow p-5">
            <h3 className="text-sm font-semibold mb-4 text-gray-700 dark:text-gray-200">
              Top Performing Devices
            </h3>
            {report.top_devices.length > 0 ? (
              <div className="space-y-3">
                {report.top_devices.map((d, i) => (
                  <div
                    key={d.id}
                    className="flex items-center gap-4 p-3 rounded-lg bg-gray-50 dark:bg-gray-700/50"
                  >
                    <span className="text-lg font-bold text-gray-300 w-6 shrink-0">#{i + 1}</span>
                    <div className="flex-1 min-w-0">
                      <p className="font-medium truncate">{d.name}</p>
                      <p className="text-xs text-gray-400">
                        {d.sessions.toLocaleString()} sessions &middot; {d.uptime_pct.toFixed(1)}% uptime
                      </p>
                    </div>
                    <span className="text-lg font-bold text-indigo-600 dark:text-indigo-400 shrink-0">
                      {d.earnings.toLocaleString()} XLM
                    </span>
                  </div>
                ))}
              </div>
            ) : (
              <p className="text-center text-gray-400 py-8">No devices registered</p>
            )}
          </div>

          {/* Device status table */}
          <div className="bg-white dark:bg-gray-800 rounded-xl shadow p-5">
            <h3 className="text-sm font-semibold mb-4 text-gray-700 dark:text-gray-200">
              Device Status &amp; Uptime
            </h3>
            {devices.length > 0 ? (
              <div className="overflow-x-auto">
                <table className="w-full text-sm">
                  <thead>
                    <tr className="text-gray-400 text-left border-b dark:border-gray-700">
                      <th className="pb-2 pr-3">Device</th>
                      <th className="pb-2 pr-3">Status</th>
                      <th className="pb-2 pr-3 text-right">Uptime</th>
                      <th className="pb-2 pr-3 text-right">Sessions</th>
                      <th className="pb-2 pr-3 text-right">Price</th>
                      <th className="pb-2 text-right">Earnings</th>
                    </tr>
                  </thead>
                  <tbody>
                    {devices.map((d) => (
                      <tr key={d.id} className="border-b dark:border-gray-700 last:border-0">
                        <td className="py-2 pr-3 font-medium">{d.name}</td>
                        <td className="py-2 pr-3">
                          {d.online ? (
                            <span className="inline-flex items-center gap-1 text-emerald-600">
                              <CheckCircle className="w-3.5 h-3.5" />
                              Online
                            </span>
                          ) : (
                            <span className="inline-flex items-center gap-1 text-red-500">
                              <AlertCircle className="w-3.5 h-3.5" />
                              Offline
                            </span>
                          )}
                        </td>
                        <td className="py-2 pr-3 text-right">{d.uptime_pct.toFixed(1)}%</td>
                        <td className="py-2 pr-3 text-right">{d.total_sessions.toLocaleString()}</td>
                        <td className="py-2 pr-3 text-right">{d.price.toFixed(2)} XLM</td>
                        <td className="py-2 text-right font-semibold">{d.total_earnings.toFixed(2)} XLM</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            ) : (
              <p className="text-center text-gray-400 py-8">No devices found</p>
            )}
          </div>

          {/* Withdrawal section */}
          <div className="bg-white dark:bg-gray-800 rounded-xl shadow p-5">
            <h3 className="text-sm font-semibold mb-4 text-gray-700 dark:text-gray-200 flex items-center gap-2">
              <Download className="w-4 h-4" />
              Withdraw Earnings
            </h3>
            <div className="flex flex-col sm:flex-row gap-3 items-start sm:items-end">
              <div className="flex-1 w-full">
                <label className="block text-xs text-gray-500 mb-1">Amount (XLM)</label>
                <input
                  type="number"
                  step="0.01"
                  min="0"
                  placeholder="0.00"
                  value={withdrawAmount}
                  onChange={(e) => setWithdrawAmount(e.target.value)}
                  className="w-full px-3 py-2 rounded-lg border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-sm focus:ring-2 focus:ring-indigo-500 focus:border-transparent outline-none"
                />
              </div>
              <div className="flex-[2] w-full">
                <label className="block text-xs text-gray-500 mb-1">Destination Address</label>
                <input
                  type="text"
                  placeholder="G…"
                  value={withdrawDest}
                  onChange={(e) => setWithdrawDest(e.target.value)}
                  className="w-full px-3 py-2 rounded-lg border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-sm font-mono focus:ring-2 focus:ring-indigo-500 focus:border-transparent outline-none"
                />
              </div>
              <button
                onClick={handleWithdraw}
                disabled={withdrawing || !withdrawAmount || !withdrawDest}
                className="px-5 py-2 rounded-lg bg-indigo-600 text-white text-sm font-medium hover:bg-indigo-700 disabled:bg-gray-400 disabled:cursor-not-allowed transition-colors flex items-center gap-2 shrink-0"
              >
                {withdrawing ? <Loader2 className="w-4 h-4 animate-spin" /> : <Download className="w-4 h-4" />}
                {withdrawing ? 'Processing…' : 'Withdraw'}
              </button>
            </div>
            {withdrawResult && (
              <div
                className={`mt-3 p-3 rounded-lg text-sm flex items-center gap-2 ${
                  withdrawResult.success
                    ? 'bg-emerald-50 dark:bg-emerald-900/20 text-emerald-700 dark:text-emerald-300 border border-emerald-200 dark:border-emerald-800'
                    : 'bg-red-50 dark:bg-red-900/20 text-red-600 dark:text-red-400 border border-red-200 dark:border-red-800'
                }`}
              >
                {withdrawResult.success ? <CheckCircle className="w-4 h-4 shrink-0" /> : <AlertCircle className="w-4 h-4 shrink-0" />}
                <span>{withdrawResult.message}</span>
              </div>
            )}
          </div>
        </>
      )}
    </div>
  )
}
