import { useAppStore } from '@/stores/appStore'
import {
  AreaChart,
  Area,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
} from 'recharts'

export function SpeedChart() {
  const { speedHistory } = useAppStore()

  const data = speedHistory.map((entry) => ({
    time: new Date(entry.time).toLocaleTimeString('en-US', { hour: '2-digit', minute: '2-digit', second: '2-digit' }),
    download: entry.download,
    upload: entry.upload,
    downloadMB: Number((entry.download / (1024 * 1024)).toFixed(2)),
    uploadMB: Number((entry.upload / (1024 * 1024)).toFixed(2)),
  }))

  return (
    <div className="h-[300px]">
      <ResponsiveContainer width="100%" height="100%">
        <AreaChart data={data} margin={{ top: 5, right: 10, left: 0, bottom: 5 }}>
          <defs>
            <linearGradient id="downloadGrad" x1="0" y1="0" x2="0" y2="1">
              <stop offset="0%" stopColor="var(--color-chart-download)" stopOpacity={0.3} />
              <stop offset="100%" stopColor="var(--color-chart-download)" stopOpacity={0} />
            </linearGradient>
            <linearGradient id="uploadGrad" x1="0" y1="0" x2="0" y2="1">
              <stop offset="0%" stopColor="var(--color-chart-upload)" stopOpacity={0.3} />
              <stop offset="100%" stopColor="var(--color-chart-upload)" stopOpacity={0} />
            </linearGradient>
          </defs>
          <CartesianGrid strokeDasharray="3 3" stroke="var(--color-border)" />
          <XAxis
            dataKey="time"
            tick={{ fontSize: 11, fill: 'var(--color-muted-foreground)' }}
            interval="preserveStartEnd"
          />
          <YAxis
            tick={{ fontSize: 11, fill: 'var(--color-muted-foreground)' }}
            tickFormatter={(v) => `${v} MB/s`}
          />
          <Tooltip
            contentStyle={{
              backgroundColor: 'var(--color-card)',
              border: '1px solid var(--color-border)',
              borderRadius: '8px',
              color: 'var(--color-card-foreground)',
              fontSize: '12px',
            }}
            formatter={(value: number, name: string) => [
              `${value} MB/s`,
              name === 'downloadMB' ? 'Download' : 'Upload',
            ]}
          />
          <Area
            type="monotone"
            dataKey="downloadMB"
            stroke="var(--color-chart-download)"
            fill="url(#downloadGrad)"
            strokeWidth={2}
          />
          <Area
            type="monotone"
            dataKey="uploadMB"
            stroke="var(--color-chart-upload)"
            fill="url(#uploadGrad)"
            strokeWidth={2}
          />
        </AreaChart>
      </ResponsiveContainer>
    </div>
  )
}
