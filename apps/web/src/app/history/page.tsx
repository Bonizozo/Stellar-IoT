import PaymentHistory from '@/components/PaymentHistory'

export default function HistoryPage() {
  return (
    <div className="container mx-auto px-4 py-8">
      <div className="max-w-5xl mx-auto">
        <h1 className="text-3xl font-bold mb-2">Payment History</h1>
        <p className="text-gray-600 dark:text-gray-400 mb-8">
          Review every device payment, follow each transaction on the Stellar block explorer, and export your records.
        </p>
        <PaymentHistory />
      </div>
    </div>
  )
}
