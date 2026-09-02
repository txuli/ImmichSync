# Security Policy

## Supported versions

ImmichSync is a small, actively-developed side project. Only the latest published
release is supported — please make sure you're on the newest version (Config page,
or [GitHub Releases](../../releases)) before reporting an issue.

## Reporting a vulnerability

Please **do not open a public issue** for security vulnerabilities (e.g. anything
that could leak your Immich API key, allow arbitrary file access outside the
intended drive, or let the updater install unsigned/untrusted binaries).

Instead, report it privately using one of these options:

- [GitHub Security Advisories](../../security/advisories/new) for this repository (preferred).
- Email the maintainer directly (see the GitHub profile at [github.com/txuli](https://github.com/txuli)).

When reporting, please include:

- A description of the vulnerability and its potential impact.
- Steps to reproduce (OS build, app version, relevant config).
- Any logs or proof-of-concept, if available.

You should get an acknowledgement within a few days. This is a spare-time project,
so a fix timeline can't be guaranteed, but valid reports will be prioritized over
regular feature work.

## Scope notes

- ImmichSync stores your Immich server URL and API key locally via
  `tauri-plugin-store`; it does not transmit them anywhere except the Immich
  server you configure.
- Uploads are performed by the bundled [`immich-go`](https://github.com/simulot/immich-go)
  sidecar binary — vulnerabilities specific to `immich-go` itself should be
  reported upstream.
- Auto-updates are fetched from this repository's GitHub Releases and verified
  against the signature configured in `tauri.conf.json`.
