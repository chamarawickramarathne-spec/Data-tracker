import { useEffect, useMemo, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { formatBytes } from '@/lib/utils'
import { ArrowDown, ArrowUp, X, BarChart3 } from 'lucide-react'
import {
  BarChart,
  Bar,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
  Legend,
} from 'recharts'

interface Point {
  uploadBytes: number
  downloadBytes: number
  totalBytes: number
}

interface AppDetailPanelProps {
  appName: string
  period: 'day' | 'month'
  date?: string
  year?: number
  month?: number
  onClose: () => void
}

export function AppDetailPanel({ appName, period, date, year, month, onClose }: AppDetailPanelProps) {
  const [points, setPoints] = useState<Point[]>([])
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    let cancelled = false
    setLoading(true)

    const load = async () => {
      try {
        if (period === 'day') {
          const data = await invoke<
            Array<{ hour: number; uploadBytes: number; downloadBytes: number; totalBytes: number }>
          >('get_app_hourly_breakdown', { appName, date })
          if (cancelled) return
          const map = new Map(data.map((d) => [d.hour, d]))
          setPoints(
            Array.from({ length: 24 }, (_, i) => {
              const existing = map.get(i)
              return {
                downloadBytes: existing?.downloadBytes ?? 0,
                uploadBytes: existing?.uploadBytes ?? 0,
                totalBytes: existing?.totalBytes ?? 0,
              }
            }),
          )
        } else {
          const data = await invoke<
            Array<{ day: number; uploadBytes: number; downloadBytes: number; totalBytes: number }>
          >('get_app_daily_breakdown_month', { appName, year, month })
          if (cancelled) return
          const yr = year ?? new Date().getFullYear()
          const mo = month ?? 1
          const daysInMonth = new Date(yr, mo, 0).getDate()
          const map = new Map(data.map((d) => [d.day, d]))
          setPoints(
            Array.from({ length: daysInMonth }, (_, i) => {
              const existing = map.get(i + 1)
              return {
                downloadBytes: existing?.downloadBytes ?? 0,
                uploadBytes: existing?.uploadBytes ?? 0,
                totalBytes: existing?.totalBytes ?? 0,
              }
            }),
          )
        }
      } catch (err) {
        console.error('Failed to load app detail:', err)
      } finally {
        if (!cancelled) setLoading(false)
      }
    }

    load()
    return () => {
      cancelled = true
    }
  }, [appName, period, date, year, month])

  const chartData = useMemo(() => {
    if (period === 'day') {
      return points.map((p, i) => ({
        label: `${String(i).padStart(2, '0')}:00`,
        download: p.downloadBytes,
        upload: p.uploadBytes,
      }))
    }
    return points.map((p, i) => ({ label: `${i + 1}`, download: p.downloadBytes, upload: p.uploadBytes }))
  }, [points, period])

  const totals = useMemo(
    () =>
      points.reduce(
        (acc, p) => ({
          downloadBytes: acc.downloadBytes + p.downloadBytes,
          uploadBytes: acc.uploadBytes + p.uploadBytes,
          totalBytes: acc.totalBytes + p.totalBytes,
        }),
        { downloadBytes: 0, uploadBytes: 0, totalBytes: 0 },
      ),
    [points],
  )

  return (
    <div className="bg-card rounded-xl border border-border p-6 mt-4">
      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center gap-3">
          <div className="w-8 h-8 rounded-lg bg-primary/10 flex items-center justify-center text-primary text-xs font-bold">
            {appName.charAt(0).toUpperCase()}
          </div>
          <div>
            <h3 className="text-base font-semibold text-card-foreground">{appName}</h3>
            <p className="text-xs text-muted-foreground">
              {period === 'day' ? 'Hourly usage' : 'Daily usage this month'}
            </p>
          </div>
        </div>
        <button
          onClick={onClose}
          className="p-1.5 rounded-lg text-muted-foreground hover:bg-muted hover:text-card-foreground transition-colors"
          title="Close details"
        >
          <X className="w-4 h-4" />
        </button>
      </div>

      <div className="flex flex-wrap items-center gap-3 mb-5">
        <div className="flex items-center gap-2 px-3 py-1.5 rounded-lg bg-chart-download/10 text-sm">
          <ArrowDown className="w-4 h-4 text-chart-download" />
          <span className="text-muted-foreground">Downloaded</span>
          <span className="font-semibold text-card-foreground">{formatBytes(totals.downloadBytes)}</span>
        </div>
        <div className="flex items-center gap-2 px-3 py-1.5 rounded-lg bg-chart-upload/10 text-sm">
          <ArrowUp className="w-4 h-4 text-chart-upload" />
          <span className="text-muted-foreground">Uploaded</span>
          <span className="font-semibold text-card-foreground">{formatBytes(totals.uploadBytes)}</span>
        </div>
        <div className="flex items-center gap-2 px-3 py-1.5 rounded-lg bg-chart-total/10 text-sm">
          <BarChart3 className="w-4 h-4 text-chart-total" />
          <span className="text-muted-foreground">Total</span>
          <span className="font-semibold text-card-foreground">{formatBytes(totals.totalBytes)}</span>
        </div>
      </div>

      <div className="h-[220px]">
        {loading ? (
          <div className="h-full flex items-center justify-center text-sm text-muted-foreground">
            Loading...
          </div>
        ) : (
          <ResponsiveContainer width="100%" height="100%">
            <BarChart data={chartData} margin={{ top: 5, right: 10, left: 0, bottom: 5 }}>
              <CartesianGrid strokeDasharray="3 3" stroke="var(--color-border)" vertical={false} />
              <XAxis
                dataKey="label"
                tick={{ fontSize: 11, fill: 'var(--color-muted-foreground)' }}
                interval="preserveStartEnd"
                minTickGap={20}
              />
              <YAxis tick={{ fontSize: 11, fill: 'var(--color-muted-foreground)' }} tickFormatter={(v) => formatBytes(v)} width={60} />
              <Tooltip
                contentStyle={{
                  backgroundColor: 'var(--color-card)',
                  border: '1px solid var(--color-border)',
                  borderRadius: '8px',
                  color: 'var(--color-card-foreground)',
                  fontSize: '12px',
                }}
                formatter={(value: number, name: string) => [formatBytes(value), name === 'download' ? 'Download' : 'Upload']}
              />
              <Legend />
              <Bar dataKey="download" fill="var(--color-chart-download)" radius={[4, 4, 0, 0]} name="Download" />
              <Bar dataKey="upload" fill="var(--color-chart-upload)" radius={[4, 4, 0, 0]} name="Upload" />
            </BarChart>
          </ResponsiveContainer>
        )}
      </div>
    </div>
  )
}
