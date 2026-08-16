import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { useAppStore } from '@/stores/appStore'
import { formatBytes } from '@/lib/utils'
import {
  Settings,
  Bell,
  BellOff,
  Power,
  Monitor,
  HardDrive,
  Shield,
  Save,
  RefreshCw,
} from 'lucide-react'

interface SettingsData {
  dailyLimitBytes: number
  monthlyLimitBytes: number
  warningThresholdPct: number
  dangerThresholdPct: number
  notificationsEnabled: boolean
  soundAlertsEnabled: boolean
  autoStartEnabled: boolean
  minimizeToTray: boolean
  theme: string
  dataRetentionDays: number
  selectedAdapter: string
  dailySummaryEnabled: boolean
  dailySummaryTime: string
}

export function SettingsPage() {
  const [settings, setSettings] = useState<SettingsData | null>(null)
  const [dailyLimit, setDailyLimit] = useState('')
  const [monthlyLimit, setMonthlyLimit] = useState('')
  const [warningThreshold, setWarningThreshold] = useState(80)
  const [dangerThreshold, setDangerThreshold] = useState(95)
  const [notifications, setNotifications] = useState(true)
  const [soundAlerts, setSoundAlerts] = useState(false)
  const [dailySummaryEnabled, setDailySummaryEnabled] = useState(false)
  const [dailySummaryTime, setDailySummaryTime] = useState('20:00')
  const [autoStart, setAutoStart] = useState(true)
  const [minimizeToTray, setMinimizeToTray] = useState(true)
  const [retentionDays, setRetentionDays] = useState(90)
  const [saving, setSaving] = useState(false)

  useEffect(() => {
    loadSettings()
  }, [])

  const loadSettings = async () => {
    try {
      const data = await invoke<SettingsData>('get_settings')
      setSettings(data)
      setDailyLimit(data.dailyLimitBytes > 0 ? String(data.dailyLimitBytes / (1024 * 1024 * 1024)) : '')
      setMonthlyLimit(data.monthlyLimitBytes > 0 ? String(data.monthlyLimitBytes / (1024 * 1024 * 1024)) : '')
      setWarningThreshold(data.warningThresholdPct)
      setDangerThreshold(data.dangerThresholdPct)
      setNotifications(data.notificationsEnabled)
      setSoundAlerts(data.soundAlertsEnabled)
      setDailySummaryEnabled(data.dailySummaryEnabled)
      setDailySummaryTime(data.dailySummaryTime)
      setAutoStart(data.autoStartEnabled)
      setMinimizeToTray(data.minimizeToTray)
      setRetentionDays(data.dataRetentionDays)
    } catch (err) {
      console.error('Failed to load settings:', err)
    }
  }

  const handleSave = async () => {
    setSaving(true)
    try {
      const dailyLimitBytes = dailyLimit ? parseFloat(dailyLimit) * 1024 * 1024 * 1024 : 0
      const monthlyLimitBytes = monthlyLimit ? parseFloat(monthlyLimit) * 1024 * 1024 * 1024 : 0

      await invoke('update_settings', {
        dailyLimitBytes: Math.floor(dailyLimitBytes),
        monthlyLimitBytes: Math.floor(monthlyLimitBytes),
        warningThresholdPct: warningThreshold,
        dangerThresholdPct: dangerThreshold,
        notificationsEnabled: notifications,
        soundAlertsEnabled: soundAlerts,
        dailySummaryEnabled: dailySummaryEnabled,
        dailySummaryTime: dailySummaryTime,
        autoStartEnabled: autoStart,
        minimizeToTray: minimizeToTray,
        dataRetentionDays: retentionDays,
      })
      await loadSettings()
    } catch (err) {
      console.error('Failed to save settings:', err)
    }
    setSaving(false)
  }

  return (
    <div className="space-y-6 max-w-3xl">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-foreground">Settings</h1>
          <p className="text-sm text-muted-foreground mt-1">Configure your data usage limits and preferences</p>
        </div>
        <button
          onClick={handleSave}
          disabled={saving}
          className="flex items-center gap-2 px-4 py-2 bg-primary text-primary-foreground rounded-lg hover:bg-primary/90 transition-colors disabled:opacity-50"
        >
          {saving ? <RefreshCw className="w-4 h-4 animate-spin" /> : <Save className="w-4 h-4" />}
          {saving ? 'Saving...' : 'Save Settings'}
        </button>
      </div>

      {/* Data Limits */}
      <div className="bg-card rounded-xl border border-border p-6">
        <div className="flex items-center gap-2 mb-4">
          <HardDrive className="w-5 h-5 text-primary" />
          <h2 className="text-lg font-semibold text-card-foreground">Data Limits</h2>
        </div>
        <div className="space-y-4">
          <div>
            <label className="block text-sm font-medium text-card-foreground mb-1.5">
              Daily Limit (GB)
            </label>
            <input
              type="number"
              value={dailyLimit}
              onChange={(e) => setDailyLimit(e.target.value)}
              placeholder="0 = No limit"
              min="0"
              step="0.1"
              className="w-full px-3 py-2 rounded-lg border border-border bg-background text-foreground text-sm focus:outline-none focus:ring-2 focus:ring-primary"
            />
          </div>
          <div>
            <label className="block text-sm font-medium text-card-foreground mb-1.5">
              Monthly Limit (GB)
            </label>
            <input
              type="number"
              value={monthlyLimit}
              onChange={(e) => setMonthlyLimit(e.target.value)}
              placeholder="0 = No limit"
              min="0"
              step="0.1"
              className="w-full px-3 py-2 rounded-lg border border-border bg-background text-foreground text-sm focus:outline-none focus:ring-2 focus:ring-primary"
            />
          </div>
          <div className="grid grid-cols-2 gap-4">
            <div>
              <label className="block text-sm font-medium text-card-foreground mb-1.5">
                Warning at: {warningThreshold}%
              </label>
              <input
                type="range"
                min="50"
                max="99"
                value={warningThreshold}
                onChange={(e) => setWarningThreshold(Number(e.target.value))}
                className="w-full accent-primary"
              />
            </div>
            <div>
              <label className="block text-sm font-medium text-card-foreground mb-1.5">
                Danger at: {dangerThreshold}%
              </label>
              <input
                type="range"
                min="50"
                max="100"
                value={dangerThreshold}
                onChange={(e) => setDangerThreshold(Number(e.target.value))}
                className="w-full accent-destructive"
              />
            </div>
          </div>
        </div>
      </div>

      {/* Notifications */}
      <div className="bg-card rounded-xl border border-border p-6">
        <div className="flex items-center gap-2 mb-4">
          {notifications ? <Bell className="w-5 h-5 text-primary" /> : <BellOff className="w-5 h-5 text-muted-foreground" />}
          <h2 className="text-lg font-semibold text-card-foreground">Notifications</h2>
        </div>
        <div className="space-y-4">
          <ToggleSetting
            label="Enable Notifications"
            description="Get notified when data limits are reached"
            value={notifications}
            onChange={setNotifications}
          />
          <ToggleSetting
            label="Sound Alerts"
            description="Play a sound when limits are reached"
            value={soundAlerts}
            onChange={setSoundAlerts}
          />
          <ToggleSetting
            label="Daily Usage Summary"
            description="Receive a summary notification of today's usage"
            value={dailySummaryEnabled}
            onChange={setDailySummaryEnabled}
          />
          {dailySummaryEnabled && (
            <div>
              <label className="block text-sm font-medium text-card-foreground mb-1.5">
                Summary Time
              </label>
              <input
                type="time"
                value={dailySummaryTime}
                onChange={(e) => setDailySummaryTime(e.target.value)}
                className="w-full px-3 py-2 rounded-lg border border-border bg-background text-foreground text-sm focus:outline-none focus:ring-2 focus:ring-primary"
              />
            </div>
          )}
        </div>
      </div>

      {/* Startup */}
      <div className="bg-card rounded-xl border border-border p-6">
        <div className="flex items-center gap-2 mb-4">
          <Power className="w-5 h-5 text-primary" />
          <h2 className="text-lg font-semibold text-card-foreground">Startup</h2>
        </div>
        <div className="space-y-4">
          <ToggleSetting
            label="Auto-start with Windows"
            description="Launch Data Tracker when your computer starts"
            value={autoStart}
            onChange={setAutoStart}
          />
          <ToggleSetting
            label="Minimize to Tray"
            description="Keep running in system tray when window is closed"
            value={minimizeToTray}
            onChange={setMinimizeToTray}
          />
        </div>
      </div>

      {/* Data */}
      <div className="bg-card rounded-xl border border-border p-6">
        <div className="flex items-center gap-2 mb-4">
          <Settings className="w-5 h-5 text-primary" />
          <h2 className="text-lg font-semibold text-card-foreground">Data Management</h2>
        </div>
        <div className="space-y-4">
          <div>
            <label className="block text-sm font-medium text-card-foreground mb-1.5">
              Data Retention: {retentionDays} days
            </label>
            <input
              type="range"
              min="7"
              max="365"
              value={retentionDays}
              onChange={(e) => setRetentionDays(Number(e.target.value))}
              className="w-full accent-primary"
            />
            <p className="text-xs text-muted-foreground mt-1">
              Data older than {retentionDays} days will be automatically deleted
            </p>
          </div>
        </div>
      </div>

      {/* Theme */}
      <div className="bg-card rounded-xl border border-border p-6">
        <div className="flex items-center gap-2 mb-4">
          <Monitor className="w-5 h-5 text-primary" />
          <h2 className="text-lg font-semibold text-card-foreground">Appearance</h2>
        </div>
        <div>
          <p className="text-sm text-muted-foreground">
            Theme follows your Windows system settings automatically.
          </p>
        </div>
      </div>
    </div>
  )
}

function ToggleSetting({
  label,
  description,
  value,
  onChange,
}: {
  label: string
  description: string
  value: boolean
  onChange: (value: boolean) => void
}) {
  return (
    <div className="flex items-center justify-between">
      <div>
        <p className="text-sm font-medium text-card-foreground">{label}</p>
        <p className="text-xs text-muted-foreground">{description}</p>
      </div>
      <button
        onClick={() => onChange(!value)}
        className={`
          relative w-11 h-6 rounded-full transition-colors
          ${value ? 'bg-primary' : 'bg-muted'}
        `}
      >
        <div
          className={`
            absolute top-0.5 w-5 h-5 rounded-full bg-white shadow transition-transform
            ${value ? 'translate-x-5.5' : 'translate-x-0.5'}
          `}
        />
      </button>
    </div>
  )
}
