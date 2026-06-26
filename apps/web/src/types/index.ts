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

// ─── Device Management (CRUD) ──────────────────────────────────────────────────

export interface ManagedDevice {
  id: string
  name: string
  type: DeviceCategory
  description: string
  price: number
  available: boolean
  location: string
  connectivity: string
  owner_address: string
  rating: number
  popularity: number
  latitude: number
  longitude: number
  created_at: string
  updated_at: string
}

export interface DeviceUpdateForm {
  name?: string
  description?: string
  price?: number
  location?: string
  available?: boolean
  connectivity?: ConnectivityType
  type?: DeviceCategory
}

// ─── Payment History ───────────────────────────────────────────────────────────

export type PaymentStatus = 'active' | 'expired' | 'ended'

export interface PaymentHistoryEntry {
  id: string
  tx_hash: string
  device_id: string
  device_name: string
  user_address: string
  amount: number
  session_id: string
  created_at: string
  expires_at: string
  status: PaymentStatus
  duration_secs: number
}

export interface PaymentHistoryResponse {
  data: PaymentHistoryEntry[]
  total_spent: number
  total_sessions: number
  total_duration_secs: number
}

export interface PaymentHistoryFilters {
  device_id?: string
  status?: PaymentStatus
  from?: string
  to?: string
}

// ─── QR Code Scan Analytics ──────────────────────────────────────────────────────

export interface QrScanDailyPoint {
  date: string
  scans: number
}

export interface QrScanAnalytics {
  device_id: string
  total_scans: number
  last_scan: string | null
  daily: QrScanDailyPoint[]
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
