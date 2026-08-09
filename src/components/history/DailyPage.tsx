import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { formatBytes, getTodayString } from '@/lib/utils'
import { Calendar, ArrowDown, ArrowUp, TrendingUp } from 'lucide-react'
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

interface DailyUsageData {
  date: string
  uploadBytes: number
  downloadBytes: number
  totalBytes: number
  peakUploadSpeed: number
  peakDownloadSpeed: number
}

interface AppUsageSummary {
  appName: string
  uploadBytes: number
  downloadBytes: number
  totalBytes: number
  percentage: number
}

interface HourlyDataPoint {
  hour: string
  download: number
  upload: number
}

export function DailyPage() {
  const [selectedDate, setSelectedDate] = useState(getTodayString())
  const [usage, setUsage] = useState<DailyUsageData | null>(null)
  const [appBreakdown, setAppBreakdown] = useState<AppUsageSummary[]>([])
  const [hourlyBreakdown, setHourlyBreakdown] = useState<HourlyDataPoint[]>([])

  useEffect(() => {
    loadDailyData()
  }, [selectedDate])

  const loadDailyData = async () => {
    try {
      const [usageData, appData, hourlyData] = await Promise.all([
        invoke<DailyUsageData>('get_daily_usage', { date: selectedDate }),
        invoke<AppUsageSummary[]>('get_daily_app_breakdown', { date: selectedDate }),
        invoke<{ hour: number; uploadBytes: number; downloadBytes: number }[]>('get_hourly_breakdown', { date: selectedDate }),
      ])
      setUsage(usageData)
      setAppBreakdown(appData)

      const hourlyMap = new Map(hourlyData.map(h => [h.hour, h]))
      const allHours = Array.from({ length: 24 }, (_, i) => {
        const existing = hourlyMap.get(i)
        return {
          hour: `${String(i).padStart(2, '0')}:00`,
          download: existing?.downloadBytes ?? 0,
          upload: existing?.uploadBytes ?? 0,
        }
      })
      setHourlyBreakdown(allHours)
    } catch (err) {
      console.error('Failed to load daily data:', err)
    }
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-foreground">Daily Usage</h1>
          <p className="text-sm text-muted-foreground mt-1">View data usage for a specific day</p>
        </div>
        <div className="flex items-center gap-2">
          <Calendar className="w-4 h-4 text-muted-foreground" />
          <input
            type="date"
            value={selectedDate}
            onChange={(e) => setSelectedDate(e.target.value)}
            className="px-3 py-2 rounded-lg border border-border bg-card text-card-foreground text-sm focus:outline-none focus:ring-2 focus:ring-primary"
          />
        </div>
      </div>

      {/* Summary Cards */}
      {usage && (
        <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
          <SummaryCard
            title="Downloaded"
            value={formatBytes(usage.downloadBytes)}
            icon={<ArrowDown className="w-4 h-4" />}
            color="text-chart-download"
            bgColor="bg-chart-download/10"
          />
          <SummaryCard
            title="Uploaded"
            value={formatBytes(usage.uploadBytes)}
            icon={<ArrowUp className="w-4 h-4" />}
            color="text-chart-upload"
            bgColor="bg-chart-upload/10"
          />
          <SummaryCard
            title="Total"
            value={formatBytes(usage.totalBytes)}
            icon={<TrendingUp className="w-4 h-4" />}
            color="text-chart-total"
            bgColor="bg-chart-total/10"
          />
          <SummaryCard
            title="Peak Speed"
            value={`${formatBytes(usage.peakDownloadSpeed)}/s`}
            icon={<TrendingUp className="w-4 h-4" />}
            color="text-warning"
            bgColor="bg-warning/10"
          />
        </div>
      )}

      {/* Hourly Chart */}
      <div className="bg-card rounded-xl border border-border p-6">
        <h2 className="text-lg font-semibold text-card-foreground mb-4">Hourly Breakdown</h2>
        <div className="h-[300px]">
          <ResponsiveContainer width="100%" height="100%">
            <BarChart data={hourlyBreakdown} margin={{ top: 5, right: 10, left: 0, bottom: 5 }}>
              <CartesianGrid strokeDasharray="3 3" stroke="var(--color-border)" />
              <XAxis dataKey="hour" tick={{ fontSize: 11, fill: 'var(--color-muted-foreground)' }} />
              <YAxis tick={{ fontSize: 11, fill: 'var(--color-muted-foreground)' }} tickFormatter={(v) => formatBytes(v)} />
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
        </div>
      </div>

      {/* App Breakdown */}
      <div className="bg-card rounded-xl border border-border p-6">
        <h2 className="text-lg font-semibold text-card-foreground mb-4">Usage by Application</h2>
        {appBreakdown.length === 0 ? (
          <div className="text-center py-8 text-muted-foreground">
            <p>No application data for this date</p>
          </div>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full">
              <thead>
                <tr className="border-b border-border">
                  <th className="text-left py-3 px-4 text-xs font-medium text-muted-foreground uppercase">Application</th>
                  <th className="text-right py-3 px-4 text-xs font-medium text-muted-foreground uppercase">Download</th>
                  <th className="text-right py-3 px-4 text-xs font-medium text-muted-foreground uppercase">Upload</th>
                  <th className="text-right py-3 px-4 text-xs font-medium text-muted-foreground uppercase">Total</th>
                  <th className="text-right py-3 px-4 text-xs font-medium text-muted-foreground uppercase">Share</th>
                </tr>
              </thead>
              <tbody>
                {appBreakdown.map((app) => (
                  <tr key={app.appName} className="border-b border-border/50 hover:bg-muted/50 transition-colors">
                    <td className="py-3 px-4">
                      <div className="flex items-center gap-3">
                        <div className="w-8 h-8 rounded-lg bg-primary/10 flex items-center justify-center text-primary text-xs font-bold">
                          {app.appName.charAt(0).toUpperCase()}
                        </div>
                        <span className="text-sm font-medium text-card-foreground">{app.appName}</span>
                      </div>
                    </td>
                    <td className="py-3 px-4 text-right text-sm text-chart-download">{formatBytes(app.downloadBytes)}</td>
                    <td className="py-3 px-4 text-right text-sm text-chart-upload">{formatBytes(app.uploadBytes)}</td>
                    <td className="py-3 px-4 text-right text-sm font-medium text-card-foreground">{formatBytes(app.totalBytes)}</td>
                    <td className="py-3 px-4 text-right">
                      <div className="flex items-center justify-end gap-2">
                        <div className="w-16 h-2 rounded-full bg-muted overflow-hidden">
                          <div
                            className="h-full rounded-full bg-primary"
                            style={{ width: `${app.percentage}%` }}
                          />
                        </div>
                        <span className="text-xs text-muted-foreground w-10 text-right">{app.percentage.toFixed(1)}%</span>
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>
    </div>
  )
}

function SummaryCard({
  title,
  value,
  icon,
  color,
  bgColor,
}: {
  title: string
  value: string
  icon: React.ReactNode
  color: string
  bgColor: string
}) {
  return (
    <div className="bg-card rounded-xl border border-border p-4">
      <div className="flex items-center gap-2 mb-2">
        <div className={`w-8 h-8 rounded-lg ${bgColor} flex items-center justify-center ${color}`}>
          {icon}
        </div>
        <span className="text-xs text-muted-foreground">{title}</span>
      </div>
      <p className="text-xl font-bold text-card-foreground">{value}</p>
    </div>
  )
}
