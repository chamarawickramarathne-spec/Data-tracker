# Data Tracker - v1.0.6 Plan

## Feature 1 - Network stats detail panel (per-app tracking + detail view)

### 1a. Per-app data collection - NEW `src-tauri/src/monitor/app_usage.rs`
- IP Helper sampling engine: `GetExtendedTcpTable` (owner-PID) + `GetPerTcpConnectionEStats` (TCP_ESTATS_DATA DataBytesIn/Out), aggregated per PID. TCP/IPv4 only (no UDP stats in windows-sys 0.59).
- PID -> process name via ToolHelp32 snapshot (`CreateToolhelp32Snapshot`, `Process32FirstW`).
- `AppUsageTracker`: capture() every tick (deltas per connection), flush() every 60s save tick -> writes `app_usage_records` + rolls up `daily_app_usage` / `monthly_app_usage`.
- Status: DONE

### 1b. Queries + commands
- `db/queries.rs`: `get_app_hourly_breakdown(app_name, date)`, `get_app_daily_breakdown_month(app_name, year, month)`, `upsert_app_daily_usage`, `upsert_app_monthly_usage`.
- `commands/usage.rs`: `get_app_hourly_breakdown`, `get_app_daily_breakdown_month`; registered in `lib.rs`.
- Status: DONE

### 1c. UI
- NEW `src/components/history/AppDetailPanel.tsx` (inline per-app chart + summary chips).
- `DailyPage.tsx`: clickable app rows -> detail panel; "Top 5 Apps Today" chart.
- `MonthlyPage.tsx`: clickable app rows -> detail panel; "Top 5 Apps This Month" chart.
- Status: DONE

## Feature 3 - Limit notifications + daily usage summary

### 3a. Alerts loop - NEW `src-tauri/src/monitor/alerts.rs`
- 60s task with AppHandle: check daily/monthly usage vs limits (warning/danger/100%), one alert per period via in-memory `AlertState`.
- Toast via `tauri_plugin_notification::NotificationExt`; sound via `MessageBeep` when `sound_alerts_enabled` (added `Win32_System_Diagnostics_Debug` + `Win32_UI_WindowsAndMessaging`).
- Daily summary toast when local time >= `daily_summary_time` and today not yet sent.
- Wired into `start_monitoring`.
- Status: DONE

### 3b. Settings plumbing
- Added `daily_summary_enabled` (default 0), `daily_summary_time` (default "20:00") to db struct/schema/queries, SettingsResponse/update_settings, types/index.ts.
- Migration: `ALTER TABLE ADD COLUMN`, error ignored if exists.
- Status: DONE

### 3c. Settings UI - `SettingsPage.tsx`
- "Daily usage summary" toggle + time input in Notifications section.
- Status: DONE

## Versioning
- Bump 1.0.5 -> 1.0.6 in `Cargo.toml`, `package.json`, `tauri.conf.json`.
- Status: DONE

## Verification
- `pnpm lint`, `tsc -b`, `pnpm tauri:build` pass (pre-existing warnings only).
- `releases/` cleaned: only `data-tracker.exe`, `DataTracker_1.0.6_x64-setup.exe`, `DataTrackerSetup.exe`.
- `scripts/copy-releases.cjs` now prunes old installers + refreshes site asset automatically.
- Status: DONE

## Docs
- Update `AGENTS.md` (Mod 1.0.6), regenerate `medial_support.txt`.
- Status: DONE

## Release (on user approval - NOT DONE, blocked on user)
- Commit, tag `v1.0.6`, push, GitHub Release with `data-tracker.exe`, `DataTrackerSetup.exe`, `DataTracker_1.0.6_x64-setup.exe`.
