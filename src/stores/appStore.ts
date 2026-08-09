import { create } from 'zustand'
import type { NetworkSpeed, UserSettings, Page } from '@/types'

interface AppState {
  currentPage: Page
  setCurrentPage: (page: Page) => void

  networkSpeed: NetworkSpeed
  setNetworkSpeed: (speed: NetworkSpeed) => void

  settings: UserSettings | null
  setSettings: (settings: UserSettings) => void

  isDark: boolean
  setIsDark: (isDark: boolean) => void

  isMonitoring: boolean
  setIsMonitoring: (isMonitoring: boolean) => void

  speedHistory: Array<{ time: number; download: number; upload: number }>
  addToSpeedHistory: (download: number, upload: number) => void

  isWindowVisible: boolean
  setIsWindowVisible: (visible: boolean) => void
}

export const useAppStore = create<AppState>((set, get) => ({
  currentPage: 'dashboard',
  setCurrentPage: (page) => set({ currentPage: page }),

  networkSpeed: {
    downloadSpeed: 0,
    uploadSpeed: 0,
    totalDownload: 0,
    totalUpload: 0,
    adapterName: '',
  },
  setNetworkSpeed: (speed) => set({ networkSpeed: speed }),

  settings: null,
  setSettings: (settings) => set({ settings }),

  isDark: false,
  setIsDark: (isDark) => {
    set({ isDark })
    if (isDark) {
      document.documentElement.classList.add('dark')
    } else {
      document.documentElement.classList.remove('dark')
    }
  },

  isMonitoring: true,
  setIsMonitoring: (isMonitoring) => set({ isMonitoring }),

  speedHistory: [],
  addToSpeedHistory: (download, upload) => {
    const { speedHistory } = get()
    const now = Date.now()
    speedHistory.push({ time: now, download, upload })
    if (speedHistory.length > 60) {
      speedHistory.splice(0, speedHistory.length - 60)
    }
    set({ speedHistory: [...speedHistory] })
  },

  isWindowVisible: true,
  setIsWindowVisible: (visible) => set({ isWindowVisible: visible }),
}))
