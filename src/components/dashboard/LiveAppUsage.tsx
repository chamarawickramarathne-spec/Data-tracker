import { useEffect, useState } from 'react'
import { listen } from '@tauri-apps/api/event'
import { formatSpeed } from '@/lib/utils'
import { ArrowDown, ArrowUp, Cpu } from 'lucide-react'
import type { AppSpeedEntry } from '@/types'

export function LiveAppUsage() {
  const [apps, setApps] = useState<AppSpeedEntry[]>([])

  useEffect(() => {
    const unlisten = listen<AppSpeedEntry[]>('per-app-usage', (event) => {
      const merged = new Map<string, { downloadSpeed: number; uploadSpeed: number; totalSpeed: number }>()
      for (const entry of event.payload) {
        const existing = merged.get(entry.appName)
        if (existing) {
          existing.downloadSpeed += entry.downloadSpeed
          existing.uploadSpeed += entry.uploadSpeed
          existing.totalSpeed += entry.totalSpeed
        } else {
          merged.set(entry.appName, {
            downloadSpeed: entry.downloadSpeed,
            uploadSpeed: entry.uploadSpeed,
            totalSpeed: entry.totalSpeed,
          })
        }
      }
      const sorted = [...merged.entries()]
        .sort((a, b) => b[1].totalSpeed - a[1].totalSpeed)
        .slice(0, 8)
        .map(([appName, speeds]) => ({ appName, ...speeds }))
      setApps(sorted)
    })
    return () => { unlisten.then((fn) => fn()) }
  }, [])

  return (
    <div className="bg-card rounded-xl border border-border p-6 h-full">
      <div className="flex items-center gap-2 mb-3">
        <Cpu className="w-4 h-4 text-primary" />
        <h3 className="text-sm font-semibold text-card-foreground">Live App Usage</h3>
        <span className="text-xs text-muted-foreground ml-auto">Top 8 by speed</span>
      </div>
      {apps.length === 0 ? (
        <p className="text-xs text-muted-foreground py-4 text-center">No active app traffic detected</p>
      ) : (
        <div className="space-y-1.5">
          {apps.map((app) => (
            <div
              key={app.appName}
              className="flex items-center justify-between py-1.5 px-2 rounded-lg hover:bg-muted/50 transition-colors"
            >
              <span className="text-sm text-card-foreground truncate max-w-[180px]">
                {app.appName}
              </span>
              <div className="flex items-center gap-3 text-xs">
                <span className="flex items-center gap-1 text-chart-download">
                  <ArrowDown className="w-3 h-3" />
                  {formatSpeed(app.downloadSpeed)}
                </span>
                <span className="flex items-center gap-1 text-chart-upload">
                  <ArrowUp className="w-3 h-3" />
                  {formatSpeed(app.uploadSpeed)}
                </span>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  )
}
