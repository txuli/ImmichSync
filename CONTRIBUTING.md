# Contributing to ImmichSync

Thanks for your interest in improving ImmichSync! This is a small side project, but issues, bug reports, and pull requests are welcome.

## Before you start

- This project is **not affiliated with Immich**. It's an unofficial client that drives the [immich-go](https://github.com/simulot/immich-go) sidecar.
- The code is licensed [CC BY-NC 4.0](LICENSE) (Attribution-NonCommercial). By contributing, you agree your contributions are provided under the same license.
- For anything non-trivial (new features, larger refactors), please open an issue first to discuss the approach before investing time in a PR.

## Project structure

```text
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

## Requirements

- Windows 10/11 (the app relies on Windows-specific APIs: toast notifications, autostart, disk polling).
- [Node.js](https://nodejs.org/) + [pnpm](https://pnpm.io/) (`packageManager` is pinned in `package.json`).
- [Rust](https://www.rust-lang.org/tools/install) (stable toolchain) and the [Tauri v2 prerequisites](https://v2.tauri.app/start/prerequisites/).

## Getting started

```powershell
pnpm install
pnpm tauri dev
```

This starts the Vite dev server and the Tauri app together with hot reload. On first run, open **Config** and point it at a test Immich server URL and API key — avoid testing against a production instance.

### Building a release build locally

```powershell
pnpm tauri build
```

Installers are produced in `src-tauri/target/release/bundle/`.

### Useful checks before submitting

```powershell
pnpm build          # tsc type-check + Vite build
cargo check          # from src-tauri/, quick Rust compile check
cargo fmt            # from src-tauri/, format Rust code
```

There's no automated test suite yet — please describe how you manually tested your change (which OS, which drive/card, dry-run vs. real upload, etc.) in your PR description.

## Making changes

- Keep frontend and backend changes focused — prefer smaller, single-purpose PRs over large ones.
- Follow the existing code style (see neighboring files for conventions in `src/` and `src-tauri/src/`).
- Run `cargo fmt` on Rust changes before committing.
- Update `README.md` if you change behavior, requirements, or the project structure described there.
- If your change affects a currently unimplemented feature (see the "Not implemented yet" section in the README), mention that explicitly.

## Commit messages

Short, descriptive commit messages are appreciated (e.g. `fix: handle missing drive letter on eject`). Conventional Commits style (`feat:`, `fix:`, `chore:`, ...) is used loosely in this repo's history — feel free to follow it, but it's not strictly enforced.

## Submitting a pull request

1. Fork the repo and create a branch off `main`.
2. Make your changes, following the checks above.
3. Open a PR against `main` with a clear description of what changed and why, plus manual testing notes.
4. Be responsive to review feedback — this is a small project maintained in spare time, so review may take a bit.

## Reporting bugs / requesting features

Please open a [GitHub issue](../../issues) with:

- What you expected to happen vs. what actually happened.
- Steps to reproduce (OS build, drive/card type, Immich server version if relevant).
- Relevant logs if available (the app uses `@tauri-apps/plugin-log` / `tauri-plugin-log` — check the app's log output).

## Releasing (maintainers)

Releases are built and signed by GitHub Actions (see [`.github/workflows/release.yml`](.github/workflows/release.yml)) whenever a `v*` tag is pushed. Version numbers must be bumped consistently in `package.json`, `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, and `src-tauri/tauri.conf.json` before tagging.
