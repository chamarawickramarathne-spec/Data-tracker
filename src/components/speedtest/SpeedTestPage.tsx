import { useState, useEffect, useCallback } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { Play, Loader2 } from 'lucide-react'
import type { SpeedTestResult, SpeedTestProgress } from '@/types'
import { SpeedTestGauge } from './SpeedTestGauge'

export function SpeedTestPage() {
  const [result, setResult] = useState<SpeedTestResult | null>(null)
  const [phase, setPhase] = useState<'idle' | 'download' | 'upload' | 'latency' | 'done'>('idle')
  const [progress, setProgress] = useState(0)

  useEffect(() => {
    const unlisten = listen<SpeedTestProgress>('speedtest-progress', (event) => {
      const p = event.payload
      setProgress(p.progress)
      if (p.phase === 'download') setPhase('download')
      else if (p.phase === 'upload') setPhase('upload')
      else if (p.phase === 'done') setPhase('done')
    })
    return () => { unlisten.then(fn => fn()) }
  }, [])

  const runTest = useCallback(async () => {
    setResult(null)
    setPhase('latency')
    setProgress(0)
    try {
      const res = await invoke<SpeedTestResult>('run_speed_test')
      setResult(res)
      setPhase('done')
    } catch (e) {
      setPhase('idle')
    }
  }, [])

  const isRunning = phase !== 'idle' && phase !== 'done'

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold text-foreground">Speed Test</h1>
        <p className="text-sm text-muted-foreground mt-1">Test your network connection speed</p>
      </div>

      <div className="bg-card rounded-xl border border-border p-8">
        <div className="flex flex-col items-center gap-8">
          <div className="grid grid-cols-3 gap-12">
            <SpeedTestGauge
              value={result?.downloadMbps ?? 0}
              label="Download"
              unit="Mbps"
              phase={phase === 'download' ? 'download' : result ? 'done' : 'idle'}
            />
            <SpeedTestGauge
              value={result?.uploadMbps ?? 0}
              label="Upload"
              unit="Mbps"
              phase={phase === 'upload' ? 'upload' : result ? 'done' : 'idle'}
            />
            <SpeedTestGauge
              value={result?.latencyMs ?? 0}
              label="Latency"
              unit="ms"
              phase={phase === 'latency' ? 'latency' : result ? 'done' : 'idle'}
            />
          </div>

          {isRunning && (
            <div className="w-64">
              <div className="flex justify-between text-xs text-muted-foreground mb-1">
                <span className="capitalize">{phase}...</span>
                <span>{Math.round(progress)}%</span>
              </div>
              <div className="w-full bg-muted rounded-full h-2">
                <div
                  className="h-2 rounded-full bg-primary transition-all duration-300"
                  style={{ width: `${progress}%` }}
                />
              </div>
            </div>
          )}

          <button
            onClick={runTest}
            disabled={isRunning}
            className="flex items-center gap-2 px-6 py-3 bg-primary text-primary-foreground rounded-lg font-medium hover:bg-primary/90 transition-colors disabled:opacity-50 disabled:cursor-default"
          >
            {isRunning ? (
              <Loader2 className="w-4 h-4 animate-spin" />
            ) : (
              <Play className="w-4 h-4" />
            )}
            {isRunning ? 'Testing...' : 'Start Test'}
          </button>

          {result && (
            <div className="text-xs text-muted-foreground">
              Server: {result.server}
            </div>
          )}
        </div>
      </div>
    </div>
  )
}
