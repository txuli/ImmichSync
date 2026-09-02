# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html)
(pre-1.0, so minor bumps may include breaking changes).

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

[0.2.2]: https://github.com/txuli/ImmichSync/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/txuli/ImmichSync/compare/v0.1.3...v0.2.1
[0.1.3]: https://github.com/txuli/ImmichSync/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/txuli/ImmichSync/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/txuli/ImmichSync/releases/tag/v0.1.1
