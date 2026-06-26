export interface Device {
  id: string
  name: string
  description: string
  price: number
  available: boolean
  location: string
}

export interface PaymentRequest {
  device_id: string
  user_address: string
  amount: number
  tx_hash?: string
}

export interface PaymentResponse {
  access_granted: boolean
  session_id: string
  expires_at: string
}

export interface Session {
  id: string
  device_id: string
  device_name: string
  user_address: string
  created_at: string
  expires_at: string
  active: boolean
}

export type DeviceCategory = 'sensor' | 'camera' | 'actuator' | 'gateway' | 'tracker' | 'other'
export type ConnectivityType = 'wifi' | 'lora' | 'zigbee' | 'bluetooth' | '4g' | 'ethernet'

export interface DeviceRegistrationForm {
  name: string
  type: DeviceCategory
  description: string
  price: number
  location: string
  connectivity: ConnectivityType
  owner_address: string
}

export interface DeviceRegistrationResponse {
  id: string
  name: string
  message: string
}

// ─── Analytics ───────────────────────────────────────────────────────────────

export type ReportPeriod = 'daily' | 'weekly' | 'monthly'

export interface TimeSeriesPoint {
  date: string
  revenue: number
  session_count: number
  unique_users: number
}

export interface PeakHour {
  hour: number
  session_count: number
}

export interface RetentionRow {
  cohort: string
  new_users: number
  returning_users: number
  retention_rate: number
}

export interface DeviceAnalyticsReport {
  device_id: string
  period: ReportPeriod
  total_revenue: number
  total_sessions: number
  total_unique_users: number
  avg_session_duration_secs: number
  time_series: TimeSeriesPoint[]
  peak_hours: PeakHour[]
  retention: RetentionRow[]
}

// ─── Earnings / Owner Dashboard ───────────────────────────────────────────────

export interface OwnerEarningsResponse {
  total_earnings_xlm: number
  pending_earnings_xlm: number
  total_devices: number
  total_sessions: number
  period: string
  time_series: TimeSeriesPoint[]
  top_devices: TopDevice[]
  uptime_avg: number
}

export interface TopDevice {
  id: string
  name: string
  earnings: number
  sessions: number
  uptime_pct: number
}

export interface OwnerDeviceStatus {
  id: string
  name: string
  online: boolean
  uptime_pct: number
  last_seen: string | null
  total_sessions: number
  total_earnings: number
  price: number
}

export interface WithdrawalRequest {
  owner_address: string
  amount: number
  destination_address: string
}

export interface WithdrawalResponse {
  success: boolean
  tx_hash: string
  amount: number
  fee: number
  message: string
}
