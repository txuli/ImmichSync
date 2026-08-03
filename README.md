# ImmichSync

A small Windows utility that watches for newly connected USB drives, scans them for photos, and uploads any it finds to an [Immich](https://immich.app/) instance, grouping them into a new album per import.

> **Disclaimer:** This is an independent, unofficial project. It is **not affiliated with, endorsed by, or sponsored by Immich** (or its authors/maintainers). It simply talks to a self-hosted Immich server through its public REST API.

## How it works

1. `main.py` polls the list of active drive letters on Windows and detects when a new one appears (e.g. plugging in a USB stick or SD card).
2. When a new drive is detected, a confirmation dialog is shown before doing anything.
3. If confirmed, the drive is walked recursively and every file matching a known image extension is uploaded to Immich.
4. A new album is created for the import, and all uploaded assets are assigned to it.

## Project structure

- `main.py` — drive detection loop and the main scanning/upload flow.
- `asset.py` — Immich API client: creating albums, uploading assets, and assigning assets to an album.
- `metadata.py` — EXIF metadata extraction (including GPS and sub-IFDs) and derivation of creation/modification timestamps for each photo.

## Requirements

- Windows (uses `pywin32` and drive-letter detection APIs).
- Python 3.
- A running Immich instance and an API key with permission to create albums and upload assets.

Install dependencies:

```bash
pip install -r requirements.txt
```

## Configuration

Copy `.env.example` to `.env` and fill in your own Immich instance URL and API key:

```bash
cp .env.example .env
```

```
IMMICH_URL=https://your-immich-instance.example.com
IMMICH_API_KEY=your-api-key-here
```

The `.env` file is git-ignored and should never be committed.

## Usage

```bash
python main.py
```

The script runs in a loop, checking for newly connected drives every few seconds. When one is found, confirm the prompt to start scanning and uploading its photos to Immich.
