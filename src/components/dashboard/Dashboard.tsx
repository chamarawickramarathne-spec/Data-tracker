import { useEffect, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { getVersion } from '@tauri-apps/api/app'
import { useAppStore } from '@/stores/appStore'
import { formatBytes, formatSpeed } from '@/lib/utils'
import { ArrowDown, ArrowUp, Activity, Zap, RefreshCw, Download, Check } from 'lucide-react'
import { SpeedChart } from './SpeedChart'

type UpdateStatus =
  | { state: 'idle' }
  | { state: 'checking' }
  | { state: 'uptodate' }
  | { state: 'downloading'; version: string }
  | { state: 'restarting' }
  | { state: 'failed'; message: string }

export function Dashboard() {
  const { networkSpeed } = useAppStore()
  const [version, setVersion] = useState('1.0.0')
  const [update, setUpdate] = useState<UpdateStatus>({ state: 'idle' })

  useEffect(() => {
    getVersion().then(setVersion).catch(() => {})
  }, [])

  const checkForUpdates = async () => {
    if (update.state === 'checking' || update.state === 'downloading' || update.state === 'restarting') return
    setUpdate({ state: 'checking' })
    try {
      const info = await invoke<{ current: string; latest: string | null }>('check_for_updates')
      setVersion(info.current)
      if (info.latest) {
        setUpdate({ state: 'downloading', version: info.latest })
        await invoke('apply_update', { repo: 'chamarawickramarathne-spec/Data-tracker', version: info.latest })
        setUpdate({ state: 'restarting' })
      } else {
        setUpdate({ state: 'uptodate' })
        setTimeout(() => setUpdate({ state: 'idle' }), 3000)
      }
    } catch (e) {
      const message = typeof e === 'string' ? e : e instanceof Error ? e.message : String(e)
      setUpdate({ state: 'failed', message })
      setTimeout(() => setUpdate({ state: 'idle' }), 5000)
    }
  }

  const buttonLabel = (() => {
    switch (update.state) {
      case 'checking':
        return 'Checking...'
      case 'uptodate':
        return 'Up to date'
      case 'downloading':
        return `Downloading v${update.version}...`
      case 'restarting':
        return 'Restarting...'
      case 'failed':
        return 'Check failed'
      default:
        return 'Check for updates'
    }
  })()

  const buttonIcon = (() => {
    switch (update.state) {
      case 'checking':
        return <RefreshCw className="w-3.5 h-3.5 animate-spin" />
      case 'downloading':
      case 'restarting':
        return <Download className="w-3.5 h-3.5" />
      case 'uptodate':
        return <Check className="w-3.5 h-3.5" />
      default:
        return <RefreshCw className="w-3.5 h-3.5" />
    }
  })()

  return (
    <div className="space-y-6">
      <div>
        <div className="flex items-center gap-3">
          <h1 className="text-2xl font-bold text-foreground">Live Dashboard</h1>
          <span className="text-xs font-medium text-muted-foreground bg-muted px-2 py-0.5 rounded-full">
            v{version}
          </span>
          <button
            onClick={checkForUpdates}
            disabled={update.state === 'checking' || update.state === 'downloading' || update.state === 'restarting'}
            title={update.state === 'failed' ? update.message : undefined}
            className="flex items-center gap-1.5 text-xs font-medium text-primary border border-primary/30 rounded-full px-2.5 py-1 hover:bg-primary/10 transition-colors disabled:opacity-60 disabled:cursor-default"
          >
            {buttonIcon}
            {buttonLabel}
          </button>
        </div>
        <p className="text-sm text-muted-foreground mt-1">Real-time network monitoring</p>
      </div>

      {/* Speed Cards */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        <SpeedCard
          title="Download"
          icon={<ArrowDown className="w-5 h-5" />}
          speed={networkSpeed.downloadSpeed}
          total={networkSpeed.totalDownload}
          color="text-chart-download"
          bgColor="bg-chart-download/10"
        />
        <SpeedCard
          title="Upload"
          icon={<ArrowUp className="w-5 h-5" />}
          speed={networkSpeed.uploadSpeed}
          total={networkSpeed.totalUpload}
          color="text-chart-upload"
          bgColor="bg-chart-upload/10"
        />
        <SpeedCard
          title="Total"
          icon={<Activity className="w-5 h-5" />}
          speed={networkSpeed.downloadSpeed + networkSpeed.uploadSpeed}
          total={networkSpeed.totalDownload + networkSpeed.totalUpload}
          color="text-chart-total"
          bgColor="bg-chart-total/10"
        />
      </div>

      {/* Speed Chart */}
      <div className="bg-card rounded-xl border border-border p-6">
        <div className="flex items-center gap-2 mb-4">
          <Zap className="w-4 h-4 text-primary" />
          <h2 className="text-lg font-semibold text-card-foreground">Speed History</h2>
          <span className="text-xs text-muted-foreground ml-auto">Last 5 minutes</span>
        </div>
        <SpeedChart />
      </div>


    </div>
  )
}

function SpeedCard({
  title,
  icon,
  speed,
  total,
  color,
  bgColor,
}: {
  title: string
  icon: React.ReactNode
  speed: number
  total: number
  color: string
  bgColor: string
}) {
  return (
    <div className="bg-card rounded-xl border border-border p-5">
      <div className="flex items-center gap-3">
        <div className={`w-10 h-10 rounded-lg ${bgColor} flex items-center justify-center ${color}`}>
          {icon}
        </div>
        <div>
          <p className="text-sm text-muted-foreground">{title}</p>
          <p className="text-xl font-bold text-card-foreground">{formatSpeed(speed)}</p>
        </div>
      </div>
      <div className="mt-3 pt-3 border-t border-border">
        <p className="text-xs text-muted-foreground">
          Total: <span className="text-card-foreground font-medium">{formatBytes(total)}</span>
        </p>
      </div>
    </div>
  )
}
