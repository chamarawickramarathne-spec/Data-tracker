import { formatBytes } from '@/lib/utils'
import type { PeakHourEntry } from '@/types'

const DAYS = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat']
const HOURS = Array.from({ length: 24 }, (_, i) => i)

export function PeakHoursHeatmap({ data }: { data: PeakHourEntry[] }) {
  const maxBytes = Math.max(...data.map(d => d.totalBytes), 1)

  const grid: number[][] = Array.from({ length: 7 }, () => Array(24).fill(0))
  for (const cell of data) {
    grid[cell.dayOfWeek][cell.hour] = cell.totalBytes
  }

  const getColor = (bytes: number) => {
    const intensity = bytes / maxBytes
    if (intensity === 0) return 'bg-muted'
    if (intensity < 0.2) return 'bg-primary/10'
    if (intensity < 0.4) return 'bg-primary/25'
    if (intensity < 0.6) return 'bg-primary/40'
    if (intensity < 0.8) return 'bg-primary/60'
    return 'bg-primary/80'
  }

  return (
    <div className="overflow-x-auto">
      <div className="inline-flex flex-col gap-0.5">
        <div className="flex gap-0.5 ml-10">
          {HOURS.map(h => (
            <div key={h} className="w-7 text-center text-[10px] text-muted-foreground">
              {h % 3 === 0 ? `${h}:00` : ''}
            </div>
          ))}
        </div>
        {DAYS.map((day, di) => (
          <div key={day} className="flex items-center gap-0.5">
            <span className="w-9 text-right text-[10px] text-muted-foreground pr-1">{day}</span>
            {HOURS.map(h => (
              <div
                key={h}
                className={`w-7 h-5 rounded-sm ${getColor(grid[di][h])} transition-colors`}
                title={`${day} ${h}:00 — ${formatBytes(grid[di][h])}`}
              />
            ))}
          </div>
        ))}
        <div className="flex items-center gap-2 mt-2 ml-10">
          <span className="text-[10px] text-muted-foreground">Less</span>
          {[0, 0.1, 0.25, 0.4, 0.6, 0.8].map((v, i) => (
            <div
              key={i}
              className={`w-4 h-3 rounded-sm ${v === 0 ? 'bg-muted' : `bg-primary/${Math.round(v * 100)}`}`}
            />
          ))}
          <span className="text-[10px] text-muted-foreground">More</span>
        </div>
      </div>
    </div>
  )
}
