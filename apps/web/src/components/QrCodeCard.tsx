'use client'

import { useEffect, useState } from 'react'
import QRCode from 'qrcode'
import { Printer, Download } from 'lucide-react'

interface QrDevice {
  id: string
  name: string
  price: number
  location: string
}

interface Props {
  device: QrDevice
  /** Custom brand name shown on the printable card. */
  brandText?: string
  /** Custom tagline shown under the QR code. */
  tagline?: string
  /** Accent colour (hex) used for the QR modules and card header. */
  accentColor?: string
}

/**
 * Renders a unique, printable QR card for a device. The QR encodes a deep link
 * to the device's payment page (`/device/:id?src=qr`) so scans are attributed
 * to the QR channel for analytics.
 */
export default function QrCodeCard({
  device,
  brandText = 'Stellar IoT',
  tagline = 'Scan to pay & unlock access',
  accentColor = '#7c3aed',
}: Props) {
  const [qrDataUrl, setQrDataUrl] = useState('')
  const [link, setLink] = useState('')

  useEffect(() => {
    const origin = typeof window !== 'undefined' ? window.location.origin : ''
    const url = `${origin}/device/${device.id}?src=qr`
    setLink(url)
    QRCode.toDataURL(url, {
      width: 320,
      margin: 1,
      color: { dark: accentColor, light: '#ffffff' },
    })
      .then(setQrDataUrl)
      .catch(() => setQrDataUrl(''))
  }, [device.id, accentColor])

  const handleDownload = () => {
    if (!qrDataUrl) return
    const a = document.createElement('a')
    a.href = qrDataUrl
    a.download = `qr-${device.id}.png`
    document.body.appendChild(a)
    a.click()
    a.remove()
  }

  const handlePrint = () => {
    if (!qrDataUrl) return
    const win = window.open('', '_blank', 'width=480,height=680')
    if (!win) return
    win.document.write(`
      <html>
        <head>
          <title>QR Card — ${device.name}</title>
          <style>
            * { box-sizing: border-box; font-family: -apple-system, Segoe UI, Roboto, sans-serif; }
            body { margin: 0; display: flex; justify-content: center; padding: 24px; }
            .card { width: 360px; border: 2px solid ${accentColor}; border-radius: 18px; overflow: hidden; }
            .head { background: ${accentColor}; color: #fff; padding: 16px; text-align: center; }
            .head h1 { margin: 0; font-size: 20px; }
            .body { padding: 24px; text-align: center; }
            .body h2 { margin: 0 0 4px; font-size: 18px; }
            .meta { color: #555; font-size: 13px; margin-bottom: 16px; }
            .price { color: ${accentColor}; font-weight: 700; }
            img { width: 280px; height: 280px; }
            .tag { margin-top: 12px; font-size: 13px; color: #333; }
            .url { margin-top: 8px; font-size: 10px; color: #999; word-break: break-all; }
          </style>
        </head>
        <body>
          <div class="card">
            <div class="head"><h1>${brandText}</h1></div>
            <div class="body">
              <h2>${device.name}</h2>
              <div class="meta">${device.location} · <span class="price">${device.price} XLM</span></div>
              <img src="${qrDataUrl}" alt="QR code" />
              <div class="tag">${tagline}</div>
              <div class="url">${link}</div>
            </div>
          </div>
          <script>window.onload = function(){ window.print(); }</script>
        </body>
      </html>
    `)
    win.document.close()
  }

  return (
    <div>
      {/* On-screen card preview */}
      <div className="rounded-2xl overflow-hidden border-2 max-w-xs mx-auto" style={{ borderColor: accentColor }}>
        <div className="text-white text-center py-4" style={{ backgroundColor: accentColor }}>
          <h3 className="font-bold text-lg">{brandText}</h3>
        </div>
        <div className="bg-white p-6 text-center">
          <h4 className="font-semibold text-gray-900">{device.name}</h4>
          <p className="text-sm text-gray-500 mb-4">
            {device.location} · <span className="font-semibold" style={{ color: accentColor }}>{device.price} XLM</span>
          </p>
          {qrDataUrl ? (
            // eslint-disable-next-line @next/next/no-img-element
            <img src={qrDataUrl} alt={`QR code for ${device.name}`} className="w-56 h-56 mx-auto" />
          ) : (
            <div className="w-56 h-56 mx-auto flex items-center justify-center text-gray-400 text-sm">Generating…</div>
          )}
          <p className="text-sm text-gray-700 mt-3">{tagline}</p>
        </div>
      </div>

      {/* Actions */}
      <div className="flex justify-center gap-3 mt-5">
        <button onClick={handlePrint} disabled={!qrDataUrl}
          className="bg-stellar-purple text-white px-4 py-2 rounded-lg text-sm font-medium hover:bg-opacity-90 disabled:bg-gray-400 transition flex items-center gap-2">
          <Printer size={16} /> Print Card
        </button>
        <button onClick={handleDownload} disabled={!qrDataUrl}
          className="border border-stellar-purple text-stellar-purple px-4 py-2 rounded-lg text-sm font-medium hover:bg-stellar-purple/5 disabled:opacity-50 transition flex items-center gap-2">
          <Download size={16} /> Download PNG
        </button>
      </div>
    </div>
  )
}
