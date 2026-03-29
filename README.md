# 🎵 Hörbert Downloader

Lokale macOS Desktop-App zum Downloaden von YouTube-Audio als MP3 für den Hörbert Kinder-Audioplayer.

## Features

- **YouTube** – Suche nach Songs, Artists, oder direkte URL (auch Playlists)
- **Spotify** – Verbinde dein Spotify, importiere Playlists und lade sie herunter
- **Hörbert Manager** – Weise MP3s den 9 Hörbert-Slots zu und schreib sie auf die SD-Karte

## Voraussetzungen

```bash
# 1. Homebrew installieren (falls noch nicht vorhanden)
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# 2. Dependencies installieren
brew install yt-dlp ffmpeg
pip3 install spotdl

# 3. Rust installieren (für Tauri)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 4. Tauri CLI
cargo install tauri-cli --version "^2"
```

## Setup & Start

```bash
npm install
npm run tauri dev
```

## Spotify Setup

1. Geh auf https://developer.spotify.com/dashboard
2. Erstelle eine neue App
3. Redirect URI eintragen: `http://127.0.0.1:8888/callback`
4. Client ID in die App eintragen (Settings → Spotify)

## Build (macOS .app)

```bash
npm run tauri build
```

Die fertige App liegt dann in `src-tauri/target/release/bundle/macos/`
