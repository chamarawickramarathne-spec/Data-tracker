import { useEffect, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { Clock, TrendingUp } from 'lucide-react'
import { formatBytes } from '@/lib/utils'
import type { UsageForecast } from '@/types'

export function ForecastCard() {
  const [forecast, setForecast] = useState<UsageForecast | null>(null)

  useEffect(() => {
    invoke<UsageForecast>('get_usage_forecast')
      .then(setForecast)
      .catch(() => {})
  }, [])

  if (!forecast) return null

  const hasDaily = forecast.dailyLimitBytes > 0
  const hasMonthly = forecast.monthlyLimitBytes > 0

  if (!hasDaily && !hasMonthly) return null

  return (
    <div className="bg-card rounded-xl border border-border p-5">
      <div className="flex items-center gap-2 mb-3">
        <TrendingUp className="w-4 h-4 text-primary" />
        <h3 className="text-sm font-semibold text-card-foreground">Usage Forecast</h3>
      </div>
      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        {hasDaily && (
          <div className="space-y-2">
            <div className="flex items-center justify-between">
              <span className="text-xs text-muted-foreground">Daily Limit</span>
              <span className="text-xs font-medium text-card-foreground">
                {formatBytes(forecast.dailyUsedBytes)} / {formatBytes(forecast.dailyLimitBytes)}
              </span>
            </div>
            <div className="w-full bg-muted rounded-full h-2">
              <div
                className="h-2 rounded-full transition-all duration-500"
                style={{
                  width: `${Math.min((forecast.dailyUsedBytes / forecast.dailyLimitBytes) * 100, 100)}%`,
                  backgroundColor: forecast.dailyHoursRemaining !== null && forecast.dailyHoursRemaining < 24
                    ? 'var(--color-destructive)'
                    : 'var(--color-primary)',
                }}
              />
            </div>
            {forecast.dailyEstimatedHit && (
              <div className="flex items-center gap-1 text-xs text-muted-foreground">
                <Clock className="w-3 h-3" />
                <span>Limit hit at ~{forecast.dailyEstimatedHit}</span>
              </div>
            )}
            {forecast.dailyHoursRemaining !== null && forecast.dailyHoursRemaining > 0 && !forecast.dailyEstimatedHit && (
              <div className="flex items-center gap-1 text-xs text-muted-foreground">
                <Clock className="w-3 h-3" />
                <span>~{Math.round(forecast.dailyHoursRemaining)}h remaining at current rate</span>
              </div>
            )}
          </div>
        )}
        {hasMonthly && (
          <div className="space-y-2">
            <div className="flex items-center justify-between">
              <span className="text-xs text-muted-foreground">Monthly Limit</span>
              <span className="text-xs font-medium text-card-foreground">
                {formatBytes(forecast.monthlyUsedBytes)} / {formatBytes(forecast.monthlyLimitBytes)}
              </span>
            </div>
            <div className="w-full bg-muted rounded-full h-2">
              <div
                className="h-2 rounded-full transition-all duration-500"
                style={{
                  width: `${Math.min((forecast.monthlyUsedBytes / forecast.monthlyLimitBytes) * 100, 100)}%`,
                  backgroundColor: forecast.monthlyDaysRemaining !== null && forecast.monthlyDaysRemaining < 5
                    ? 'var(--color-destructive)'
                    : 'var(--color-primary)',
                }}
              />
            </div>
            {forecast.monthlyEstimatedHit && (
              <div className="flex items-center gap-1 text-xs text-muted-foreground">
                <Clock className="w-3 h-3" />
                <span>Limit hit ~{forecast.monthlyEstimatedHit}</span>
              </div>
            )}
            {forecast.monthlyDaysRemaining !== null && forecast.monthlyDaysRemaining > 0 && !forecast.monthlyEstimatedHit && (
              <div className="flex items-center gap-1 text-xs text-muted-foreground">
                <Clock className="w-3 h-3" />
                <span>~{Math.round(forecast.monthlyDaysRemaining)} days remaining</span>
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  )
}
