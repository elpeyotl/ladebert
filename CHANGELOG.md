# Changelog

## v1.0.0 (2026-03-29)

Erster Release der Hörbert Downloader App.

### YouTube
- Suche nach Songs, Hörspielen und Kindercontent
- Kategorie-Chips: Hörspiel, Kinderlieder, Gute-Nacht, Märchen, Wissen
- Dauer-Filter: Alle, < 5 Min, 5-30 Min, > 30 Min
- Schnell-Suchen mit beliebten Vorschlägen (Bibi Blocksberg, Die drei ???, etc.)
- Dynamische Placeholder-Texte je nach Kategorie
- URL/Playlist-Modus für direkte YouTube-Links
- "Mehr Ergebnisse laden" Pagination
- "Alle herunterladen" Button für Suchergebnisse
- Automatische Ordner-Organisation nach Kategorie (Hörspiel/, Märchen/, etc.)

### Spotify
- OAuth PKCE Verbindung (kein Client-Secret nötig)
- Playlist-Import und Track-Übersicht
- Einzel- und Playlist-Download über YouTube-Suche
- Download-Log mit Fortschritt

### Downloads
- Formate: Original (schneller, keine Konvertierung), MP3, AAC
- Download abbrechen (Cancel-Button in der Download-Bar)
- Bereits heruntergeladene Songs automatisch überspringen (--download-archive)
- Fortschrittsanzeige mit Speed und ETA
- Spracheinstellung (Deutsch/English/Keine) für Suchergebnisse

### Hörbert Manager
- 9-Slot System passend zum physischen Hörbert
- SD-Karte lesen, beschreiben und auswerfen
- Drag & Drop von Downloads in Slots
- Automatische WAV-Konvertierung (PCM 16-bit, 32kHz, Mono)
- Speicherplatz-Anzeige (frei / total)
- Loading-Spinner beim Laden der SD-Karte
- Tracks abspielen, löschen und umsortieren

### Allgemein
- Tauri 2 Desktop-App (Rust + Vue 3)
- Integrierter Audio-Player zum Vorhören
- Dark Theme
- Hörbert-Icon
- Einstellungen: Download-Ordner, Audio-Format, Suchsprache
