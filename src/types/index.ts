export interface NetworkSpeed {
  downloadSpeed: number;
  uploadSpeed: number;
  totalDownload: number;
  totalUpload: number;
  adapterName: string;
}

export interface NetworkAdapter {
  name: string;
  description: string;
  isConnected: boolean;
  speed: number;
}

export interface DailyUsage {
  date: string;
  uploadBytes: number;
  downloadBytes: number;
  totalBytes: number;
  peakUploadSpeed: number;
  peakDownloadSpeed: number;
}

export interface HourlyUsage {
  hour: number;
  uploadBytes: number;
  downloadBytes: number;
  totalBytes: number;
}

export interface AppUsageSummary {
  appName: string;
  uploadBytes: number;
  downloadBytes: number;
  totalBytes: number;
  percentage: number;
}

export interface MonthlyUsage {
  year: number;
  month: number;
  uploadBytes: number;
  downloadBytes: number;
  totalBytes: number;
}

export interface DailyCalendarEntry {
  date: string;
  totalBytes: number;
}

export interface UserSettings {
  dailyLimitBytes: number;
  monthlyLimitBytes: number;
  warningThresholdPct: number;
  dangerThresholdPct: number;
  notificationsEnabled: boolean;
  soundAlertsEnabled: boolean;
  autoStartEnabled: boolean;
  minimizeToTray: boolean;
  theme: string;
  dataRetentionDays: number;
  selectedAdapter: string;
  dailySummaryEnabled: boolean;
  dailySummaryTime: string;
}

export interface ThemeInfo {
  theme: string;
  isDark: boolean;
}

export interface UsageForecast {
  dailyLimitBytes: number;
  dailyUsedBytes: number;
  dailyRatePerHour: number;
  dailyHoursRemaining: number | null;
  dailyEstimatedHit: string | null;
  monthlyLimitBytes: number;
  monthlyUsedBytes: number;
  monthlyRatePerDay: number;
  monthlyDaysRemaining: number | null;
  monthlyEstimatedHit: string | null;
}

export interface SpeedTestResult {
  downloadMbps: number;
  uploadMbps: number;
  latencyMs: number;
  server: string;
}

export interface SpeedTestProgress {
  phase: string;
  progress: number;
}

export interface PeakHourEntry {
  dayOfWeek: number;
  hour: number;
  totalBytes: number;
}

export type Page = 'dashboard' | 'daily' | 'monthly' | 'settings' | 'speedtest' | 'peakhours';

export interface AppSpeedEntry {
  appName: string;
  downloadSpeed: number;
  uploadSpeed: number;
  totalSpeed: number;
}

export interface SessionStats {
  uptimeSeconds: number;
  totalDownload: number;
  totalUpload: number;
  peakDownloadSpeed: number;
  peakUploadSpeed: number;
  avgDownloadSpeed: number;
  avgUploadSpeed: number;
}
