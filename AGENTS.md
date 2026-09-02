# Data Tracker - Application Memory (Modification Log)

This file is the modification memory for the Data Tracker application. Every change bumps a mod number and adds a new entry. Versioning starts at 1.0.0.

## Mod 1.2.4 - Live App Usage fallback fix (v1.2.4)

**Date:** 2026-09-02

### What was fixed
- **Live App Usage showed no data**: `live_app_speeds()` depended on EStats byte counters in the `pending` buffer. When EStats fails for all connections (common on many Windows configs), the buffer was always empty, so the component always showed "No active app traffic detected".
- **Fix**: added `live_speeds_with_fallback()` method that falls back to distributing adapter speed proportionally among processes weighted by TCP connection count (same proven logic the 60s save cycle uses). The monitoring loop now calls this method with the adapter's total speed, so per-app data always appears when there IS network traffic.

### Files / Components
- `src-tauri/src/monitor/app_usage.rs` — new `live_speeds_with_fallback()` method
- `src-tauri/src/monitor/mod.rs` — moved live speeds emission after adapter stats read, pass fallback total

### Verified
- `cargo check` clean (4 pre-existing dead-code warnings, no new ones).

## Mod 1.2.3 - Live Connection Count (v1.2.3)

**Date:** 2026-09-02

### What was added
- **Active Connections** stat in the Session Summary card: shows the current number of established TCP connections (IPv4 + IPv6), updating every 3 seconds.
- **Backend** (`src-tauri/src/monitor/app_usage.rs`): new `connection_count()` method on `AppUsageTracker` returns `all_conns.len()`.
- **`session-stats` event** now includes `activeConnections` field.

### Files / Components
- `src-tauri/src/monitor/app_usage.rs` — `connection_count()` method
- `src-tauri/src/monitor/mod.rs` — added `activeConnections` to `session-stats` event
- `src/types/index.ts` — added `activeConnections` to `SessionStats`
- `src/components/dashboard/SessionSummary.tsx` — added Connections stat with Wifi icon

### Verified
- `cargo check` clean (4 pre-existing dead-code warnings, no new ones).
- `tsc -b` clean (no errors).

## Mod 1.2.2 - Enforce minimizeToTray setting (v1.2.2)

**Date:** 2026-09-02

### What was fixed
- **Close button always hid to tray regardless of setting**: the `minimizeToTray` setting was saved to the database but never read. The close button always called `appWindow.hide()` with no way for users to actually close the app from the titlebar.
- **Fix**: `Titlebar.tsx` now reads `settings.minimizeToTray` from the Zustand store. When `false`, the close button calls `appWindow.close()` instead of `appWindow.hide()`. Defaults to tray behavior when setting is `null` (first launch).

### Files / Components
- `src/components/layout/Titlebar.tsx` — reads `settings.minimizeToTray` from store, conditionally closes or hides

### Verified
- `tsc -b` clean (no errors).

## Mod 1.2.1 - Current Session Summary Card (v1.2.1)

**Date:** 2026-09-02

### What was added
- **Session Summary card** on the Dashboard (`src/components/dashboard/SessionSummary.tsx`): Shows session uptime, total data transferred, peak download/upload speeds, and average download/upload speeds. Updates every 3 seconds.
- **Backend session tracking** (`src-tauri/src/monitor/mod.rs`): Cumulative session counters (never reset) for total bytes and peak speeds, plus uptime via `Instant::now()`. Emits `session-stats` Tauri event on every tick.

### Files / Components
- **New**: `src/components/dashboard/SessionSummary.tsx`
- **Modified**: `src-tauri/src/monitor/mod.rs` — session cumulative tracking + `session-stats` event
- **Modified**: `src/types/index.ts` — `SessionStats` interface
- **Modified**: `src/components/dashboard/Dashboard.tsx` — import and render `SessionSummary`

### Verified
- `cargo check` clean (4 pre-existing dead-code warnings, no new ones).
- `tsc -b` clean (no errors).

## Mod 1.2.0 - Per-App Real-Time View (v1.2.0)

**Date:** 2026-09-02

### What was added
- **Live App Usage card** on the Dashboard (`src/components/dashboard/LiveAppUsage.tsx`): Shows top 10 apps by real-time speed, with download and upload speeds per app. Updates every 3 seconds. Listens to the new `per-app-usage` Tauri event.
- **Backend `live_app_speeds()` method** (`src-tauri/src/monitor/app_usage.rs`): Calculates per-app speed deltas by comparing current pending data against previous tick. Returns `Vec<AppSpeedEntry>` without draining the pending buffer (unlike `flush()`).
- **`per-app-usage` event** (`src-tauri/src/monitor/mod.rs`): Emitted on every 3-second tick after `capture()`, carrying real-time per-app speed data to the frontend.

### Files / Components
- **New**: `src/components/dashboard/LiveAppUsage.tsx`
- **Modified**: `src-tauri/src/monitor/app_usage.rs` — `AppSpeedEntry` struct, `prev_pending` field, `live_app_speeds()` method
- **Modified**: `src-tauri/src/monitor/mod.rs` — emit `per-app-usage` event
- **Modified**: `src/types/index.ts` — `AppSpeedEntry` interface
- **Modified**: `src/components/dashboard/Dashboard.tsx` — import and render `LiveAppUsage`

### Verified
- `cargo check` clean (4 pre-existing dead-code warnings, no new ones).
- `tsc -b` clean (no errors).

## Mod 1.1.2 - Settings save confirmation (v1.1.2)

**Date:** 2026-09-02

### What was fixed
- **No feedback after "Save Settings"**: the save button showed "Saving..." while in progress, then reverted to "Save Settings" with no indication the save succeeded. Users had no way to know if their changes were applied.
- **Fix**: added a `saved` state that shows a green "Saved!" button with a checkmark icon for 2 seconds after a successful save, then automatically reverts to the default "Save Settings" button.

### Files / Components
- `src/components/settings/SettingsPage.tsx` — added `saved` state, `Check` icon import, conditional button styling/text

### Verified
- `tsc -b` clean (no errors).

## Mod 1.1.1 - Forecast progress bar color fix (v1.1.1)

**Date:** 2026-09-02

### What was fixed
- **Usage Forecast progress bars had no color**: the daily and monthly progress bars in `ForecastCard.tsx` used `hsl(var(--danger))` and `hsl(var(--primary))` as inline `backgroundColor`, but the CSS theme defines `--color-primary` and `--color-destructive` as hex values (not HSL components). The variable names and function wrapper were both wrong, so the bars rendered with no visible fill color.
- **Fix**: replaced `hsl(var(--danger))` → `var(--color-destructive)` and `hsl(var(--primary))` → `var(--color-primary)` in both daily and monthly progress bars.

### Files / Components
- `src/components/dashboard/ForecastCard.tsx` — progress bar inline style fix

### Verified
- `tsc -b` clean (no errors).

## Mod 1.1.0 - Usage Forecast + Speed Test + Peak Hours Heatmap (v1.1.0)

**Date:** 2026-08-26

### What was added

#### Feature 1 - Usage Forecast
- **Dashboard forecast card** (`src/components/dashboard/ForecastCard.tsx`): Shows daily and monthly usage progress bars with estimated time until limit is hit. Only visible when limits are configured.
- **Backend** (`src-tauri/src/commands/usage.rs`): New `get_usage_forecast` command calculates usage rates from `usage_snapshots` (hourly) and `daily_usage` (daily) tables, projects when limits will be reached based on current usage trajectory.
- **New queries** (`src-tauri/src/db/queries.rs`): `get_today_usage_rate` (bytes + hours active today) and `get_monthly_usage_rate` (bytes + days active this month).

#### Feature 2 - Speed Test
- **Speed Test page** (`src/components/speedtest/SpeedTestPage.tsx`): Full-page speed test with animated circular gauges for download, upload, and latency. Uses Cloudflare's speed test endpoints.
- **Speed Test gauge** (`src/components/speedtest/SpeedTestGauge.tsx`): SVG circular gauge with color-coded thresholds (green < 50 Mbps, yellow 50-200, red > 200).
- **Backend** (`src-tauri/src/commands/speedtest.rs`): Async `run_speed_test` command: downloads 25MB from Cloudflare, uploads 25MB, pings 5 times for median latency. Emits `speedtest-progress` events for real-time UI updates.
- **Dependencies**: Added `futures-util` for streaming download, enabled `stream` feature on `reqwest`.

#### Feature 3 - Peak Hours Heatmap
- **Heatmap component** (`src/components/history/PeakHoursHeatmap.tsx`): 7×24 grid (day-of-week × hour) with color intensity representing data usage. Hover tooltips show exact bytes.
- **Peak Hours page** (`src/components/history/PeakHoursPage.tsx`): Month/year selector, heatmap grid, peak hour and quietest hour summary cards.
- **Backend** (`src-tauri/src/commands/usage.rs`): New `get_peak_hours_heatmap` command queries `usage_snapshots` grouped by day-of-week and hour.
- **New query** (`src-tauri/src/db/queries.rs`): `get_peak_hours_data` returns `Vec<PeakHourCell>` for the 7×24 grid.

### Routing + Navigation
- New page types: `'speedtest'`, `'peakhours'` added to `Page` type in `types/index.ts`
- Sidebar updated with Peak Hours (Grid3x3 icon) and Speed Test (Gauge icon) nav items
- `AppLayout.tsx` updated with new page cases

### Files / Components
- **New**: `src-tauri/src/commands/speedtest.rs`, `src/components/speedtest/SpeedTestPage.tsx`, `src/components/speedtest/SpeedTestGauge.tsx`, `src/components/dashboard/ForecastCard.tsx`, `src/components/history/PeakHoursHeatmap.tsx`, `src/components/history/PeakHoursPage.tsx`
- **Modified**: `src-tauri/src/commands/mod.rs`, `src-tauri/src/commands/usage.rs`, `src-tauri/src/db/queries.rs`, `src-tauri/src/lib.rs`, `src-tauri/Cargo.toml`, `src/types/index.ts`, `src/components/layout/Sidebar.tsx`, `src/components/layout/AppLayout.tsx`, `src/components/dashboard/Dashboard.tsx`

### Verified
- `cargo check` clean (4 pre-existing dead-code warnings, no new ones).
- `tsc -b` clean (no errors).

## Mod 1.0.9 - Fallback fix: EStats-independent PID tracking + visible diagnostics (v1.0.9)

**Date:** 2026-08-26

### What was fixed
- **Per-app sections still empty despite v1.0.8**: The proportional fallback added in v1.0.8 was self-defeating. It read PID counts from `self.prev`, which was only populated when EStats succeeded. When EStats failed for all connections (the root cause of empty data), `self.prev` was empty, so the fallback had no PID data and never fired.
- **Diagnostic logging invisible in release builds**: All v1.0.8 diagnostic messages used `trace!`/`debug!` level, which `env_logger::init()` suppresses in release mode (defaults to `error!`-only). There was no way to diagnose EStats failures in production.

### Changes
- **`all_conns` field** (`src-tauri/src/monitor/app_usage.rs`): New `Mutex<HashMap<ConnKey, u32>>` on `AppUsageTracker` that tracks ALL established TCP connections (IPv4 + IPv6) regardless of EStats success. Populated from `GetExtendedTcpTable` rows with non-zero PIDs.
- **`TcpSnapshot` struct** (`src-tauri/src/monitor/app_usage.rs`): `tcp_snapshot()` and `tcp6_snapshot()` now return `TcpSnapshot { estats, all_keys }` — EStats data AND all connection keys in one struct. This avoids scanning the TCP table twice.
- **`active_pid_counts()` rewritten** (`src-tauri/src/monitor/app_usage.rs`): Now reads from `self.all_conns` (all established connections) instead of `self.prev` (EStats-successful connections only). The fallback in `mod.rs:91-112` now has PID data whenever there are active TCP connections, regardless of EStats status.
- **Log levels raised** (`src-tauri/src/monitor/app_usage.rs`): `trace!` → `warn!` for `SetPerTcpConnectionEStats`/`GetPerTcpConnectionEStats` failures. `debug!` → `info!` for snapshot summaries, flush results, and capture stats.
- **`env_logger` filter** (`src-tauri/src/lib.rs`): Changed from bare `env_logger::init()` to `Builder::from_env(Env::default().default_filter_or("info")).init()` so `info!`+ messages are visible in release builds.

### Files / Components
- `src-tauri/src/monitor/app_usage.rs` — TcpSnapshot struct, all_conns field, active_pid_counts rewrite, log level changes
- `src-tauri/src/lib.rs` — env_logger filter configuration

### Verified
- `cargo check` clean (4 pre-existing dead-code warnings, no new ones).

## Mod 1.0.8 - Per-app tracking fix: IPv6 support + proportional fallback + diagnostic logging (v1.0.8)

**Date:** 2026-08-25

### What was fixed
- **Per-app sections always empty** (Top 5 Apps Today, Usage by Application, Top 5 Apps This Month, Top Apps This Month): The per-app EStats-based tracking only monitored IPv4 TCP connections (`AF_INET`). Most modern Windows traffic uses IPv6 or QUIC/UDP, so the tracker collected zero data for the majority of connections. Additionally, `SetPerTcpConnectionEStats` return values were never checked — if EStats collection failed for a connection, it was silently skipped with no diagnostic output.

### Changes
- **IPv6 TCP support** (`src-tauri/src/monitor/app_usage.rs`): Added `tcp6_snapshot()` using `GetExtendedTcpTable(AF_INET6)` + `GetPerTcp6ConnectionEStats`/`SetPerTcp6ConnectionEStats`. Converts `MIB_TCP6ROW_OWNER_PID` to `MIB_TCP6ROW` via `mem::transmute` for the IPv6 EStats API. IPv6 connections are now tracked alongside IPv4.
- **ConnKey enum** (`src-tauri/src/monitor/app_usage.rs`): Replaced the `type ConnKey = (u32, u32, u32, u32, u32)` tuple with a `ConnKey` enum (`V4 { pid, la, lp, ra, rp }` / `V6 { pid, la: [u8;16], lp, ra: [u8;16], rp }`) to support both address families in the connection snapshot HashMap.
- **Diagnostic logging** (`src-tauri/src/monitor/app_usage.rs`): Added `log::debug!` for connection counts, EStats success/failure counts, pending PID counts, and flush sample totals. Added `log::trace!` for individual `SetPerTcpConnectionEStats` and `GetPerTcpConnectionEStats` failures. Added `log::warn!` for `GetExtendedTcpTable` failures.
- **`SetPerTcpConnectionEStats` error handling** (`src-tauri/src/monitor/app_usage.rs`): Return value is now checked and logged (previously silently discarded).
- **Proportional fallback** (`src-tauri/src/monitor/mod.rs:91-112`): If `flush()` returns zero samples but the adapter has traffic, the adapter total bytes are distributed proportionally among all processes with active TCP connections (weighted by connection count). This ensures the per-app sections are never completely empty when there IS network traffic, even if EStats fails for all connections.
- **`active_pid_counts()` method** (`src-tauri/src/monitor/app_usage.rs`): New public method on `AppUsageTracker` that returns `(pid, connection_count)` pairs from the last snapshot, used by the fallback logic.
- **`process_names()` made public** (`src-tauri/src/monitor/app_usage.rs`): Exported for the fallback in `monitor/mod.rs`.

### Files / Components
- `src-tauri/src/monitor/app_usage.rs` — rewrote with IPv6 support, ConnKey enum, logging, public APIs
- `src-tauri/src/monitor/mod.rs` — added proportional fallback + logging

### Verified
- `pnpm lint` + `tsc -b` pass (pre-existing warnings only).
- `cargo check` clean (4 pre-existing dead-code warnings, no new ones).

## Mod 1.0.7 - Chart data bugfixes: upload/download swap + TCP EStats struct fix + adapter cap (v1.0.7)

**Date:** 2026-08-18

### What was fixed
- **Upload/download labels swapped** (`src-tauri/src/monitor/app_usage.rs:93-96`): `capture()` accumulated `delta_in` (DataBytesIn = received/download) in `entry.0` and `delta_out` (DataBytesOut = sent/upload) in `entry.1`, but `flush()` mapped `entry.0` → `upload_bytes` and `entry.1` → `download_bytes`. Fixed by swapping the field assignments so DataBytesIn → download_bytes and DataBytesOut → upload_bytes.
- **Wrong TCP EStats struct size** (`src-tauri/src/monitor/app_usage.rs:22-38,181-182`): `windows-sys` 0.59 `TCP_ESTATS_DATA_ROD_v0` is 96 bytes (14 fields) but the actual Windows SDK `TCP_ESTATS_DATA_ROD_v0` is 56 bytes (4 u64 + 6 u32). Passing `rod_size=96` to `GetPerTcpConnectionEStats` may cause the OS to write data at incorrect offsets or behave unpredictably. Defined a local `#[repr(C)] TcpEstatsDataRod` struct matching the SDK's 56-byte layout and used its size for `rod_size`.
- **Per-app data exceeds adapter data** (`src-tauri/src/monitor/mod.rs:82-107`): Added a sanity check: if the sum of all per-app sample bytes in a save cycle exceeds the adapter-level session bytes, all app samples are proportionally scaled down to match the adapter total. Logs a warning when this happens.

### Verified
- `pnpm lint` + `tsc -b` pass (pre-existing warnings only).
- `cargo check` clean (4 pre-existing dead-code warnings, no new ones).
- `pnpm tauri:build` built `data-tracker.exe` (v1.0.7) + NSIS + MSI; `releases/` holds `data-tracker.exe`, `DataTracker_1.0.7_x64-setup.exe`, `DataTrackerSetup.exe`.

## Mod 1.0.6 - Per-app tracking + detail panel + usage notifications (v1.0.6)

**Date:** 2026-08-16

### Feature 1 - Network stats detail panel (per-app tracking)
- **Per-app data was never recorded before this mod**: `app_usage_records` / `daily_app_usage` / `monthly_app_usage` existed but were never written, so the app tables were always empty.
- **New per-app tracking engine** (`src-tauri/src/monitor/app_usage.rs`): IP Helper sampling. Every 3s tick it snapshots established IPv4 TCP connections (`GetExtendedTcpTable` owner-PID) and reads per-connection cumulative byte counters via `GetPerTcpConnectionEStats` (`TCP_ESTATS_DATA`, `SetPerTcpConnectionEStats` enables collection). Deltas per PID accumulate in memory; every 60s save tick they are flushed, resolved to process names via ToolHelp32 (`CreateToolhelp32Snapshot`/`Process32FirstW`), and written to `app_usage_records` + rolled up into `daily_app_usage` / `monthly_app_usage` (`upsert_app_daily_usage` / `upsert_app_monthly_usage`).
- **New queries** (`db/queries.rs`): `get_app_hourly_breakdown(app, date)` (per-hour from records) and `get_app_daily_breakdown_month(app, year, month)` (per-day from daily rollup). Commands `get_app_hourly_breakdown` / `get_app_daily_breakdown_month` registered in `lib.rs`.
- **UI**: clickable app rows on Daily/Monthly pages open an inline detail panel (`src/components/history/AppDetailPanel.tsx`) with a per-hour (day) or per-day (month) bar chart + download/upload/total chips. New "Top 5 Apps" horizontal-bar chart (`src/components/history/TopAppsChart.tsx`) on both history pages.
- **Known limitation**: TCP/IPv4 only (windows-sys 0.59 has no `GetPerUdpEndpointEStats`), sampling-based, so short-lived/UDP (QUIC) flows can be undercounted. Per-app history starts empty after this build.

### Feature 3 - Limit notifications + daily usage summary
- **Limits were never enforced before this mod**: the settings toggles existed but nothing checked usage or sent notifications.
- **New alerts loop** (`src-tauri/src/monitor/alerts.rs`), spawned from `start_monitoring` on a 60s interval: compares today's/month's usage vs `dailyLimitBytes`/`monthlyLimitBytes` at the warning %/danger %/100% levels. An in-memory `AlertState` sends each alert **once per period** (daily flags reset on new day, monthly on new month). Alerts are Windows toasts via the already-registered `tauri-plugin-notification` (`NotificationExt`); if `sound_alerts_enabled`, a system sound plays (`MessageBeep`, added `Win32_System_Diagnostics_Debug` + `Win32_UI_WindowsAndMessaging` features to `Cargo.toml`). Both limit alerts and summary are gated on `notifications_enabled`.
- **Daily usage summary**: new settings `daily_summary_enabled` (default off) + `daily_summary_time` (default `20:00`, HH:MM). When local time reaches the configured time and today's summary hasn't fired yet, a toast shows today's download/upload/total. Plumbed through `UserSettings` (db struct + schema + migration `ALTER TABLE ADD COLUMN` ignored if exists), `get_settings`/`update_settings`, `SettingsResponse`, TS types, and a toggle + `type="time"` input in the Settings > Notifications section.
- **`scripts/copy-releases.cjs` now prunes old versioned installers** and always refreshes `DataTrackerSetup.exe` (site asset) so `releases/` keeps only the current version.

### Verified
- `pnpm lint` + `tsc -b` pass (pre-existing warnings only: exhaustive-deps, unused SettingsPage imports).
- `cargo check` clean (4 pre-existing dead-code warnings, no new ones).
- `pnpm tauri:build` built `data-tracker.exe` + NSIS + MSI; `releases/` now holds only `data-tracker.exe`, `DataTracker_1.0.6_x64-setup.exe`, `DataTrackerSetup.exe`.
- Not yet committed or released (awaiting user approval).

## Mod 1.0.5 - UI fixes + update auto-restart (v1.0.5)

**Date:** 2026-08-16

### What was fixed
- **Removed hardcoded version in sidebar bottom** (`src/components/layout/Sidebar.tsx`): the left-bottom footer showing `v1.0.0` (which was also wrong) is gone. The version pill lives only on the Live Dashboard header (`Dashboard.tsx`).
- **Speed chart now live while the window is open, not only when focused** (`src/components/layout/AppLayout.tsx`): the `tauri://blur` listener set `isWindowVisible = false` whenever the window lost focus, so the `network-speed` listener skipped updates and the Speed History chart froze. Blur listener removed; `isWindowVisible` now only goes `false` when the window is actually hidden to tray (Titlebar close). `tauri://focus` (restore from tray / single-instance) and the `onResized` -> `isVisible()` safety check remain.
- **Update now auto-restarts without waiting for the user to close the app** (`src-tauri/src/commands/update.rs`): the `apply_update.cmd` script loops `del` until the running exe unlocks and then swaps + relaunches, but the app never exited, so it spun forever until the user manually closed it. `apply_update` now takes `app: tauri::AppHandle` and, after the download + script launch succeed, spawns an async task that sleeps ~800ms (so the UI can show "Restarting...") then calls `app.exit(0)`. The script's own wait loop then swaps `data-tracker.exe` and relaunches automatically. Inner `Result` now unwrapped with `??` (no unused-must-use warning).
- **Repo cleanup**: removed unused tracked files - `src/assets/react.svg`, `src/assets/vite.svg`, `src/assets/hero.png`, `src/App.css` (never imported), `public/icons.svg` (never referenced); deleted the now-empty `src/assets/` folder and the `datatracker Sell/` folder (already gitignored). `releases/` now keeps only `data-tracker.exe`, `DataTracker_1.0.5_x64-setup.exe`, `DataTrackerSetup.exe` (old 1.0.0-1.0.4 installers removed, per "keep only new version" rule). Root `cmd.exe` kept by user request.

### Verified
- `pnpm lint` and `tsc -b` pass (only pre-existing warnings). Rust compiles clean via `pnpm tauri:build` (6 pre-existing dead-code warnings, no new ones).
- Built `data-tracker.exe` (v1.0.5) + `DataTracker_1.0.5_x64-setup.exe` copied to `releases/`, plus `DataTrackerSetup.exe` (renamed installer, site asset).
- Not yet committed or released (awaiting user approval).

## Mod 1.0.4 - Rate-limit-free update check + restored site download asset (v1.0.4)

**Date:** 2026-08-12

### What was fixed
- **"Check for updates" failing with HTTP 403**: v1.0.3 switched the check to the anonymous GitHub REST API (`api.github.com/.../releases/latest`), which is rate-limited to 60 requests/hour per IP. When the quota is exhausted the API returns 403, so the app showed "Check failed". 
- **Site download link 404**: the site's link `.../releases/download/v1.0.3/DataTrackerSetup.exe` broke because `DataTrackerSetup.exe` was dropped from the v1.0.3 release (it existed in v1.0.2). 
- **Fix 1 - no API rate limit** (`src-tauri/src/commands/update.rs`): `run_check_for_updates` now GETs `https://github.com/{repo}/releases/latest`, which 302-redirects to `/releases/tag/vX.Y.Z`; the tag is parsed from the final URL (reqwest follows the redirect, 15s timeout, `DataTracker-update-check` user-agent). Same `UpdateInfo { current, latest }` contract and same error surfacing in `Dashboard.tsx`.
- **Fix 2 - restore site asset**: `DataTrackerSetup.exe` (= the NSIS installer, renamed) uploaded to the v1.0.3 release (HTTP 200 verified) and carried on every release going forward (v1.0.4 included).

### Verified
- `pnpm lint` and `tsc -b` pass (only pre-existing warnings). Rust compiles clean via `pnpm tauri:build`.
- Built `data-tracker.exe` (v1.0.4) + `DataTracker_1.0.4_x64-setup.exe` copied to `releases/`.
- Released via git: tag `v1.0.4` pushed; GitHub Release `v1.0.4` created with three assets: `data-tracker.exe` (in-app self-update), `DataTrackerSetup.exe` (site download), `DataTracker_1.0.4_x64-setup.exe`. All anonymous downloads verified HTTP 200. `releases/latest` verified to redirect to `/releases/tag/v1.0.4`. `medial_support.txt` regenerated.

## Mod 1.0.3 - Update check via GitHub API, no git required (v1.0.3)

**Date:** 2026-08-12

### What was fixed
- **"Check for updates" failing with "Check failed"**: the update check spawned `git ls-remote` against the repo, which failed fast in the app's runtime environment (git installed per-user at `%LOCALAPPDATA%\Programs\git` may be missing from the PATH inherited by a GUI-launched process), or git exited non-zero. The UI swallowed the real error message so only a generic "Check failed" was shown.
- **Fix 1 - no more git dependency** (`src-tauri/src/commands/update.rs`): `run_check_for_updates` now calls the GitHub REST API `GET https://api.github.com/repos/{repo}/releases/latest` via the existing `reqwest` (blocking, rustls + `json` feature added to `Cargo.toml`) with a 15s timeout and `DataTracker-update-check` user-agent. It parses `tag_name` and compares against `CARGO_PKG_VERSION` with the existing semver logic, keeping the same `UpdateInfo { current, latest }` contract. Removes the documented "git required" limitation.
- **Fix 2 - apply download timeout**: the release download in `run_apply_update` now uses a 60s-timeout client instead of bare `reqwest::blocking::get`.
- **Fix 3 - surface real errors** (`src/components/dashboard/Dashboard.tsx`): the failed state now captures and carries the actual error message; the button shows it as a tooltip (`title`) and the state resets after 5s.

### Verified
- `pnpm lint` and `tsc -b` pass (only pre-existing warnings). Rust compiles clean via `pnpm tauri:build`.
- Built `data-tracker.exe` (v1.0.3) + `DataTracker_1.0.3_x64-setup.exe` copied to `releases/`. Note: copy-releases needs the app closed (destination exe locked while running).
- Released via git: tag `v1.0.3` pushed; GitHub Release `v1.0.3` created with `data-tracker.exe` + `DataTracker_1.0.3_x64-setup.exe`. Anonymous exe download verified HTTP 200. GitHub API `latest` endpoint returns `tag_name: v1.0.3` (anonymous rate limit 60/hr per IP; the app reports HTTP 403 clearly if it ever hits it). `medial_support.txt` regenerated.

## Mod 1.0.2 - Single-instance enforcement (v1.0.2)

**Date:** 2026-08-09

### What was fixed
- **Second launch spawned a new instance**: launching `data-tracker.exe` while the app was already running (e.g. tray-hidden) started a duplicate process and the tray icon/DB could conflict.
- **Fix**: added `tauri-plugin-single-instance` (`src-tauri/Cargo.toml`), registered first in `src-tauri/src/lib.rs`. A second launch now exits immediately and the existing "main" window is shown, unminimized and focused.

### Verified
- `pnpm lint` and `tsc -b` pass. Rust compiles clean via `pnpm tauri:build`.
- Built `data-tracker.exe` + `DataTracker_1.0.2_x64-setup.exe` copied to `releases/`.
- Released via git: annotated tag `v1.0.2` pushed; GitHub Release `v1.0.2` carries **two assets** - `DataTracker_1.0.2_x64-setup.exe` (installer = main release download) and `data-tracker.exe` (kept so the in-app "Check for updates" can swap the exe in place). Anonymous downloads verified HTTP 200. `medial_support.txt` regenerated.

## Mod 1.0.1 - Git-based update feature + version in header (v1.0.1)

**Date:** 2026-08-09

### What was added
- **Git-based self-update** (`src-tauri/src/commands/update.rs`): matches the SmartCopy/Copy-tracker pattern.
  - `check_for_updates` runs `git ls-remote --tags https://github.com/chamarawickramarathne-spec/Data-tracker.git`, parses `vX.Y.Z` tags (lightweight and annotated, prerelease ignored), and returns `{ current, latest }` where `latest` is the newest tag newer than `CARGO_PKG_VERSION` (else `null`).
  - `apply_update` downloads `https://github.com/{repo}/releases/download/v{version}/data-tracker.exe` to `{exe}.new` via `reqwest` (blocking, rustls), writes an `apply_update.cmd` next to the running exe that waits until the exe can be deleted, swaps `{exe}.new` in, relaunches the app, and cleans up. The cmd runs with `CREATE_NO_WINDOW`.
  - Commands registered in `lib.rs` invoke handler; module added to `commands/mod.rs`.
- **Version + update button on Live Dashboard** (`src/components/dashboard/Dashboard.tsx`): the header now shows a `vX.Y.Z` pill (from `@tauri-apps/api/app` `getVersion`) next to the "Live Dashboard" title, plus a "Check for updates" button. SmartCopy-style flow: `Check for updates -> Checking... -> Up to date | Downloading vX.Y.Z... -> Restarting... | Check failed` (resets after 3s). On a newer version the app auto-downloads and restarts to apply.
- **Dependency**: `reqwest` added to `src-tauri/Cargo.toml`.

### Verified
- `pnpm lint` and `tsc -b` pass. Rust compiles clean via `pnpm tauri:build`.
- Built `data-tracker.exe` + `DataTracker_1.0.1_x64-setup.exe` copied to `releases/`.
- Released via git: repo `chamarawickramarathne-spec/Data-tracker` (public); annotated tags `v1.0.0` (baseline) and `v1.0.1` pushed; GitHub Release `v1.0.1` created with `releases/data-tracker.exe` asset (anonymous download verified HTTP 200, ~17.5 MB). `medial_support.txt` regenerated.

## Mod 1.0.0 - Initial Release (v1.0.0)

**Date:** 2026-08-09

**Stack:** Tauri 2 (Rust) + React 19 + Vite 8 + TypeScript, Tailwind CSS, Zustand, Recharts, rusqlite (`datatracker.db`), NSIS installer.

### What was built
- **Live Dashboard**: real-time download/upload/total speed cards + 5-minute speed history chart.
- **History**: daily and monthly usage pages with app breakdown and usage history.
- **Network monitoring** (Rust): per-adapter sampling via `windows-sys` IP Helper, per-app usage via ETW, persisted to SQLite.
- **Settings**: data limits (daily/monthly), warning/danger thresholds, notifications, sound alerts, auto-start, minimize-to-tray, data retention.
- **System tray**: show window / quit menu, left-click restore.
- **Packaging**: `pnpm tauri:build` -> `src-tauri/target/release/bundle/nsis/DataTracker_1.0.0_x64-setup.exe`; `scripts/copy-releases.cjs` copies exe + installer to `releases/`.

### Known limitations (v1.0.0)
- No git-based updates yet (added in 1.0.1).
- Git self-update requires `git` on the user's machine.

### App rules followed
- Clean code, every file under 300 lines.
- Version 1.0.0 -> 1.0.1.
- Windows app -> exe + installer produced (releases/).
- `medial_support.txt` maintained in root.
