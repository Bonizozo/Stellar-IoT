'use client'

import { useEffect, useState } from 'react'
import dynamic from 'next/dynamic'
import { Device } from '@/types'
import { getDevices } from '@/services/api'
import PayButton from './PayButton'
import { Loader2, MapPin, Navigation } from 'lucide-react'
import { Input } from '@/components/ui/input'
import { Card } from '@/components/ui/card'

// Dynamically import Leaflet components to avoid SSR issues
const MapContainer = dynamic(() => import('react-leaflet').then(mod => mod.MapContainer), { ssr: false })
const TileLayer = dynamic(() => import('react-leaflet').then(mod => mod.TileLayer), { ssr: false })
const Marker = dynamic(() => import('react-leaflet').then(mod => mod.Marker), { ssr: false })
const Popup = dynamic(() => import('react-leaflet').then(mod => mod.Popup), { ssr: false })

interface DeviceWithCoords extends Device {
  latitude?: number
  longitude?: number
}

export function MapView() {
  const [devices, setDevices] = useState<DeviceWithCoords[]>([])
  const [loading, setLoading] = useState(true)
  const [userLocation, setUserLocation] = useState<{ lat: number; lng: number } | null>(null)
  const [radiusKm, setRadiusKm] = useState(10)
  const [mapCenter, setMapCenter] = useState<[number, number]>([37.7749, -122.4194])

  useEffect(() => {
    async function loadDevices() {
      try {
        const data = await getDevices()
        // Filter devices that have coordinates
        const withCoords = data.filter((d: Device) => (d as any).latitude && (d as any).longitude) as DeviceWithCoords[]
        setDevices(withCoords)
      } catch (err) {
        console.error('Failed to load devices:', err)
      } finally {
        setLoading(false)
      }
    }
    loadDevices()
  }, [])

  const handleGetMyLocation = () => {
    if (!navigator.geolocation) {
      alert('Geolocation is not supported by your browser')
      return
    }
    navigator.geolocation.getCurrentPosition(
      (pos) => {
        const { latitude, longitude } = pos.coords
        setUserLocation({ lat: latitude, lng: longitude })
        setMapCenter([latitude, longitude])
      },
      (err) => {
        console.error('Geolocation error:', err)
        alert('Unable to retrieve your location')
      }
    )
  }

  // Filter devices by proximity if user location and radius are set
  const visibleDevices = userLocation
    ? devices.filter((d) => {
        if (!d.latitude || !d.longitude) return false
        const dist = haversineKm(userLocation.lat, userLocation.lng, d.latitude, d.longitude)
        return dist <= radiusKm
      })
    : devices

  if (loading) {
    return (
      <div className="flex items-center justify-center h-96">
        <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
      </div>
    )
  }

  return (
    <div className="space-y-4">
      {/* Controls */}
      <Card className="p-4 flex flex-wrap items-center gap-4">
        <button
          onClick={handleGetMyLocation}
          className="inline-flex items-center gap-2 rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90"
        >
          <Navigation className="h-4 w-4" />
          Use My Location
        </button>

        {userLocation && (
          <div className="flex items-center gap-2">
            <label htmlFor="radius" className="text-sm font-medium">
              Radius:
            </label>
            <Input
              id="radius"
              type="number"
              min={1}
              max={100}
              value={radiusKm}
              onChange={(e: React.ChangeEvent<HTMLInputElement>) => setRadiusKm(Number(e.target.value))}
              className="w-20"
            />
            <span className="text-sm text-muted-foreground">km</span>
          </div>
        )}

        <div className="ml-auto text-sm text-muted-foreground">
          {visibleDevices.length} device{visibleDevices.length !== 1 ? 's' : ''} shown
        </div>
      </Card>

      {/* Map */}
      <div className="h-[600px] rounded-lg overflow-hidden border">
        {typeof window !== 'undefined' && (
          <MapContainer center={mapCenter} zoom={13} className="h-full w-full">
            <TileLayer
              attribution='&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors'
              url="https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png"
            />

            {visibleDevices.map((device) => (
              <Marker
                key={device.id}
                position={[device.latitude!, device.longitude!]}
                eventHandlers={{
                  click: () => {
                    // Optional: handle marker click
                  },
                }}
              >
                <Popup>
                  <div className="space-y-2 min-w-[200px]">
                    <div className="flex items-center gap-2">
                      <MapPin className="h-4 w-4 text-primary" />
                      <h3 className="font-semibold">{device.name}</h3>
                    </div>
                    <p className="text-sm text-muted-foreground">{device.description}</p>
                    <div className="text-sm">
                      <span className="font-medium">Location:</span> {device.location}
                    </div>
                    <div className="text-sm">
                      <span className="font-medium">Price:</span> {device.price} XLM
                    </div>
                    <div className="text-sm">
                      <span className="font-medium">Status:</span>{' '}
                      <span className={device.available ? 'text-green-600' : 'text-red-600'}>
                        {device.available ? 'Available' : 'Unavailable'}
                      </span>
                    </div>
                    {device.available && (
                      <PayButton device={device} />
                    )}
                  </div>
                </Popup>
              </Marker>
            ))}
          </MapContainer>
        )}
      </div>
    </div>
  )
}

// Haversine formula for distance calculation (km)
function haversineKm(lat1: number, lng1: number, lat2: number, lng2: number): number {
  const R = 6371 // Earth's radius in km
  const dLat = ((lat2 - lat1) * Math.PI) / 180
  const dLng = ((lng2 - lng1) * Math.PI) / 180
  const a =
    Math.sin(dLat / 2) * Math.sin(dLat / 2) +
    Math.cos((lat1 * Math.PI) / 180) *
      Math.cos((lat2 * Math.PI) / 180) *
      Math.sin(dLng / 2) *
      Math.sin(dLng / 2)
  const c = 2 * Math.atan2(Math.sqrt(a), Math.sqrt(1 - a))
  return R * c
}