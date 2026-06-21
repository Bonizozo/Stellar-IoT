'use client'

import { useState } from 'react'
import { DeviceRegistrationForm as FormData, DeviceCategory, ConnectivityType } from '@/types'
import { registerDevice } from '@/services/api'
import { validateDeviceRegistration } from '@/lib/deviceRegistration'

const CATEGORIES: DeviceCategory[] = ['sensor', 'camera', 'actuator', 'gateway', 'tracker', 'other']
const CONNECTIVITY: ConnectivityType[] = ['wifi', 'lora', 'zigbee', 'bluetooth', '4g', 'ethernet']

const EMPTY: FormData = {
  name: '', type: 'sensor', description: '', price: 0,
  location: '', connectivity: 'wifi', owner_address: '',
}

export default function DeviceRegistrationForm() {
  const [form, setForm] = useState<FormData>(EMPTY)
  const [errors, setErrors] = useState<Partial<Record<keyof FormData, string>>>({})
  const [preview, setPreview] = useState(false)
  const [submitting, setSubmitting] = useState(false)
  const [success, setSuccess] = useState<string | null>(null)
  const [apiError, setApiError] = useState<string | null>(null)

  const validate = (): boolean => {
    const e = validateDeviceRegistration(form)
    setErrors(e)
    return Object.keys(e).length === 0
  }

  const set = (field: keyof FormData) => (
    e: React.ChangeEvent<HTMLInputElement | HTMLSelectElement | HTMLTextAreaElement>
  ) => {
    setForm(f => ({ ...f, [field]: field === 'price' ? Number(e.target.value) : e.target.value }))
    setErrors(err => ({ ...err, [field]: undefined }))
  }

  const handleSubmit = async () => {
    setApiError(null)
    setSubmitting(true)
    try {
      const res = await registerDevice(form)
      setSuccess(`Device "${res.name}" registered successfully! ID: ${res.id}`)
      setForm(EMPTY)
      setPreview(false)
    } catch (err) {
      setApiError(err instanceof Error ? err.message : 'Registration failed')
    } finally {
      setSubmitting(false)
    }
  }

  const field = (label: string, key: keyof FormData, input: React.ReactNode) => (
    <div>
      <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">{label}</label>
      {input}
      {errors[key] && <p className="text-red-500 text-xs mt-1">{errors[key]}</p>}
    </div>
  )

  const inputCls = (key: keyof FormData) =>
    `w-full border rounded-lg px-3 py-2 bg-white dark:bg-gray-700 dark:text-white focus:outline-none focus:ring-2 focus:ring-stellar-purple ${errors[key] ? 'border-red-500' : 'border-gray-300 dark:border-gray-600'}`

  if (success) {
    return (
      <div className="bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800 rounded-lg p-6 text-center">
        <div className="text-4xl mb-3">✅</div>
        <p className="text-green-800 dark:text-green-300 font-medium">{success}</p>
        <button onClick={() => setSuccess(null)} className="mt-4 text-stellar-purple underline text-sm">
          Register another device
        </button>
      </div>
    )
  }

  if (preview) {
    return (
      <div className="space-y-6">
        <h2 className="text-xl font-semibold">Preview Registration</h2>
        <div className="bg-gray-50 dark:bg-gray-700 rounded-lg p-6 space-y-3 text-sm">
          {(Object.entries(form) as [keyof FormData, string | number][]).map(([k, v]) => (
            <div key={k} className="flex justify-between">
              <span className="text-gray-500 dark:text-gray-400 capitalize">{k.replace('_', ' ')}:</span>
              <span className="font-medium">{k === 'price' ? `${v} XLM` : String(v)}</span>
            </div>
          ))}
        </div>
        {apiError && <p className="text-red-500 text-sm">{apiError}</p>}
        <div className="flex gap-3">
          <button onClick={() => setPreview(false)} className="flex-1 border border-gray-300 dark:border-gray-600 py-2 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700">
            Edit
          </button>
          <button onClick={handleSubmit} disabled={submitting}
            className="flex-1 bg-stellar-purple text-white py-2 rounded-lg hover:bg-opacity-90 disabled:bg-gray-400 disabled:cursor-not-allowed">
            {submitting ? 'Submitting...' : 'Confirm & Register'}
          </button>
        </div>
      </div>
    )
  }

  return (
    <form onSubmit={e => { e.preventDefault(); if (validate()) setPreview(true) }} className="space-y-5">
      {field('Device Name', 'name',
        <input className={inputCls('name')} value={form.name} onChange={set('name')} placeholder="e.g. Rooftop Temperature Sensor" />
      )}

      <div className="grid grid-cols-2 gap-4">
        {field('Category', 'type',
          <select className={inputCls('type')} value={form.type} onChange={set('type')}>
            {CATEGORIES.map(c => <option key={c} value={c}>{c.charAt(0).toUpperCase() + c.slice(1)}</option>)}
          </select>
        )}
        {field('Connectivity', 'connectivity',
          <select className={inputCls('connectivity')} value={form.connectivity} onChange={set('connectivity')}>
            {CONNECTIVITY.map(c => <option key={c} value={c}>{c.toUpperCase()}</option>)}
          </select>
        )}
      </div>

      {field('Description', 'description',
        <textarea className={inputCls('description')} rows={3} value={form.description} onChange={set('description')} placeholder="Describe what this device does and its data output" />
      )}

      <div className="grid grid-cols-2 gap-4">
        {field('Price per Session (XLM)', 'price',
          <input type="number" min="0" step="0.01" className={inputCls('price')} value={form.price || ''} onChange={set('price')} placeholder="0.00" />
        )}
        {field('Location', 'location',
          <input className={inputCls('location')} value={form.location} onChange={set('location')} placeholder="e.g. New York, USA" />
        )}
      </div>

      {field('Owner Stellar Address', 'owner_address',
        <input className={inputCls('owner_address')} value={form.owner_address} onChange={set('owner_address')} placeholder="GXXXXXXX..." />
      )}

      <button type="submit" className="w-full bg-stellar-purple text-white py-3 rounded-lg font-semibold hover:bg-opacity-90 transition-colors">
        Preview Registration
      </button>
    </form>
  )
}
