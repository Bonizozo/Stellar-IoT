import EarningsDashboard from '@/components/EarningsDashboard'

export default function EarningsPage() {
  return (
    <main className="max-w-5xl mx-auto px-4 py-8">
      <div className="mb-6">
        <h1 className="text-2xl font-bold">Device Owner Earnings</h1>
        <p className="text-sm text-gray-500 mt-1">
          Track your revenue, device performance, and manage withdrawals
        </p>
      </div>
      <EarningsDashboard />
    </main>
  )
}
