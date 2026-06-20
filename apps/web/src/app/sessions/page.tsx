import SessionManager from '@/components/SessionManager'

export default function SessionsPage() {
  return (
    <div className="container mx-auto px-4 py-8">
      <div className="max-w-4xl mx-auto">
        <h1 className="text-3xl font-bold mb-2">My Sessions</h1>
        <p className="text-gray-600 dark:text-gray-400 mb-8">
          Manage your active device sessions and view history.
        </p>
        <SessionManager />
      </div>
    </div>
  )
}
