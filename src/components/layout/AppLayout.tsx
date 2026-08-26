import { useEffect } from 'react'
import { listen } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { useAppStore } from '@/stores/appStore'
import type { UserSettings } from '@/types'
import { Sidebar } from './Sidebar'
import { Titlebar } from './Titlebar'
import { Dashboard } from '@/components/dashboard/Dashboard'
import { DailyPage } from '@/components/history/DailyPage'
import { MonthlyPage } from '@/components/history/MonthlyPage'
import { SettingsPage } from '@/components/settings/SettingsPage'
import { SpeedTestPage } from '@/components/speedtest/SpeedTestPage'
import { PeakHoursPage } from '@/components/history/PeakHoursPage'

export function AppLayout() {
  const { currentPage, setNetworkSpeed, setIsDark, setSettings, addToSpeedHistory, setIsWindowVisible } = useAppStore()

  useEffect(() => {
    // Load initial theme
    invoke<{ theme: string; isDark: boolean }>('get_system_theme')
      .then((themeInfo) => {
        setIsDark(themeInfo.isDark)
        if (themeInfo.isDark) {
          document.documentElement.classList.add('dark')
        } else {
          document.documentElement.classList.remove('dark')
        }
      })
      .catch(() => {})

    // Load settings
    invoke<UserSettings>('get_settings')
      .then((settings) => setSettings(settings as any))
      .catch(() => {})

    // Track window visibility
    const appWindow = getCurrentWindow()
    const unlistenFocus = appWindow.onResized(async () => {
      const visible = await appWindow.isVisible()
      setIsWindowVisible(visible)
    })

    const unlistenTauriFocus = listen('tauri://focus', () => {
      setIsWindowVisible(true)
    })

    // Listen for network speed updates - skip if window hidden to tray
    const unlistenSpeed = listen<{
      downloadSpeed: number
      uploadSpeed: number
      totalDownload: number
      totalUpload: number
      adapterName: string
    }>('network-speed', (event) => {
      const { isWindowVisible } = useAppStore.getState()
      if (!isWindowVisible) return

      setNetworkSpeed(event.payload)
      addToSpeedHistory(event.payload.downloadSpeed, event.payload.uploadSpeed)
    })

    return () => {
      unlistenSpeed.then((fn) => fn())
      unlistenTauriFocus.then((fn) => fn())
      unlistenFocus.then((fn) => fn())
    }
  }, [])

  const renderPage = () => {
    switch (currentPage) {
      case 'dashboard':
        return <Dashboard />
      case 'daily':
        return <DailyPage />
      case 'monthly':
        return <MonthlyPage />
      case 'settings':
        return <SettingsPage />
      case 'speedtest':
        return <SpeedTestPage />
      case 'peakhours':
        return <PeakHoursPage />
      default:
        return <Dashboard />
    }
  }

  return (
    <div className="h-screen flex flex-col bg-background">
      <Titlebar />
      <div className="flex flex-1 overflow-hidden">
        <Sidebar />
        <main className="flex-1 overflow-auto p-6">
          {renderPage()}
        </main>
      </div>
    </div>
  )
}
