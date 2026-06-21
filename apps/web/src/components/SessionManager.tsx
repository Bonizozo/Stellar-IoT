'use client'

import { useEffect, useState, useCallback } from 'react'
import { Session } from '@/types'
import { getSessions, extendSession, endSession } from '@/services/api'

// Placeholder wallet address — replace with real Freighter integration
const USER_ADDRESS = 'GXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX'

function useCountdown(expiresAt: string): string {
  const calc = useCallback(() => {
    const diff = new Date(expiresAt).getTime() - Date.now()
    if (diff <= 0) return '00:00:00'
    const h = Math.floor(diff / 3_600_000)
    const m = Math.floor((diff % 3_600_000) / 60_000)
    const s = Math.floor((diff % 60_000) / 1_000)
    return [h, m, s].map(n => String(n).padStart(2, '0')).join(':')
  }, [expiresAt])
  const [time, setTime] = useState(calc)
  useEffect(() => {
    const id = setInterval(() => setTime(calc), 1_000)
    return () => clearInterval(id)
  }, [calc])
  return time
}

function SessionCard({ session, onEnd, onExtend }: {
  session: Session
  onEnd: (id: string) => void
  onExtend: (id: string) => void
}) {
  const countdown = useCountdown(session.expires_at)
  const expired = countdown === '00:00:00'
  const isUrgent = !expired && new Date(session.expires_at).getTime() - Date.now() < 5 * 60_000

  return (
    <div className={`bg-white dark:bg-gray-800 rounded-lg shadow p-6 border-l-4 ${session.active && !expired ? (isUrgent ? 'border-yellow-500' : 'border-green-500') : 'border-gray-400'}`}>
      <div className="flex justify-between items-start mb-3">
        <div>
          <h3 className="font-semibold text-lg">{session.device_name}</h3>
          <p className="text-xs text-gray-500">ID: {session.id}</p>
        </div>
        <span className={`text-xs px-2 py-1 rounded ${session.active && !expired ? 'bg-green-100 text-green-800' : 'bg-gray-100 text-gray-600'}`}>
          {session.active && !expired ? 'Active' : 'Expired'}
        </span>
      </div>

      {session.active && !expired && (
        <div className={`text-center rounded-lg py-3 mb-4 ${isUrgent ? 'bg-yellow-50 dark:bg-yellow-900/20' : 'bg-gray-50 dark:bg-gray-700'}`}>
          <p className="text-xs text-gray-500 mb-1">Time Remaining</p>
          <p className={`text-3xl font-mono font-bold ${isUrgent ? 'text-yellow-600' : 'text-stellar-purple'}`}>{countdown}</p>
          {isUrgent && <p className="text-xs text-yellow-600 mt-1">⚠ Session expiring soon</p>}
        </div>
      )}

      <div className="text-xs text-gray-500 space-y-1 mb-4">
        <p>Started: {new Date(session.created_at).toLocaleString()}</p>
        <p>Expires: {new Date(session.expires_at).toLocaleString()}</p>
        <p className="truncate">Address: {session.user_address}</p>
      </div>

      {session.active && !expired && (
        <div className="flex gap-2">
          <button onClick={() => onExtend(session.id)}
            className="flex-1 bg-stellar-purple text-white py-2 rounded-lg text-sm hover:bg-opacity-90 transition-colors">
            Extend Session
          </button>
          <button onClick={() => onEnd(session.id)}
            className="flex-1 border border-red-300 text-red-600 py-2 rounded-lg text-sm hover:bg-red-50 dark:hover:bg-red-900/20 transition-colors">
            End Session
          </button>
        </div>
      )}
    </div>
  )
}

export default function SessionManager() {
  const [sessions, setSessions] = useState<Session[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [activeFilter, setActiveFilter] = useState<'all' | 'active' | 'history'>('all')

  const load = useCallback(async () => {
    try {
      const data = await getSessions(USER_ADDRESS)
      setSessions(data)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load sessions')
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => { load() }, [load])

  const handleEnd = async (id: string) => {
    if (!confirm('End this session?')) return
    try {
      await endSession(id)
      setSessions(s => s.map(sess => sess.id === id ? { ...sess, active: false } : sess))
    } catch {
      alert('Failed to end session')
    }
  }

  const handleExtend = async (id: string) => {
    const session = sessions.find(s => s.id === id)
    if (!session) return
    try {
      await extendSession(id, { device_id: session.device_id, user_address: USER_ADDRESS, amount: 1 })
      await load()
    } catch {
      alert('Failed to extend session')
    }
  }

  const filtered = sessions.filter(s => {
    const expired = new Date(s.expires_at).getTime() < Date.now()
    if (activeFilter === 'active') return s.active && !expired
    if (activeFilter === 'history') return !s.active || expired
    return true
  })

  if (loading) return <div className="text-center py-12">Loading sessions...</div>
  if (error) return <div className="bg-red-50 text-red-700 p-4 rounded-lg">{error}</div>

  return (
    <div>
      <div className="flex gap-2 mb-6">
        {(['all', 'active', 'history'] as const).map(f => (
          <button key={f} onClick={() => setActiveFilter(f)}
            className={`px-4 py-2 rounded-lg text-sm font-medium transition-colors ${activeFilter === f ? 'bg-stellar-purple text-white' : 'bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-600'}`}>
            {f.charAt(0).toUpperCase() + f.slice(1)}
          </button>
        ))}
        <button onClick={load} className="ml-auto text-sm text-stellar-purple hover:underline">↻ Refresh</button>
      </div>

      {filtered.length === 0 ? (
        <div className="text-center py-12 text-gray-500">
          {activeFilter === 'active' ? 'No active sessions.' : 'No sessions found.'}
        </div>
      ) : (
        <div className="grid gap-4 md:grid-cols-2">
          {filtered.map(s => (
            <SessionCard key={s.id} session={s} onEnd={handleEnd} onExtend={handleExtend} />
          ))}
        </div>
      )}
    </div>
  )
}
