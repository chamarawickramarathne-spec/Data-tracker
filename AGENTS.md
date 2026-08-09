# Data Tracker - Application Memory (Modification Log)

This file is the modification memory for the Data Tracker application. Every change bumps a mod number and adds a new entry. Versioning starts at 1.0.0.

## Mod 1.0.2 - Single-instance enforcement (v1.0.2)

**Date:** 2026-08-09

### What was fixed
- **Second launch spawned a new instance**: launching `data-tracker.exe` while the app was already running (e.g. tray-hidden) started a duplicate process and the tray icon/DB could conflict.
- **Fix**: added `tauri-plugin-single-instance` (`src-tauri/Cargo.toml`), registered first in `src-tauri/src/lib.rs`. A second launch now exits immediately and the existing "main" window is shown, unminimized and focused.

### Verified
- `pnpm lint` and `tsc -b` pass. Rust compiles clean via `pnpm tauri:build`.
- Built `data-tracker.exe` + `DataTracker_1.0.2_x64-setup.exe` copied to `releases/`.
- Released via git: annotated tag `v1.0.2` pushed; GitHub Release `v1.0.2` created with `releases/data-tracker.exe` asset so installed 1.0.1 copies auto-update. `medial_support.txt` regenerated.

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
