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
}

export interface ThemeInfo {
  theme: string;
  isDark: boolean;
}

export type Page = 'dashboard' | 'daily' | 'monthly' | 'settings';
