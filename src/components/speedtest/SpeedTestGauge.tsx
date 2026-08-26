import { Gauge } from 'lucide-react'

export function SpeedTestGauge({
  value,
  label,
  unit,
  phase,
}: {
  value: number
  label: string
  unit: string
  phase: 'idle' | 'download' | 'upload' | 'latency' | 'done'
}) {
  const getColor = () => {
    if (phase === 'idle') return 'text-muted-foreground'
    if (value < 50) return 'text-green-500'
    if (value < 200) return 'text-yellow-500'
    return 'text-red-500'
  }

  return (
    <div className="flex flex-col items-center gap-2">
      <div className="relative">
        <svg width="120" height="120" viewBox="0 0 120 120">
          <circle
            cx="60" cy="60" r="50"
            fill="none"
            stroke="hsl(var(--muted))"
            strokeWidth="8"
          />
          <circle
            cx="60" cy="60" r="50"
            fill="none"
            stroke="currentColor"
            strokeWidth="8"
            strokeLinecap="round"
            strokeDasharray={`${2 * Math.PI * 50}`}
            strokeDashoffset={`${2 * Math.PI * 50 * (1 - Math.min(value / 500, 1))}`}
            transform="rotate(-90 60 60)"
            className={`${getColor()} transition-all duration-300`}
          />
        </svg>
        <div className="absolute inset-0 flex flex-col items-center justify-center">
          <span className={`text-2xl font-bold ${getColor()}`}>
            {phase === 'idle' ? '--' : value.toFixed(1)}
          </span>
          <span className="text-xs text-muted-foreground">{unit}</span>
        </div>
      </div>
      <div className="flex items-center gap-1">
        <Gauge className="w-3 h-3 text-muted-foreground" />
        <span className="text-sm text-muted-foreground">{label}</span>
      </div>
    </div>
  )
}
