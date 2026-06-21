'use client'

import { useCallback, useEffect, useState } from 'react'
import { useParams } from 'next/navigation'
import PayButton from '@/components/PayButton'
import { getDevices } from '@/services/api'
import { Device } from '@/types'

export default function DevicePage() {
  const params = useParams()
  const [device, setDevice] = useState<Device | null>(null)
  const [loading, setLoading] = useState(true)
  const deviceId = Array.isArray(params.id) ? params.id[0] : params.id

  const loadDevice = useCallback(async () => {
    if (!deviceId) {
      setDevice(null)
      setLoading(false)
      return
    }

    try {
      const devices = await getDevices()
      const found = devices.find((d) => d.id === deviceId)
      setDevice(found || null)
    } catch (error) {
      console.error('Failed to load device:', error)
    } finally {
      setLoading(false)
    }
  }, [deviceId])

  useEffect(() => {
    loadDevice()
  }, [loadDevice])

  if (loading) return <div className="container mx-auto px-4 py-8">Loading...</div>
  if (!device) return <div className="container mx-auto px-4 py-8">Device not found</div>

  return (
    <div className="container mx-auto px-4 py-8">
      <div className="max-w-2xl mx-auto bg-white dark:bg-gray-800 rounded-lg shadow-lg p-8">
        <h1 className="text-3xl font-bold mb-4">{device.name}</h1>
        <p className="text-gray-600 dark:text-gray-400 mb-6">{device.description}</p>
        
        <div className="mb-6">
          <div className="flex justify-between items-center mb-2">
            <span className="text-gray-700 dark:text-gray-300">Price:</span>
            <span className="text-2xl font-bold">{device.price} XLM</span>
          </div>
          <div className="flex justify-between items-center mb-2">
            <span className="text-gray-700 dark:text-gray-300">Status:</span>
            <span className={`px-3 py-1 rounded ${device.available ? 'bg-green-100 text-green-800' : 'bg-red-100 text-red-800'}`}>
              {device.available ? 'Available' : 'Unavailable'}
            </span>
          </div>
          <div className="flex justify-between items-center">
            <span className="text-gray-700 dark:text-gray-300">Location:</span>
            <span>{device.location}</span>
          </div>
        </div>

        <PayButton device={device} />
      </div>
    </div>
  )
}
