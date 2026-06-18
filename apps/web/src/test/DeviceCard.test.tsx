import { describe, it, expect } from 'vitest'
import { render, screen } from '@testing-library/react'
import DeviceCard from '@/components/DeviceCard'
import { Device } from '@/types'

const mockDevice: Device = {
  id: 'device-001',
  name: 'Smart Lock Alpha',
  description: 'High-security smart lock for residential use',
  price: 5.0,
  available: true,
  location: 'Building A, Floor 3',
}

describe('DeviceCard', () => {
  it('renders device name', () => {
    render(<DeviceCard device={mockDevice} />)
    expect(screen.getByText('Smart Lock Alpha')).toBeInTheDocument()
  })

  it('renders device description', () => {
    render(<DeviceCard device={mockDevice} />)
    expect(screen.getByText('High-security smart lock for residential use')).toBeInTheDocument()
  })

  it('renders device price', () => {
    render(<DeviceCard device={mockDevice} />)
    expect(screen.getByText('5 XLM')).toBeInTheDocument()
  })

  it('renders device location', () => {
    render(<DeviceCard device={mockDevice} />)
    expect(screen.getByText('Building A, Floor 3')).toBeInTheDocument()
  })

  it('shows Available badge when device is available', () => {
    render(<DeviceCard device={mockDevice} />)
    expect(screen.getByText('Available')).toBeInTheDocument()
  })

  it('shows Unavailable badge when device is not available', () => {
    render(<DeviceCard device={{ ...mockDevice, available: false }} />)
    expect(screen.getByText('Unavailable')).toBeInTheDocument()
  })

  it('renders View Details link pointing to correct device page', () => {
    render(<DeviceCard device={mockDevice} />)
    const link = screen.getByRole('link', { name: /view details/i })
    expect(link).toHaveAttribute('href', '/device/device-001')
  })
})
