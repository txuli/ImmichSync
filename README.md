# ImmichSync

A Windows tray app that watches for newly connected USB drives / SD cards, and offers to upload the photos and videos it finds straight to your [Immich](https://immich.app/) server — grouped into a dated album, no manual `immich-go` commands needed.

> **Disclaimer:** This is an independent, unofficial project. It is **not affiliated with, endorsed by, or sponsored by Immich** (or its authors/maintainers). It talks to a self-hosted Immich server through [immich-go](https://github.com/simulot/immich-go), which is bundled as a sidecar binary.

## Features

- **Automatic drive detection** — runs in the background and watches for removable disks being connected.
- **Native Windows toast notifications** with "Sync" / "Ignore" actions when a new device is detected.
- **One-click upload** via [immich-go](https://github.com/simulot/immich-go), which handles deduplication, EXIF/date extraction, and album assignment. Each sync creates a new album named `ImmichSync - <date> <time>`.
- **Dashboard** with live sync status (idle / syncing / success / error), server connection health, and a recent-activity feed.
- **Run on startup** — optionally launches at login and stays hidden in the tray (no window flash) until you open it or a sync needs your attention.
- **Self-updating** — checks GitHub Releases for new versions and can download/install/relaunch from the Config page.

### Not implemented yet

- The **"Remove Assets from SD Card"** and **"Notifications"** toggles in Config are persisted but not wired to any backend logic yet — the setting is saved, but nothing acts on it.
- There's no UI yet to filter which file extensions get synced (immich-go supports `--include-extensions` / `--exclude-extensions`, just not exposed in the app).

## How it works

1. A background thread polls the list of removable disks (via [`sysinfo`](https://crates.io/crates/sysinfo)).
2. When a new one appears, a Windows toast notification pops up with "Sync" / "Ignore" buttons.
3. Tapping "Sync" runs the bundled `immich-go upload from-folder` sidecar against the drive's mount point, using the server URL / API key you configured.
4. A follow-up notification tells you whether the upload succeeded or failed, and the Dashboard reflects it live.

## Requirements

- Windows 10/11 (toast notifications and autostart use Windows-specific APIs).
- A running Immich instance and an API key with permission to create albums and upload assets.

## Getting started (development)

```powershell
pnpm install
pnpm tauri dev
```

This starts the Vite dev server and the Tauri app together. On first run, open the app, go to **Config**, and paste your Immich server URL and API key.

### Building a release build locally

```powershell
pnpm tauri build
```

Installers are produced in `src-tauri/target/release/bundle/`.

## Project structure

```
src/                     React + TypeScript frontend
  pages/                 Dashboard and Config screens
  components/            Shared UI pieces (nav bar, forms, toggles)
src-tauri/
  src/
    lib.rs               App setup, tray icon, disk-polling loop
    sync.rs               Runs the immich-go sidecar
    notification/         Windows toast notifications + sync-status events
  binaries/              Bundled immich-go sidecar executable
  capabilities/          Tauri v2 permission manifests
.github/workflows/       CI: builds and publishes signed releases on tag push
```

## Releasing / auto-updates

Releases are built and signed by GitHub Actions (see [`.github/workflows/release.yml`](.github/workflows/release.yml)) whenever a `v*` tag is pushed, and published as a GitHub Release with a `latest.json` manifest. The app checks that manifest (see the `updater` block in `src-tauri/tauri.conf.json`) to offer in-app updates from the Config page.

## Tech stack

- [Tauri v2](https://tauri.app/) (Rust) for the desktop shell, tray icon, and native APIs.
- React + TypeScript + Tailwind CSS for the UI.
- [immich-go](https://github.com/simulot/immich-go) as the upload engine.

## License

[CC BY-NC 4.0](LICENSE) — Attribution-NonCommercial.
