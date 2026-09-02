import { getCurrentWindow } from '@tauri-apps/api/window'
import { useAppStore } from '@/stores/appStore'
import { Minus, Square, X, Wifi } from 'lucide-react'

export function Titlebar() {
  const { networkSpeed, setIsWindowVisible, settings } = useAppStore()

  const appWindow = getCurrentWindow()

  const minimize = async () => {
    try { await appWindow.minimize() } catch {}
  }
  const maximize = async () => {
    try { const isMaximized = await appWindow.isMaximized()
      if (isMaximized) await appWindow.unmaximize()
      else await appWindow.maximize()
    } catch {}
  }
  const close = async () => {
    try {
      if (settings?.minimizeToTray !== false) {
        setIsWindowVisible(false)
        await appWindow.hide()
      } else {
        await appWindow.close()
      }
    } catch {}
  }

  return (
    <div
      data-tauri-drag-region
      className="flex items-center justify-between h-10 px-4 bg-card border-b border-border select-none"
    >
      <div className="flex items-center gap-2" data-tauri-drag-region>
        <Wifi className="w-4 h-4 text-primary" />
        <span className="text-sm font-semibold text-card-foreground">Data Tracker</span>
        <span className="text-xs text-muted-foreground ml-2">
          {networkSpeed.adapterName || 'No adapter'}
        </span>
      </div>

      <div className="flex items-center gap-1">
        <button
          onClick={minimize}
          className="w-8 h-8 flex items-center justify-center rounded hover:bg-muted transition-colors"
        >
          <Minus className="w-4 h-4" />
        </button>
        <button
          onClick={maximize}
          className="w-8 h-8 flex items-center justify-center rounded hover:bg-muted transition-colors"
        >
          <Square className="w-3 h-3" />
        </button>
        <button
          onClick={close}
          className="w-8 h-8 flex items-center justify-center rounded hover:bg-destructive hover:text-destructive-foreground transition-colors"
        >
          <X className="w-4 h-4" />
        </button>
      </div>
    </div>
  )
}
