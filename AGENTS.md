# Data Tracker - Application Memory (Modification Log)

This file is the modification memory for the Data Tracker application. Every change bumps a mod number and adds a new entry. Versioning starts at 1.0.0.

## Mod 1.3.4 - Dummy release to validate in-app update (v1.3.4)

**Date:** 2026-10-12

### What was changed
- **No functional change.** Version-only bump 1.3.3 → 1.3.4 to produce a newer release for testing the in-app "Check for updates" flow end-to-end.
- Confirms the v1.3.3 updater fix: install runs 1.3.3 → checks → finds 1.3.4 → downloads the arch payload → verifies via `SHA256SUMS.txt` → auto-restarts.
- Version bumped in `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, `src-tauri/src/commands/speedtest.rs`.

### Files / Components
- `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, `src-tauri/src/commands/speedtest.rs` — version 1.3.4
- `AGENTS_PLAN.md` — updated

### Verified
- `pnpm tauri:build` (x64) + `pnpm tauri build --target i686-pc-windows-msvc` (x86) produced exe + NSIS + MSI bundles.
- `releases/SHA256SUMS.txt` matches the actual payloads; both payloads report version 1.3.4.

### Important notes
- Test requires an installed **1.3.3** (clients ≤1.3.2 cannot self-update unsigned builds). From 1.3.3 onward the in-app update pulls 1.3.4 automatically.

## Mod 1.3.3 - Reliable release-based self-update (v1.3.3)

**Date:** 2026-10-12

### What was fixed
- **In-app self-update failed to verify the download**, so it never applied. `fetch_release_checksum` (`src-tauri/src/commands/update.rs`) fetched the GitHub release page and did a strict **line-prefix** match: a line had to *start* with `SHA256-<arch>:` (or `SHA256:`) and the rest had to be *exactly* 64 hex chars. GitHub renders the body as `<p>SHA256-x64: …<br> SHA256-x86: …<br> SHA256: …</p>`, so:
  - **x86 → always failed** (arch line ends in `<br>`, 68 chars).
  - **plain `SHA256:` → always failed** (ends in `</p>`, 68 chars) — this is the label used by clients ≤1.3.0.
  - **x64 → passed only by luck**, because the `og:description` meta tag happened to contain one clean `SHA256-x64:` line.
- **Fix — publish a machine-readable checksum asset**: `scripts/copy-releases.cjs` now generates `releases/SHA256SUMS.txt` (sha256sum format) for `data-tracker-x64.exe`, `data-tracker-x86.exe`, and the legacy `data-tracker.exe` (x64).
- **Fix — robust lookup** (`update.rs`): `fetch_release_checksum` now
  1. downloads `…/releases/download/v<version>/SHA256SUMS.txt` and matches the current arch's asset (`parse_checksum_file`; legacy `data-tracker.exe` accepted as x64), then
  2. falls back to a whole-body scan of the release page that finds the first complete 64-hex digest after `SHA256-<arch>:` then `SHA256:` (`extract_hex_after`), tolerating surrounding HTML.
  - `normalize_hex` + `verify_sha256` unchanged. Authenticode remains optional (unsigned tolerated, invalid/untrusted rejected).
- Version bumped 1.3.2 → 1.3.3 (`package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, speedtest user-agent).

### Files / Components
- `src-tauri/src/commands/update.rs` — `fetch_release_checksum` rewritten; new `parse_checksum_file` + `extract_hex_after`
- `scripts/copy-releases.cjs` — writes `SHA256SUMS.txt` for the self-update payloads
- `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, `src-tauri/src/commands/speedtest.rs` — version 1.3.3
- `AGENTS_PLAN.md`, `medial_support.txt` — updated

### Verified
- `cargo check` clean (4 pre-existing dead-code warnings, no new); `tsc -b` clean; `pnpm lint` clean (pre-existing warnings only).
- `pnpm tauri:build` (x64) + `pnpm tauri build --target i686-pc-windows-msvc` (x86) produced exe + NSIS + MSI bundles.
- `releases/SHA256SUMS.txt` verified against the actual payload hashes; both payloads report version 1.3.3.

### Security note
- Update integrity is still enforced via SHA-256; the expected digest now comes from a dedicated release asset (with an HTML-scan fallback). Executables remain unsigned (no code-signing certificate), so Authenticode stays optional exactly as in 1.3.2.

### Important notes
- **Clients ≤1.3.2 cannot self-update** an unsigned build (1.3.0/1.3.1 require a valid Authenticode signature; 1.3.2's x86 checksum parsing fails). **v1.3.3 requires a one-time manual install**; from 1.3.3 onward, "Check for updates" downloads → SHA-256 verifies → auto-restarts for both x86 and x64.

## Mod 1.3.2 - Fix update check for unsigned builds (v1.3.2)

**Date:** 2026-10-12

### What was fixed
- **"Check for updates" failed**: v1.3.1 renamed the self-update payload to arch-specific assets (`data-tracker-x64.exe` / `data-tracker-x86.exe`) and dropped the legacy `data-tracker.exe` asset and the plain `SHA256:` line. Older clients (≤1.3.0) still download `data-tracker.exe` and parse a plain `SHA256:` line, so they hit **HTTP 404** and showed "Check failed".
  - Restored a legacy `data-tracker.exe` (x64) alias, now emitted by `scripts/copy-releases.cjs`.
  - The v1.3.1 release was also back-patched server-side with `data-tracker.exe` + a plain `SHA256:` line for old clients.
- **In-app self-update was impossible for unsigned builds**: `verify_authenticode` required a valid Authenticode signature, but the executables are not signed, so every update was refused. It now **tolerates a missing signature** (falls back to SHA-256 integrity) while still **rejecting a signature that is present but invalid or untrusted** (`is_unsigned()` checks `TRUST_E_NOSIGNATURE` / `TRUST_E_SUBJECT_FORM_UNKNOWN` / `TRUST_E_PROVIDER_UNKNOWN`).
- Version bumped 1.3.1 → 1.3.2 (`package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, speedtest user-agent).

### Files / Components
- `src-tauri/src/commands/update.rs` — optional Authenticode (`is_unsigned`), warn + proceed for unsigned; arch-aware download retained
- `scripts/copy-releases.cjs` — emits legacy `data-tracker.exe` (x64 alias) alongside arch payloads
- `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, `src-tauri/src/commands/speedtest.rs` — version 1.3.2
- `AGENTS_PLAN.md` — updated

### Security note
- Signature verification is now enforced only when a signature exists; integrity is still enforced via the SHA-256 checksum published in the release body. This relaxation is required because the project has no code-signing certificate (user-approved decision).

### Important notes
- Clients on ≤1.3.0 still cannot self-update (their built-in check is strict) and need a one-time manual install of 1.3.2.
- From 1.3.2 onward, in-app self-update works for unsigned releases (SHA-256 verified).
- Dual-arch build retained: `data-tracker-x64.exe`, `data-tracker-x86.exe`, `DataTracker_1.3.2_x64-setup.exe`, `DataTracker_1.3.2_x86-setup.exe`, `DataTrackerSetup.exe`.

## Mod 1.3.1 - Remove Usage Forecast feature (v1.3.1)

**Date:** 2026-10-12

### What was changed
- **Removed the "Usage Forecast" section** from the Live Dashboard. It projected when daily/monthly data limits would be hit based on current usage rate but was judged not useful versus the existing limit alerts and Session Summary.
- Frontend: deleted the dedicated `ForecastCard.tsx`; removed its import and render from `Dashboard.tsx`; removed the `UsageForecast` TS interface.
- Backend: removed the `get_usage_forecast` command + `UsageForecast` struct from `commands/usage.rs`, its registration in `lib.rs`, and the now-unused `UsageForecastRow` struct + `get_today_usage_rate` / `get_monthly_usage_rate` helpers in `db/queries.rs`.
- Version bumped 1.3.0 → 1.3.1 (`package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, speedtest user-agent).
- **Dual-architecture release build**: added an x86 (`i686-pc-windows-msvc`) build alongside x64 to satisfy AGENTS.md §5.
  - `scripts/copy-releases.cjs` now handles both target dirs (`target/release` = x64, `target/i686-pc-windows-msvc/release` = x86), emits per-arch self-update payloads `data-tracker-x64.exe` / `data-tracker-x86.exe`, copies both versioned installers, prunes old per-arch installers, and keeps `DataTrackerSetup.exe` mirroring the newest x64 installer.
  - `src-tauri/src/commands/update.rs` is now architecture-aware (AGENTS.md §7.1): `arch_suffix()` maps `std::env::consts::ARCH` to `x64`/`x86`, downloads `data-tracker-<arch>.exe`, and verifies against the matching `SHA256-x64:` / `SHA256-x86:` release-body line (plain `SHA256:` still accepted as a fallback for older single-arch releases).

### Files / Components
- `src/components/dashboard/ForecastCard.tsx` — deleted
- `src/components/dashboard/Dashboard.tsx` — removed import + render
- `src/types/index.ts` — removed `UsageForecast`
- `src-tauri/src/commands/usage.rs` — removed `UsageForecast` struct + `get_usage_forecast`
- `src-tauri/src/db/queries.rs` — removed `UsageForecastRow` + `get_today_usage_rate` + `get_monthly_usage_rate`
- `src-tauri/src/lib.rs` — unregistered `get_usage_forecast`
- `src-tauri/src/commands/update.rs` — arch-aware download + checksum verification
- `scripts/copy-releases.cjs` — dual-arch artifact collection
- `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, `src-tauri/src/commands/speedtest.rs` — version 1.3.1
- `AGENTS_PLAN.md`, `medial_support.txt` — updated

### Verified
- `pnpm lint` clean (pre-existing warnings only); `tsc -b` clean; `cargo check` clean (4 pre-existing dead-code warnings, no new).
- `pnpm tauri:build` (x64) and `pnpm tauri build --target i686-pc-windows-msvc` (x86) both produced exe + NSIS + MSI bundles.

### Release (v1.3.1)
- Tag `v1.3.1`; GitHub release `Data Tracker 1.3.1` with assets:
  - `data-tracker-x64.exe` — SHA256 `78749AB93BDBD03CEF76F9D492DFF02057A9CB3EA997998AA9C5D7E13D234D54`
  - `data-tracker-x86.exe` — SHA256 `D04F3FADF343ED2307A043C401EB446483B2DE2AF11EC62A30661E7FE1D88F1E`
  - `DataTracker_1.3.1_x64-setup.exe`, `DataTracker_1.3.1_x86-setup.exe`, `DataTrackerSetup.exe`

### Important notes
- No dependency, capability, or database-schema changes were required. The `usage_snapshots` and `daily_usage` tables remain and are still used by history/peak-hours features.
- The shipped executables are **not Authenticode-signed** (no code-signing certificate available), so the updater's `WinVerifyTrust` check will still refuse in-app self-updates; users update via the installer. SHA-256 integrity is still enforced.

## Mod 1.3.0 - Security audit remediation (v1.3.0)

**Date:** 2026-09-08

### What was fixed

#### CRITICAL - Signed/checksum-verified auto-update (C1)
- **Unsigned, unverified update download**: `apply_update` previously downloaded `data-tracker.exe` from GitHub over HTTPS with NO signature or checksum verification, then executed a `cmd` script that swapped the running exe. Any compromise of the repo/TLS could ship arbitrary code.
- **Fix** (`src-tauri/src/commands/update.rs`):
  - Downloaded bytes now verified against a **SHA-256 checksum** published in the GitHub release body (line `SHA256: <hex>`).
  - Staged exe must pass **Windows Authenticode signature verification** via `WinVerifyTrust` (`WINTRUST_ACTION_GENERIC_VERIFY_V2`, `WTD_UI_NONE`, `WTD_REVOCATION_CHECK_NONE`, `WTD_SAFER_FLAG`) before the swap script is written.
  - `repo` is now **pinned server-side** to `UPDATE_REPO` (the `apply_update` command no longer accepts a frontend-supplied `repo`; only `version`), and `version` is validated (`verify_version_format`) to reject path traversal (`/`, `\`, `..`) and malformed semver.
- New deps: `sha2` (direct), `windows-sys` features `Win32_Security_WinTrust` + `Win32_Security_Cryptography`, `chrono::Datelike` import in `queries.rs`.
- Frontend `Dashboard.tsx` `invoke('apply_update', ...)` updated to pass only `version`.

#### CRITICAL - CSP disabled (C2)
- **`"csp": null`** removed; replaced with a strict CSP in `src-tauri/tauri.conf.json`: `default-src 'self'; connect-src 'self' https://github.com https://speed.cloudflare.com; img-src 'self' data:; style-src 'self' 'unsafe-inline'; script-src 'self'; font-src 'self' data:`.

#### HIGH - Unbounded DB growth / inert retention (H1)
- `data_retention_days` was saved but never enforced; `usage_snapshots` (every 60s) + `app_usage_records` (per-app every 60s) grew without bound.
- **Fix**:
  - New `purge_old_usage(conn, retention_days)` in `db/queries.rs` deletes snapped/app records older than the window and rolled-up daily/monthly rows whose period predates the window.
  - Called at startup in `lib.rs` setup, and once per day from the `monitor/alerts.rs` loop (`enforce_retention`).

#### HIGH - PII logging + high-frequency info logs (H2)
- Default `env_logger` filter was `info`, logging process names and per-3s connection counts.
- **Fix**: default filter lowered to `warn`; per-tick connection/capture/`tcp_snapshot`/`tcp6_snapshot` logs demoted to `debug!`.

#### HIGH - Silently swallowed runtime errors (H3)
- DB writes in the save cycle used `let _ = ...`, so failures were silent (data loss masked).
- **Fix** (`monitor/mod.rs`): each insert/upsert result is checked; failures are accumulated and logged via `log::warn!` with count + detail.
- Autostart sync errors in `lib.rs` setup and `commands/settings.rs` now logged instead of ignored.

#### MEDIUM - Over-broad permissions + version drift (M1/M2)
- **Capabilities trimmed** (`src-tauri/capabilities/default.json`): removed unused `set-position`/`set-size`/`center`; added only the used window ops (`maximize`/`unmaximize`/`is-maximized`/`is-visible`).
- **Version drift fixed**: `package.json` bumped 1.1.0 → 1.3.0; `speedtest.rs` user-agent 1.1.0 → 1.3.0 (final release version).

### Files / Components
- `src-tauri/src/commands/update.rs` — checksum + Authenticode verification, pinned repo, version validation
- `src-tauri/Cargo.toml` — `sha2`, WinTrust + Cryptography windows-sys features
- `src-tauri/tauri.conf.json` — enforced CSP
- `src-tauri/src/db/queries.rs` — `purge_old_usage`; `chrono::Datelike` import
- `src-tauri/src/lib.rs` — retention purge on startup, `warn` default log filter, autostart error logging
- `src-tauri/src/monitor/alerts.rs` — `enforce_retention` daily purge
- `src-tauri/src/monitor/mod.rs` — DB write error aggregation/logging
- `src-tauri/src/monitor/app_usage.rs` — `debug!` for per-tick logs
- `src-tauri/src/commands/settings.rs` — autostart error logging
- `src-tauri/capabilities/default.json` — trimmed window permissions
- `src/components/dashboard/Dashboard.tsx` — `apply_update` call signature (version only)
- `package.json` — version 1.3.0
- `src-tauri/src/commands/speedtest.rs` — user-agent 1.3.0

### Verified
- `cargo check` clean (4 pre-existing dead-code warnings, no new ones).
- `tsc -b` clean (no errors).
- `oxlint` clean (only pre-existing warnings).

### Important notes
- Releases must now include a `SHA256: <64-hex>` line in the release body matching the `data-tracker.exe` binary, otherwise self-update will refuse to apply (this is intentional).
- The downloaded exe must be Authenticode-signed and trusted on the target machine to pass `WinVerifyTrust`.

## Mod 1.2.9 - Live App Usage: group by app name (v1.2.9)

**Date:** 2026-09-02

### What was fixed
- **Live App Usage showed more than 8 entries**: The backend sent one entry per PID, so a single app with multiple connections (e.g. Chrome) appeared as multiple rows.
- **Fix**: Frontend now merges entries by app name before slicing to top 8.

### Files / Components
- `src/components/dashboard/LiveAppUsage.tsx` — merge by app name before sort/slice

## Mod 1.2.8 - Dashboard layout: 60/40 split (v1.2.8)

**Date:** 2026-09-02

### What was changed
- **Dashboard layout**: Speed History and Live App Usage now use a 60/40 split (`grid-cols-[3fr_2fr]`).

### Files / Components
- `src/components/dashboard/Dashboard.tsx` — grid layout changed to `lg:grid-cols-[3fr_2fr]`

## Mod 1.2.7 - Live App Usage top 8 + 75/25 dashboard split (v1.2.7)

**Date:** 2026-09-02

### What was changed
- **Live App Usage** now shows top 8 apps (was 10).
- **Dashboard layout**: Speed History and Live App Usage now use a 75/25 split (`grid-cols-[3fr_1fr]`) giving Speed History more space.

### Files / Components
- `src/components/dashboard/Dashboard.tsx` — grid layout changed to `lg:grid-cols-[3fr_1fr]`
- `src/components/dashboard/LiveAppUsage.tsx` — top 8 slice, label updated

## Mod 1.2.6 - Dashboard layout: Live App Usage beside Speed History (v1.2.6)

**Date:** 2026-09-02

### What was changed
- **Live App Usage** moved from below Speed History to beside it in a 2-column grid (`lg:grid-cols-2`). Both cards now match height, no scrollbars.

### Files / Components
- `src/components/dashboard/Dashboard.tsx` — Speed History + Live App Usage in 2-col grid
- `src/components/dashboard/LiveAppUsage.tsx` — `h-full` added, padding matches Speed History

## Mod 1.2.5 - Live App Usage fallback fix (v1.2.5)

**Date:** 2026-09-02

### What was fixed
- **All apps showed as "Unknown"**: Both `live_app_speeds()` and `live_speeds_with_fallback()` read from `self.names` cache, but it was only populated during `flush()` (every 60 seconds). On every 3s tick the cache was stale/empty, so all process names resolved to "Unknown".
- **Fix**: Both methods now call `process_names()` to refresh the PID→name cache before resolving names. Names are immediately available on every tick.

### Files / Components
- `src-tauri/src/monitor/app_usage.rs` — both `live_app_speeds()` and `live_speeds_with_fallback()` now refresh names cache before lookup

### Verified
- `cargo check` clean (4 pre-existing dead-code warnings, no new ones).

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
