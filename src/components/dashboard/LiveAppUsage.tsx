import { useEffect, useState } from 'react'
import { listen } from '@tauri-apps/api/event'
import { formatSpeed } from '@/lib/utils'
import { ArrowDown, ArrowUp, Cpu } from 'lucide-react'
import type { AppSpeedEntry } from '@/types'

export function LiveAppUsage() {
  const [apps, setApps] = useState<AppSpeedEntry[]>([])

  useEffect(() => {
    const unlisten = listen<AppSpeedEntry[]>('per-app-usage', (event) => {
      const sorted = [...event.payload].sort((a, b) => b.totalSpeed - a.totalSpeed)
      setApps(sorted.slice(0, 10))
    })
    return () => { unlisten.then((fn) => fn()) }
  }, [])

  return (
    <div className="bg-card rounded-xl border border-border p-5">
      <div className="flex items-center gap-2 mb-3">
        <Cpu className="w-4 h-4 text-primary" />
        <h3 className="text-sm font-semibold text-card-foreground">Live App Usage</h3>
        <span className="text-xs text-muted-foreground ml-auto">Top 10 by speed</span>
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
