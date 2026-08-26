import { useEffect, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { Grid3x3, Clock, Sunrise } from 'lucide-react'
import type { PeakHourEntry } from '@/types'
import { formatBytes } from '@/lib/utils'
import { PeakHoursHeatmap } from './PeakHoursHeatmap'

const MONTH_NAMES = ['January', 'February', 'March', 'April', 'May', 'June',
  'July', 'August', 'September', 'October', 'November', 'December']

export function PeakHoursPage() {
  const now = new Date()
  const [year, setYear] = useState(now.getFullYear())
  const [month, setMonth] = useState(now.getMonth() + 1)
  const [data, setData] = useState<PeakHourEntry[]>([])

  useEffect(() => {
    invoke<PeakHourEntry[]>('get_peak_hours_heatmap', { year, month })
      .then(setData)
      .catch(() => setData([]))
  }, [year, month])

  const peakEntry = data.reduce((max, d) => d.totalBytes > max.totalBytes ? d : max, { dayOfWeek: 0, hour: 0, totalBytes: 0 })
  const quietEntry = data.filter(d => d.totalBytes > 0).reduce((min, d) => d.totalBytes < min.totalBytes ? d : min, { dayOfWeek: 0, hour: 0, totalBytes: Infinity })

  const peakHour = `${peakEntry.hour}:00`
  const quietHour = quietEntry.totalBytes < Infinity ? `${quietEntry.hour}:00` : '--'

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-foreground">Peak Hours</h1>
          <p className="text-sm text-muted-foreground mt-1">Data usage by hour and day of week</p>
        </div>
        <div className="flex items-center gap-3">
          <select
            value={month}
            onChange={(e) => setMonth(Number(e.target.value))}
            className="bg-muted border border-border rounded-lg px-3 py-1.5 text-sm text-foreground"
          >
            {MONTH_NAMES.map((name, i) => (
              <option key={i} value={i + 1}>{name}</option>
            ))}
          </select>
          <select
            value={year}
            onChange={(e) => setYear(Number(e.target.value))}
            className="bg-muted border border-border rounded-lg px-3 py-1.5 text-sm text-foreground"
          >
            {Array.from({ length: 6 }, (_, i) => now.getFullYear() - i).map(y => (
              <option key={y} value={y}>{y}</option>
            ))}
          </select>
        </div>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        <div className="bg-card rounded-xl border border-border p-4 flex items-center gap-3">
          <div className="w-10 h-10 rounded-lg bg-chart-upload/10 flex items-center justify-center text-chart-upload">
            <Clock className="w-5 h-5" />
          </div>
          <div>
            <p className="text-xs text-muted-foreground">Peak Hour</p>
            <p className="text-lg font-bold text-card-foreground">{peakHour}</p>
          </div>
        </div>
        <div className="bg-card rounded-xl border border-border p-4 flex items-center gap-3">
          <div className="w-10 h-10 rounded-lg bg-chart-download/10 flex items-center justify-center text-chart-download">
            <Sunrise className="w-5 h-5" />
          </div>
          <div>
            <p className="text-xs text-muted-foreground">Quietest Hour</p>
            <p className="text-lg font-bold text-card-foreground">{quietHour}</p>
          </div>
        </div>
      </div>

      <div className="bg-card rounded-xl border border-border p-6">
        <div className="flex items-center gap-2 mb-4">
          <Grid3x3 className="w-4 h-4 text-primary" />
          <h2 className="text-lg font-semibold text-card-foreground">Usage Heatmap</h2>
        </div>
        {data.length > 0 ? (
          <PeakHoursHeatmap data={data} />
        ) : (
          <p className="text-sm text-muted-foreground text-center py-8">No data for this month yet</p>
        )}
      </div>
    </div>
  )
}
