import { formatBytes } from '@/lib/utils'
import {
  BarChart,
  Bar,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
} from 'recharts'

interface AppUsageSummary {
  appName: string
  uploadBytes: number
  downloadBytes: number
  totalBytes: number
  percentage: number
}

export function TopAppsChart({ apps }: { apps: AppUsageSummary[] }) {
  const data = apps.slice(0, 5).map((a) => ({ name: a.appName, total: a.totalBytes }))

  if (data.length === 0) {
    return <div className="text-center py-8 text-muted-foreground text-sm">No application data yet</div>
  }

  return (
    <div className="h-[220px]">
      <ResponsiveContainer width="100%" height="100%">
        <BarChart data={data} layout="vertical" margin={{ top: 5, right: 10, left: 0, bottom: 5 }}>
          <CartesianGrid strokeDasharray="3 3" stroke="var(--color-border)" horizontal={false} />
          <XAxis
            type="number"
            tick={{ fontSize: 11, fill: 'var(--color-muted-foreground)' }}
            tickFormatter={(v) => formatBytes(v)}
          />
          <YAxis
            type="category"
            dataKey="name"
            width={140}
            tick={{ fontSize: 11, fill: 'var(--color-muted-foreground)' }}
          />
          <Tooltip
            contentStyle={{
              backgroundColor: 'var(--color-card)',
              border: '1px solid var(--color-border)',
              borderRadius: '8px',
              color: 'var(--color-card-foreground)',
              fontSize: '12px',
            }}
            formatter={(value: number) => [formatBytes(value), 'Total']}
          />
          <Bar dataKey="total" fill="var(--color-chart-total)" radius={[0, 4, 4, 0]} barSize={14} />
        </BarChart>
      </ResponsiveContainer>
    </div>
  )
}
