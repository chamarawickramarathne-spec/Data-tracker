# Data Tracker - Development Plan

## Current Version: 1.2.8

---

## Completed

### Mod 1.0.0 - Initial Release
- Live Dashboard (real-time speed cards + 5-min chart)
- History (daily/monthly usage with app breakdown)
- Network monitoring (Rust: per-adapter + per-app, SQLite)
- Settings (limits, thresholds, notifications, auto-start, tray, retention)
- System tray, NSIS installer
- Stack: Tauri 2 + React 19 + Vite 8 + TypeScript + Tailwind + Zustand + Recharts + rusqlite

### Mod 1.0.1 - Git-based self-update
- `check_for_updates` / `apply_update` via git tags + reqwest
- Version pill + update button on Live Dashboard

### Mod 1.0.2 - Single-instance enforcement
- `tauri-plugin-single-instance` prevents duplicate launches

### Mod 1.0.3 - Update check via GitHub API (no git required)
- Switched from `git ls-remote` to GitHub REST API `releases/latest`
- Download timeout + real error surfacing in UI

### Mod 1.0.4 - Rate-limit-free update check
- Switched to `GET /releases/latest` redirect (no API rate limit)
- Restored `DataTrackerSetup.exe` site download asset

### Mod 1.0.5 - UI fixes + update auto-restart (no separate tag)
- Removed hardcoded version in sidebar
- Speed chart live while window is open (not only focused)
- Update auto-restarts without user closing app
- Repo cleanup (unused files removed)
- **Note**: v1.0.5 tag was never created; changes folded into v1.0.6 commit (`ff14df4`)

### Mod 1.0.6 - Per-app tracking + detail panel + usage notifications
- New per-app tracking engine (`app_usage.rs`): IP Helper sampling, TCP/IPv4, EStats
- Per-hour/per-day breakdowns + Top 5 Apps charts
- Clickable app rows → inline detail panel
- Alerts loop: limit warnings/danger + daily summary toast
- Settings: daily_summary_enabled + daily_summary_time

### Mod 1.0.7 - Chart data bugfixes
- Fixed upload/download label swap
- Fixed TCP EStats struct size (56 bytes, not 96)
- Added adapter cap: per-app data can't exceed adapter total

### Mod 1.0.8 - Per-app tracking fix: IPv6 + fallback + logging
- IPv6 TCP support via `GetExtendedTcpTable(AF_INET6)` + `GetPerTcp6ConnectionEStats`
- ConnKey enum to support both address families
- Proportional fallback when EStats returns zero samples
- Diagnostic logging (debug/trace/warn) for EStats operations
- `SetPerTcpConnectionEStats` return values now checked

### Mod 1.0.9 - Fallback fix: EStats-independent PID tracking + visible diagnostics
- `all_conns` field tracks ALL established TCP connections regardless of EStats success
- `active_pid_counts()` reads from `all_conns` instead of `self.prev`
- Proportional fallback now works when EStats fails for all connections
- Log levels raised: `trace!`→`warn!` for EStats failures, `debug!`→`info!` for summaries
- `env_logger` configured with `info` filter for visible release diagnostics

### Mod 1.1.0 - Usage Forecast + Speed Test + Peak Hours Heatmap
- **Usage Forecast**: Dashboard card showing estimated time until daily/monthly limits are hit based on current usage rate
- **Speed Test**: Full page with animated gauges, downloads/uploads 25MB from Cloudflare, measures latency
- **Peak Hours Heatmap**: 7x24 grid showing data usage by hour and day of week, with peak/quietest hour summaries
- New pages: Speed Test, Peak Hours (added to sidebar navigation)

### Mod 1.1.1 - Forecast progress bar color fix
- Fixed ForecastCard progress bars using wrong CSS variable names (`hsl(var(--danger))` / `hsl(var(--primary))`) → `var(--color-destructive)` / `var(--color-primary)`

### Mod 1.2.8 - Dashboard 60/40 split
- Speed History and Live App Usage use 60/40 split (`grid-cols-[3fr_2fr]`)

### Mod 1.2.7 - Live App Usage top 8 + 75/25 dashboard split
- Top 8 apps (was 10)
- 75/25 split: Speed History gets more space, Live App Usage narrower

### Mod 1.2.6 - Dashboard layout: Live App Usage beside Speed History
- Live App Usage moved beside Speed History in 2-column grid
- Both cards match height, no scrollbars

### Mod 1.2.5 - Live App Usage names fix
- Fixed all apps showing as "Unknown" in Live App Usage
- Both `live_app_speeds()` and `live_speeds_with_fallback()` now call `process_names()` to refresh PID→name cache before lookup
- Names are immediately available on every 3s tick

### Mod 1.2.4 - Live App Usage fallback fix
- Fixed Live App Usage showing no data when EStats fails
- Added `live_speeds_with_fallback()` method using adapter speed proportional distribution
- Monitoring loop now uses fallback with adapter total speed on every 3s tick

### Mod 1.2.3 - Live Connection Count
- Active Connections stat in Session Summary card showing current TCP connection count
- Backend `connection_count()` method on `AppUsageTracker` returns `all_conns.len()`

### Mod 1.2.2 - Enforce minimizeToTray setting
- Close button now reads `settings.minimizeToTray` from Zustand store
- When false, clicking X actually closes the app instead of hiding to tray

### Mod 1.2.1 - Current Session Summary Card
- Session Summary card on Dashboard showing uptime, total data, peak speeds, avg speeds
- Backend emits `session-stats` event every 3 seconds with cumulative session data

### Mod 1.2.0 - Per-App Real-Time View
- Live App Usage card on Dashboard showing top 10 apps by real-time speed
- Backend emits `per-app-usage` event every 3 seconds with speed deltas
- `live_app_speeds()` method compares current vs previous tick without draining pending buffer

### Mod 1.1.2 - Settings save confirmation
- Save button now shows green "Saved" with checkmark for 2 seconds after successful save

---

## Pending

None. All planned features and fixes are complete.

---

## Known Gaps

- **v1.0.5 tag missing**: Changes were committed as part of v1.0.6. No separate tag exists.
- **Per-app tracking limitations**: TCP only (no UDP/QUIC), sampling-based, short-lived flows undercounted.
