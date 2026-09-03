# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html)
(pre-1.0, so minor bumps may include breaking changes).

## [1.0.0] - 2026-09-03

### Added

- Known devices page now lists every device actually saved in SQLite (was a static mock row) with an editable album name, a "Direct upload" toggle, and a Remove button that deletes the device from the database.
- Sync history and upload totals are now persisted: every finished sync (success or error) is written to the `activity` table and successful uploads fold into running totals in `stats`, so the Dashboard's "Recent activity" feed survives app restarts instead of resetting on every launch.
- "Remove Assets from SD Card" is now wired up end to end: once enabled, `sync_assets` deletes the uploaded media files from the device folder after immich-go confirms the upload succeeded.
- The Immich credentials section on the Config page shows a connected/disconnected status card (checkmark, warning, or spinner) once configured, with an Edit/Cancel flow instead of an always-open form.
- The Immich URL field validates in real time against a regex and shows a checkmark or a red border/message — trailing slashes are rejected.
- The app checks for available updates automatically on startup, instead of only when the Config page's "Check for updates" button is pressed.

### Changed

- Sync status (connection indicator, current sync, history) and the update-check result now live in shared Zustand stores (`store/syncStore.ts`, `store/updateStore.ts`) instead of being duplicated across separate `listen("sync-status", ...)` subscriptions in `navBar.tsx` and `dashboard.tsx`.
- Saving Immich credentials now verifies the connection first and only saves if it succeeds; the separate "Test connection" button was removed.
- Shared TypeScript types (`ValidResponse`, `SyncStatusEvent`, `StoredFlag`, `View`, `DbDevice`, ...) moved out of individual components into `src/types/`, replacing several duplicated inline type literals.
- Known devices, Config, and Manual upload pages restyled to match the rest of the app's dark theme and shared components (`ImmichForm`, `Options`, device icon).

### Fixed

- The `devices` table's album column is named `albumName` in SQLite, but the known-devices list read it as `album_name`, so the album field always showed empty.

## [0.2.2] - 2026-09-02

### Fixed

- `immichsync.db` was never created: `tauri-plugin-sql`'s `load`/`execute` commands had no permission granted in `capabilities/default.json`, so `Database.load()` was silently rejected by Tauri's ACL on every call, including the one inside the new-device form that was supposed to bootstrap the database in the first place. Added `sql:default` and `sql:allow-execute`.
- The database is now also initialized eagerly on app startup (`App.tsx`) instead of only lazily inside the new-device form, so the drive-scan loop has a database to query even before any device has gone through that flow.
- `scan.rs` no longer swallows a failed `get_pool()` silently; the error is now logged.

### Changed

- Default sync album name simplified to `ImmichSync` (was a dated `ImmichSync - YYYY-MM-DD HH-MM` placeholder) in both `sync_assets`'s fallback and the manual upload page's placeholder text.

## [0.2.1] - 2026-09-02

### Added

- SQLite-backed device registry (`tauri-plugin-sql` + `sqlx`) storing each recognized drive's name, mount path, destination album, and auto-upload preference.
- New-device flow: pick a destination album from the app before the first sync, or trigger a quick upload straight away from the toast notification.
- Known devices reconnecting show a "Device reconnected" notification that syncs straight to the previously saved album, or uploads automatically when "upload without asking" is enabled.

### Changed

- `sync_assets` now takes an optional album name, falling back to a dated album (`ImmichSync - YYYY-MM-DD HH-MM`) when none is given.
- Renamed notification modules for clarity: `known_device` (reconnected, already-configured devices) vs. `new_device` (unrecognized devices) — previously both misleadingly named `newDevice`/`noneDevice`.
- Manual upload page translated to English.

### Fixed

- The `v0.2.0` release was built before the app's own version number was bumped, so its installer and `latest.json` both reported `0.1.2` — the auto-updater never saw it as newer. `0.2.0` is skipped; this release carries the same changes with the version numbers actually bumped.

## [0.1.3] - 2026-08-26

### Added

- Manual upload page, reachable from the navbar, to trigger uploads on demand (via the `immich-go` sidecar) without waiting for automatic drive detection.
- Native dialog plugin integration for the manual upload flow.

### Changed

- Refreshed navbar styling.

## [0.1.2] - 2026-08-17

### Changed

- Improved window controls, added a window title, and a proper close/hide-to-tray flow.
- Refreshed the navbar style.

## [0.1.1] - 2026-08-17

### Added

- Automatic drive/SD card detection with Windows toast notifications ("Sync" / "Ignore").
- Upload flow via the bundled `immich-go` sidecar, with dated album creation.
- Dashboard with live sync status, server health, and recent-activity feed.
- Run-on-startup and self-updating (GitHub Releases + `latest.json` manifest).
- Config persistence via `tauri-plugin-store`.

[1.0.0]: https://github.com/txuli/ImmichSync/compare/v0.2.2...v1.0.0
[0.2.2]: https://github.com/txuli/ImmichSync/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/txuli/ImmichSync/compare/v0.1.3...v0.2.1
[0.1.3]: https://github.com/txuli/ImmichSync/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/txuli/ImmichSync/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/txuli/ImmichSync/releases/tag/v0.1.1
