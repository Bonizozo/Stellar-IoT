'use client'

import { useCallback, useEffect, useState } from 'react'
import { QrCode, ScanLine, RefreshCw } from 'lucide-react'
import QrCodeCard from '@/components/QrCodeCard'
import { getDevices, getQrAnalytics } from '@/services/api'
import { Device, QrScanAnalytics } from '@/types'

export default function QrPage() {
  const [devices, setDevices] = useState<Device[]>([])
  const [selectedId, setSelectedId] = useState('')
  const [analytics, setAnalytics] = useState<QrScanAnalytics | null>(null)
  const [loading, setLoading] = useState(true)

  // Custom branding controls
  const [brandText, setBrandText] = useState('Stellar IoT')
  const [tagline, setTagline] = useState('Scan to pay & unlock access')
  const [accentColor, setAccentColor] = useState('#7c3aed')

  useEffect(() => {
    getDevices()
      .then((d) => {
        setDevices(d)
        if (d.length) setSelectedId(d[0].id)
      })
      .catch((e) => console.error('Failed to load devices:', e))
      .finally(() => setLoading(false))
  }, [])

  const selected = devices.find((d) => d.id === selectedId)

  const loadAnalytics = useCallback(async () => {
    if (!selectedId) return
    try {
      setAnalytics(await getQrAnalytics(selectedId))
    } catch (e) {
      console.error('Failed to load QR analytics:', e)
    }
  }, [selectedId])

  useEffect(() => {
    loadAnalytics()
  }, [loadAnalytics])

  return (
    <div className="container mx-auto px-4 py-8">
      <div className="max-w-5xl mx-auto">
        <h1 className="text-3xl font-bold mb-2 flex items-center gap-2">
          <QrCode /> Device QR Codes
        </h1>
        <p className="text-gray-600 dark:text-gray-400 mb-8">
          Generate a printable QR card for each device. Users scan it to jump straight to the payment page —
          every scan is tracked below.
        </p>

        {loading ? (
          <div className="text-center py-12">Loading devices...</div>
        ) : devices.length === 0 ? (
          <div className="text-center py-12 text-gray-500">No devices available.</div>
        ) : (
          <div className="grid md:grid-cols-2 gap-8">
            {/* Controls + card */}
            <div className="space-y-6">
              <div>
                <label className="block text-sm font-medium mb-1">Device</label>
                <select value={selectedId} onChange={(e) => setSelectedId(e.target.value)}
                  className="w-full border rounded-lg px-3 py-2 bg-white dark:bg-gray-700 dark:border-gray-600">
                  {devices.map((d) => (
                    <option key={d.id} value={d.id}>{d.name}</option>
                  ))}
                </select>
              </div>

              <div className="bg-white dark:bg-gray-800 rounded-lg shadow p-4 space-y-4">
                <p className="text-sm font-semibold text-gray-500">Custom branding</p>
                <div className="grid grid-cols-2 gap-3">
                  <div>
                    <label className="block text-xs text-gray-500 mb-1">Brand name</label>
                    <input value={brandText} onChange={(e) => setBrandText(e.target.value)}
                      className="w-full border rounded-lg px-3 py-2 bg-white dark:bg-gray-700 dark:border-gray-600 text-sm" />
                  </div>
                  <div>
                    <label className="block text-xs text-gray-500 mb-1">Accent colour</label>
                    <input type="color" value={accentColor} onChange={(e) => setAccentColor(e.target.value)}
                      className="w-full h-10 border rounded-lg bg-white dark:bg-gray-700 dark:border-gray-600" />
                  </div>
                </div>
                <div>
                  <label className="block text-xs text-gray-500 mb-1">Tagline</label>
                  <input value={tagline} onChange={(e) => setTagline(e.target.value)}
                    className="w-full border rounded-lg px-3 py-2 bg-white dark:bg-gray-700 dark:border-gray-600 text-sm" />
                </div>
              </div>

              {selected && (
                <QrCodeCard
                  device={selected}
                  brandText={brandText}
                  tagline={tagline}
                  accentColor={accentColor}
                />
              )}
            </div>

            {/* Analytics */}
            <div>
              <div className="flex items-center justify-between mb-4">
                <h2 className="text-xl font-semibold flex items-center gap-2"><ScanLine size={20} /> Scan Analytics</h2>
                <button onClick={loadAnalytics} className="text-sm text-stellar-purple hover:underline flex items-center gap-1">
                  <RefreshCw size={14} /> Refresh
                </button>
              </div>

              {analytics ? (
                <div className="space-y-4">
                  <div className="grid grid-cols-2 gap-4">
                    <div className="bg-white dark:bg-gray-800 rounded-lg shadow p-5">
                      <p className="text-sm text-gray-500">Total Scans</p>
                      <p className="text-3xl font-bold text-stellar-purple mt-1">{analytics.total_scans}</p>
                    </div>
                    <div className="bg-white dark:bg-gray-800 rounded-lg shadow p-5">
                      <p className="text-sm text-gray-500">Last Scan</p>
                      <p className="text-sm font-medium mt-2">
                        {analytics.last_scan ? new Date(analytics.last_scan).toLocaleString() : 'Never'}
                      </p>
                    </div>
                  </div>

                  <div className="bg-white dark:bg-gray-800 rounded-lg shadow p-5">
                    <p className="text-sm font-semibold text-gray-500 mb-3">Scans by day</p>
                    {analytics.daily.length === 0 ? (
                      <p className="text-sm text-gray-400">No scans recorded yet.</p>
                    ) : (
                      <ul className="space-y-2">
                        {analytics.daily.map((d) => (
                          <li key={d.date} className="flex items-center gap-3">
                            <span className="text-xs text-gray-500 w-24">{d.date}</span>
                            <div className="flex-1 bg-gray-100 dark:bg-gray-700 rounded-full h-3 overflow-hidden">
                              <div className="h-full bg-stellar-purple"
                                style={{ width: `${Math.min(100, (d.scans / Math.max(...analytics.daily.map((x) => x.scans))) * 100)}%` }} />
                            </div>
                            <span className="text-sm font-medium w-8 text-right">{d.scans}</span>
                          </li>
                        ))}
                      </ul>
                    )}
                  </div>
                </div>
              ) : (
                <div className="text-center py-12 text-gray-500">Loading analytics...</div>
              )}
            </div>
          </div>
        )}
      </div>
    </div>
  )
}
