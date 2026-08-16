import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { formatBytes, getMonthName } from '@/lib/utils'
import { CalendarDays, ArrowDown, ArrowUp, TrendingUp, Trophy } from 'lucide-react'
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
import { AppDetailPanel } from './AppDetailPanel'
import { TopAppsChart } from './TopAppsChart'

interface MonthlyUsageData {
  year: number
  month: number
  uploadBytes: number
  downloadBytes: number
  totalBytes: number
}

interface DailyDataPoint {
  day: string
  download: number
  upload: number
}

interface AppUsageSummary {
  appName: string
  uploadBytes: number
  downloadBytes: number
  totalBytes: number
  percentage: number
}

export function MonthlyPage() {
  const now = new Date()
  const [selectedYear, setSelectedYear] = useState(now.getFullYear())
  const [selectedMonth, setSelectedMonth] = useState(now.getMonth() + 1)
  const [usage, setUsage] = useState<MonthlyUsageData | null>(null)
  const [appBreakdown, setAppBreakdown] = useState<AppUsageSummary[]>([])
  const [dailyBreakdown, setDailyBreakdown] = useState<DailyDataPoint[]>([])
  const [selectedApp, setSelectedApp] = useState<string | null>(null)

  useEffect(() => {
    setSelectedApp(null)
    loadMonthlyData()
  }, [selectedYear, selectedMonth])

  const loadMonthlyData = async () => {
    try {
      const [usageData, appData, dailyData] = await Promise.all([
        invoke<MonthlyUsageData>('get_monthly_usage', { year: selectedYear, month: selectedMonth }),
        invoke<AppUsageSummary[]>('get_monthly_app_breakdown', { year: selectedYear, month: selectedMonth }),
        invoke<{ day: number; uploadBytes: number; downloadBytes: number }[]>('get_daily_breakdown', { year: selectedYear, month: selectedMonth }),
      ])
      setUsage(usageData)
      setAppBreakdown(appData)

      const daysInMonth = new Date(selectedYear, selectedMonth, 0).getDate()
      const dailyMap = new Map(dailyData.map(d => [d.day, d]))
      const allDays = Array.from({ length: daysInMonth }, (_, i) => {
        const existing = dailyMap.get(i + 1)
        return {
          day: `${i + 1}`,
          download: existing?.downloadBytes ?? 0,
          upload: existing?.uploadBytes ?? 0,
        }
      })
      setDailyBreakdown(allDays)
    } catch (err) {
      console.error('Failed to load monthly data:', err)
    }
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-foreground">Monthly Usage</h1>
          <p className="text-sm text-muted-foreground mt-1">View data usage for a specific month</p>
        </div>
        <div className="flex items-center gap-2">
          <CalendarDays className="w-4 h-4 text-muted-foreground" />
          <select
            value={selectedMonth}
            onChange={(e) => setSelectedMonth(Number(e.target.value))}
            className="px-3 py-2 rounded-lg border border-border bg-card text-card-foreground text-sm focus:outline-none focus:ring-2 focus:ring-primary"
          >
            {Array.from({ length: 12 }, (_, i) => (
              <option key={i + 1} value={i + 1}>
                {getMonthName(i + 1)}
              </option>
            ))}
          </select>
          <select
            value={selectedYear}
            onChange={(e) => setSelectedYear(Number(e.target.value))}
            className="px-3 py-2 rounded-lg border border-border bg-card text-card-foreground text-sm focus:outline-none focus:ring-2 focus:ring-primary"
          >
            {Array.from({ length: 5 }, (_, i) => (
              <option key={now.getFullYear() - i} value={now.getFullYear() - i}>
                {now.getFullYear() - i}
              </option>
            ))}
          </select>
        </div>
      </div>

      {/* Summary Cards */}
      {usage && (
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          <SummaryCard
            title="Total Downloaded"
            value={formatBytes(usage.downloadBytes)}
            icon={<ArrowDown className="w-4 h-4" />}
            color="text-chart-download"
            bgColor="bg-chart-download/10"
          />
          <SummaryCard
            title="Total Uploaded"
            value={formatBytes(usage.uploadBytes)}
            icon={<ArrowUp className="w-4 h-4" />}
            color="text-chart-upload"
            bgColor="bg-chart-upload/10"
          />
          <SummaryCard
            title="Grand Total"
            value={formatBytes(usage.totalBytes)}
            icon={<TrendingUp className="w-4 h-4" />}
            color="text-chart-total"
            bgColor="bg-chart-total/10"
          />
        </div>
      )}

      {/* Daily Chart */}
      <div className="bg-card rounded-xl border border-border p-6">
        <h2 className="text-lg font-semibold text-card-foreground mb-4">Daily Breakdown</h2>
        <div className="h-[300px]">
          <ResponsiveContainer width="100%" height="100%">
            <BarChart data={dailyBreakdown} margin={{ top: 5, right: 10, left: 0, bottom: 5 }}>
              <CartesianGrid strokeDasharray="3 3" stroke="var(--color-border)" />
              <XAxis dataKey="day" tick={{ fontSize: 11, fill: 'var(--color-muted-foreground)' }} />
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

      {/* Top 5 Apps */}
      <div className="bg-card rounded-xl border border-border p-6">
        <div className="flex items-center gap-2 mb-4">
          <Trophy className="w-4 h-4 text-primary" />
          <h2 className="text-lg font-semibold text-card-foreground">Top 5 Apps This Month</h2>
        </div>
        <TopAppsChart apps={appBreakdown} />
      </div>

      {/* App Breakdown */}
      <div className="bg-card rounded-xl border border-border p-6">
        <h2 className="text-lg font-semibold text-card-foreground mb-4">Top Apps This Month</h2>
        {appBreakdown.length === 0 ? (
          <div className="text-center py-8 text-muted-foreground">
            <p>No application data for this month</p>
          </div>
        ) : (
          <div className="space-y-3">
            {appBreakdown.slice(0, 15).map((app) => (
              <div
                key={app.appName}
                onClick={() => setSelectedApp(selectedApp === app.appName ? null : app.appName)}
                className={`flex items-center gap-4 p-2 rounded-lg cursor-pointer hover:bg-muted/50 transition-colors ${
                  selectedApp === app.appName ? 'bg-primary/5' : ''
                }`}
              >
                <div className="w-8 h-8 rounded-lg bg-primary/10 flex items-center justify-center text-primary text-xs font-bold flex-shrink-0">
                  {app.appName.charAt(0).toUpperCase()}
                </div>
                <div className="flex-1 min-w-0">
                  <div className="flex items-center justify-between mb-1">
                    <span className="text-sm font-medium text-card-foreground truncate">{app.appName}</span>
                    <span className="text-sm text-muted-foreground ml-2 flex-shrink-0">{formatBytes(app.totalBytes)}</span>
                  </div>
                  <div className="flex gap-2">
                    <div className="flex-1 h-2 rounded-full bg-muted overflow-hidden">
                      <div
                        className="h-full rounded-full bg-chart-download"
                        style={{ width: `${app.percentage}%` }}
                      />
                    </div>
                    <span className="text-xs text-muted-foreground w-10 text-right">{app.percentage.toFixed(1)}%</span>
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}
        {selectedApp && (
          <AppDetailPanel
            appName={selectedApp}
            period="month"
            year={selectedYear}
            month={selectedMonth}
            onClose={() => setSelectedApp(null)}
          />
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
