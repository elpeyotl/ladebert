# Changelog

## v1.5.1 (2026-03-29)

### Verbesserungen
- Playlist-Downloads werden in Playlists/{Name}/ organisiert

## v1.5.0 (2026-03-29)

### Verbesserungen
- Sprachfilter nur noch bei Kategorie-Suchen (Hörspiel, Kinderlieder etc.), nicht bei freier Suche
- Landingpage-Text aktualisiert

## v1.4.0 (2026-03-29)

### Neu
- SD-Karte formatieren (FAT32/MBR) direkt in der App – kompatibel mit Hörbert
- Ordnerstruktur (0-8), hoerbert.xml, info.xml und index.m3u werden automatisch erstellt
- Prominenter grüner Auswerfen-Button
- "SD-Karte kann entfernt werden" Meldung nach dem Auswerfen
- SD-Karten-Name wird im Badge mit SD-Icon angezeigt
- YouTube Playlist-Suche: Suche nach Alben und Playlisten
- Verständliche Fehlermeldung wenn SD-Karte voll ist
- Unvollständige WAV-Dateien werden bei Fehler aufgeräumt
- SD-Karten-State wird beim Start geprüft (kein Geister-Hörbert)

### Bugfixes
- SD-Karte wird jetzt als MBR formatiert (GPT funktionierte nicht mit Hörbert)
- Crash beim Schreiben auf SD-Karte behoben (UTF-8 Umlaute in Dateinamen)
- XML-Sonderzeichen (& < >) werden jetzt korrekt escaped
- Lösch-Dialoge blockieren jetzt zuverlässig (native Tauri-Dialoge)
- Update-Check zeigt nur noch neuere Versionen an

## v1.3.0 (2026-03-29)

### Neu
- SD-Karte formatieren (FAT32, Label "HOERBERT") direkt in der App
- Fortschrittsanzeige während der Formatierung
- SD-Karten-Name wird im Badge angezeigt (z.B. "HOERBERT", "NO NAME")
- Nach Formatierung wird neuer Mount-Pfad automatisch erkannt
- YouTube Playlist-Suche: Suche nach Alben und Playlisten
- Verständliche Fehlermeldung wenn SD-Karte voll ist
- Unvollständige WAV-Dateien werden bei Fehler aufgeräumt

### Bugfixes
- Crash beim Schreiben auf SD-Karte behoben (UTF-8 Umlaute in Dateinamen)
- XML-Sonderzeichen (& < >) werden jetzt korrekt escaped
- Lösch-Dialoge blockieren jetzt zuverlässig (native Tauri-Dialoge)
- Update-Check zeigt nur noch neuere Versionen an

## v1.2.0 (2026-03-29)

### Neu
- YouTube Playlist-Suche: Suche nach Alben und Playlisten direkt in der Suchansicht

### Bugfixes
- Crash beim Schreiben auf SD-Karte behoben (UTF-8 Umlaute in Dateinamen)
- XML-Sonderzeichen (& < >) werden jetzt korrekt escaped
- Lösch-Dialoge blockieren jetzt zuverlässig (native Tauri-Dialoge statt Browser-confirm)

## v1.1.1 (2026-03-29)

### Bugfix
- Crash behoben: App stürzte ab wenn yt-dlp stdout/stderr Pipes nicht erstellt werden konnten

### Neu
- Download-Queue: Mehrere Downloads gleichzeitig in Warteschlange stellen (YouTube + Spotify)
- Update-Check: Hinweis in der Sidebar wenn eine neue Version verfügbar ist
- Download-/Player-Bar verdeckt keine Buttons mehr (dynamisches Padding)
- Quellordner-Name wird im Hörbert-Panel angezeigt
- Standard-Format auf "Original" geändert (schneller, keine Konvertierung)

## v1.0.0 (2026-03-29)

Erster Release der Ladebert App.

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
