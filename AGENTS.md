# Data Tracker - Application Memory (Modification Log)

This file is the modification memory for the Data Tracker application. Every change bumps a mod number and adds a new entry. Versioning starts at 1.0.0.

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
