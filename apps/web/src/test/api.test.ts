import { describe, it, expect, vi, beforeEach } from 'vitest'
import { getDevices, makePayment } from '@/services/api'

const mockDevices = [
  {
    id: 'device-001',
    name: 'Smart Lock Alpha',
    description: 'High-security smart lock',
    price: 5.0,
    available: true,
    location: 'Building A',
  },
]

const mockPaymentResponse = {
  access_granted: true,
  session_id: 'sess-abc-123',
  expires_at: '2026-06-18T12:00:00Z',
}

beforeEach(() => {
  vi.resetAllMocks()
})

describe('getDevices', () => {
  it('returns device list on success', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn().mockResolvedValue({
        ok: true,
        json: async () => mockDevices,
      })
    )

    const devices = await getDevices()
    expect(devices).toEqual(mockDevices)
    expect(fetch).toHaveBeenCalledWith('http://localhost:8000/devices')
  })

  it('throws when response is not ok', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn().mockResolvedValue({ ok: false })
    )

    await expect(getDevices()).rejects.toThrow('Failed to fetch devices')
  })
})

describe('makePayment', () => {
  it('sends correct payload and returns response', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn().mockResolvedValue({
        ok: true,
        json: async () => mockPaymentResponse,
      })
    )

    const payload = { device_id: 'device-001', user_address: 'GABC123', amount: 5.0 }
    const result = await makePayment(payload)

    expect(result).toEqual(mockPaymentResponse)
    expect(fetch).toHaveBeenCalledWith(
      'http://localhost:8000/pay',
      expect.objectContaining({
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload),
      })
    )
  })

  it('throws when payment fails', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn().mockResolvedValue({ ok: false })
    )

    await expect(
      makePayment({ device_id: 'device-001', user_address: 'GABC', amount: 1 })
    ).rejects.toThrow('Payment failed')
  })
})
