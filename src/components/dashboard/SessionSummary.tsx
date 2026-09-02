import { useEffect, useState } from 'react'
import { listen } from '@tauri-apps/api/event'
import { formatBytes, formatSpeed } from '@/lib/utils'
import { Clock, ArrowDown, ArrowUp, TrendingUp, BarChart3 } from 'lucide-react'
import type { SessionStats } from '@/types'

function formatUptime(secs: number): string {
  const h = Math.floor(secs / 3600)
  const m = Math.floor((secs % 3600) / 60)
  const s = secs % 60
  if (h > 0) return `${h}h ${m}m`
  if (m > 0) return `${m}m ${s}s`
  return `${s}s`
}

export function SessionSummary() {
  const [stats, setStats] = useState<SessionStats | null>(null)

  useEffect(() => {
    const unlisten = listen<SessionStats>('session-stats', (event) => {
      setStats(event.payload)
    })
    return () => { unlisten.then((fn) => fn()) }
  }, [])

  if (!stats) return null

  return (
    <div className="bg-card rounded-xl border border-border p-5">
      <div className="flex items-center gap-2 mb-3">
        <BarChart3 className="w-4 h-4 text-primary" />
        <h3 className="text-sm font-semibold text-card-foreground">Session Summary</h3>
      </div>
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
        <StatItem
          icon={<Clock className="w-4 h-4" />}
          label="Uptime"
          value={formatUptime(stats.uptimeSeconds)}
          color="text-primary"
        />
        <StatItem
          icon={<TrendingUp className="w-4 h-4" />}
          label="Total Data"
          value={formatBytes(stats.totalDownload + stats.totalUpload)}
          color="text-chart-total"
        />
        <StatItem
          icon={<ArrowDown className="w-4 h-4" />}
          label="Peak Down"
          value={formatSpeed(stats.peakDownloadSpeed)}
          color="text-chart-download"
        />
        <StatItem
          icon={<ArrowUp className="w-4 h-4" />}
          label="Peak Up"
          value={formatSpeed(stats.peakUploadSpeed)}
          color="text-chart-upload"
        />
      </div>
      <div className="grid grid-cols-2 gap-4 mt-3 pt-3 border-t border-border">
        <AvgItem label="Avg Download" value={formatSpeed(stats.avgDownloadSpeed)} />
        <AvgItem label="Avg Upload" value={formatSpeed(stats.avgUploadSpeed)} />
      </div>
    </div>
  )
}

function StatItem({
  icon,
  label,
  value,
  color,
}: {
  icon: React.ReactNode
  label: string
  value: string
  color: string
}) {
  return (
    <div className="flex items-center gap-2">
      <div className={`${color}`}>{icon}</div>
      <div>
        <p className="text-xs text-muted-foreground">{label}</p>
        <p className="text-sm font-semibold text-card-foreground">{value}</p>
      </div>
    </div>
  )
}

function AvgItem({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex items-center justify-between">
      <span className="text-xs text-muted-foreground">{label}</span>
      <span className="text-xs font-medium text-card-foreground">{value}</span>
    </div>
  )
}
