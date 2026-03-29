use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::{Arc, OnceLock};
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
use tokio::process::Command;
use tokio::sync::Mutex;

// Global store for active download child processes (keyed by event_id)
fn active_downloads() -> &'static Arc<Mutex<HashMap<String, u32>>> {
    static INSTANCE: OnceLock<Arc<Mutex<HashMap<String, u32>>>> = OnceLock::new();
    INSTANCE.get_or_init(|| Arc::new(Mutex::new(HashMap::new())))
}

// ─── Data types ──────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SpotifyTrack {
    pub title: String,
    pub artist: String,
    pub duration_ms: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchResult {
    pub id: String,
    pub title: String,
    pub channel: String,
    pub duration: String,
    pub thumbnail: String,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DownloadProgress {
    pub percent: f32,
    pub speed: String,
    pub eta: String,
    pub filename: String,
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn format_duration(seconds: f64) -> String {
    let s = seconds as u64;
    let m = s / 60;
    let h = m / 60;
    if h > 0 {
        format!("{}:{:02}:{:02}", h, m % 60, s % 60)
    } else {
        format!("{}:{:02}", m, s % 60)
    }
}

fn parse_progress_line(line: &str) -> DownloadProgress {
    let mut percent = 0.0f32;
    let mut speed = String::new();
    let mut eta = String::new();
    let mut filename = String::new();

    // Parse: [download]   45.2% of   3.50MiB at   1.23MiB/s ETA 00:02
    if let Some(pct_pos) = line.find('%') {
        let start = line[..pct_pos]
            .rfind(|c: char| c.is_whitespace())
            .map(|i| i + 1)
            .unwrap_or(0);
        percent = line[start..pct_pos].trim().parse().unwrap_or(0.0);
    }
    if let Some(at_pos) = line.find("at ") {
        let rest = &line[at_pos + 3..];
        speed = rest.split_whitespace().next().unwrap_or("").to_string();
    }
    if let Some(eta_pos) = line.find("ETA ") {
        let rest = &line[eta_pos + 4..];
        eta = rest.split_whitespace().next().unwrap_or("").to_string();
    }
    if let Some(dest_pos) = line.find("Destination: ") {
        filename = line[dest_pos + 13..].trim().to_string();
    }

    DownloadProgress { percent, speed, eta, filename }
}

fn sidecar_dir() -> std::path::PathBuf {
    // In production: binaries are next to the app executable
    // In dev: they're in src-tauri/binaries/
    let exe = std::env::current_exe().unwrap_or_default();
    let exe_dir = exe.parent().unwrap_or(std::path::Path::new("."));
    exe_dir.to_path_buf()
}

fn extended_path() -> String {
    let home = std::env::var("HOME").unwrap_or_default();
    let current = std::env::var("PATH").unwrap_or_default();
    let sidecar = sidecar_dir().to_string_lossy().to_string();
    format!(
        "{sidecar}:{home}/.local/bin:/opt/homebrew/bin:/usr/local/bin:{current}",
        sidecar = sidecar,
        home = home,
        current = current
    )
}

fn expand_tilde(path: &str) -> String {
    if path.starts_with('~') {
        if let Some(home) = std::env::var_os("HOME") {
            return path.replacen('~', &home.to_string_lossy(), 1);
        }
    }
    path.to_string()
}

// ─── Commands ─────────────────────────────────────────────────────────────────

/// Search YouTube for tracks using yt-dlp
#[tauri::command]
pub async fn search_youtube(query: String, limit: Option<u32>) -> Result<Vec<SearchResult>, String> {
    let count = limit.unwrap_or(10);
    let output = Command::new("yt-dlp")
        .env("PATH", extended_path())
        .args([
            &format!("ytsearch{}:{}", count, query),
            "--dump-json",
            "--flat-playlist",
            "--quiet",
        ])
        .output()
        .await
        .map_err(|e| format!("yt-dlp nicht gefunden. Bitte installieren: brew install yt-dlp\n\nFehler: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if stdout.trim().is_empty() && !output.status.success() {
        return Err(format!("yt-dlp Fehler: {}", stderr.trim()));
    }

    let mut results = Vec::new();

    for line in stdout.lines() {
        if line.trim().is_empty() {
            continue;
        }
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
            let id = json["id"].as_str().unwrap_or("").to_string();
            results.push(SearchResult {
                id: id.clone(),
                title: json["title"].as_str().unwrap_or("Unbekannt").to_string(),
                channel: json["uploader"]
                    .as_str()
                    .or_else(|| json["channel"].as_str())
                    .unwrap_or("Unbekannt")
                    .to_string(),
                duration: format_duration(json["duration"].as_f64().unwrap_or(0.0)),
                thumbnail: json["thumbnail"]
                    .as_str()
                    .map(String::from)
                    .or_else(|| {
                        json["thumbnails"]
                            .as_array()
                            .and_then(|arr| arr.last())
                            .and_then(|t| t["url"].as_str())
                            .map(String::from)
                    })
                    .unwrap_or_default(),
                url: format!("https://www.youtube.com/watch?v={}", id),
            });
        }
    }

    Ok(results)
}

/// Get playlist or single video info
#[tauri::command]
pub async fn get_playlist_info(url: String) -> Result<serde_json::Value, String> {
    let output = Command::new("yt-dlp")
        .env("PATH", extended_path())
        .args([
            "--dump-json",
            "--flat-playlist",
            "--quiet",
            &url,
        ])
        .output()
        .await
        .map_err(|e| format!("yt-dlp Fehler: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut tracks = Vec::new();
    let mut playlist_title = String::new();

    for line in stdout.lines() {
        if line.trim().is_empty() {
            continue;
        }
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
            if playlist_title.is_empty() {
                playlist_title = json["playlist_title"]
                    .as_str()
                    .or_else(|| json["playlist"].as_str())
                    .unwrap_or("")
                    .to_string();
            }
            tracks.push(json);
        }
    }

    Ok(serde_json::json!({
        "tracks": tracks,
        "title": playlist_title
    }))
}

/// Download audio from URL with progress events
#[tauri::command]
pub async fn download_audio(
    app: AppHandle,
    url: String,
    format: String,
    output_dir: String,
    event_id: String,
) -> Result<String, String> {
    let is_original = format == "original";
    let audio_fmt = if format == "aac" { "aac" } else { "mp3" };
    let expanded_dir = expand_tilde(&output_dir);

    // Create output dir if needed
    std::fs::create_dir_all(&expanded_dir).map_err(|e| e.to_string())?;

    let output_template = format!("{}/%(title)s.%(ext)s", expanded_dir);
    let archive_path = format!("{}/.download-archive.txt", expanded_dir);

    let ffmpeg_dir = sidecar_dir().to_string_lossy().to_string();

    let mut args = vec![
        "-x".to_string(),
        "--audio-quality".to_string(), "0".to_string(),
        "--add-metadata".to_string(),
        "--ffmpeg-location".to_string(), ffmpeg_dir,
        "--download-archive".to_string(), archive_path,
        "-o".to_string(), output_template,
        "--newline".to_string(),
        "--progress".to_string(),
    ];

    // Only convert when a specific format is requested
    if !is_original {
        args.push("--audio-format".to_string());
        args.push(audio_fmt.to_string());
    }

    args.push(url.clone());

    let mut child = Command::new("yt-dlp")
        .env("PATH", extended_path())
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("yt-dlp nicht gefunden: {}", e))?;

    // Store PID so we can cancel
    let pid = child.id().unwrap_or(0);
    {
        let mut downloads = active_downloads().lock().await;
        downloads.insert(event_id.clone(), pid);
    }

    let stdout = child.stdout.take().ok_or("Failed to capture stdout")?;
    let stderr = child.stderr.take().ok_or("Failed to capture stderr")?;
    let mut reader = BufReader::new(stdout).lines();
    let mut err_reader = BufReader::new(stderr).lines();

    let app_clone = app.clone();
    let eid = event_id.clone();

    // Collect stderr in background
    let stderr_handle = tokio::spawn(async move {
        let mut errors = Vec::new();
        while let Ok(Some(line)) = err_reader.next_line().await {
            errors.push(line);
        }
        errors
    });

    // Track if this was a skip (already downloaded)
    let mut was_skipped = false;

    // Read stdout and emit progress events
    while let Ok(Some(line)) = reader.next_line().await {
        if line.contains("has already been recorded in the archive") {
            was_skipped = true;
        } else if line.contains("[download]") && line.contains('%') {
            let progress = parse_progress_line(&line);
            let _ = app_clone.emit(&format!("download-progress-{}", eid), progress);
        } else if line.contains("[ExtractAudio]") || line.contains("[ffmpeg]") {
            let _ = app_clone.emit(
                &format!("download-progress-{}", eid),
                DownloadProgress {
                    percent: 95.0,
                    speed: String::new(),
                    eta: "Konvertiere...".to_string(),
                    filename: String::new(),
                },
            );
        }
    }

    let status = child.wait().await.map_err(|e| e.to_string())?;
    let stderr_lines = stderr_handle.await.unwrap_or_default();

    // Remove from active downloads
    {
        let mut downloads = active_downloads().lock().await;
        downloads.remove(&event_id);
    }

    if status.success() || was_skipped {
        let _ = app.emit(
            &format!("download-done-{}", event_id),
            serde_json::json!({ "success": true, "skipped": was_skipped }),
        );
        Ok(expanded_dir)
    } else {
        let err_msg = stderr_lines
            .iter()
            .filter(|l| l.contains("ERROR") || l.contains("error"))
            .cloned()
            .collect::<Vec<_>>()
            .join("\n");
        Err(if err_msg.is_empty() {
            stderr_lines.join("\n")
        } else {
            err_msg
        })
    }
}

/// Cancel an active download by killing the yt-dlp process
#[tauri::command]
pub async fn cancel_download(event_id: String) -> Result<(), String> {
    let mut downloads = active_downloads().lock().await;
    if let Some(pid) = downloads.remove(&event_id) {
        if pid > 0 {
            unsafe { libc::kill(pid as i32, libc::SIGTERM); }
        }
        Ok(())
    } else {
        Ok(())
    }
}

/// Cancel all active downloads
#[tauri::command]
pub async fn cancel_all_downloads() -> Result<(), String> {
    let mut downloads = active_downloads().lock().await;
    for (_, pid) in downloads.drain() {
        if pid > 0 {
            unsafe { libc::kill(pid as i32, libc::SIGTERM); }
        }
    }
    Ok(())
}

/// Download Spotify playlist via spotdl
#[tauri::command]
pub async fn download_spotify_playlist(
    spotify_url: String,
    output_dir: String,
    format: String,
) -> Result<Vec<String>, String> {
    let expanded_dir = expand_tilde(&output_dir);
    std::fs::create_dir_all(&expanded_dir).map_err(|e| e.to_string())?;

    let audio_fmt = if format == "aac" { "m4a" } else { "mp3" };

    let output = Command::new("spotdl")
        .env("PATH", extended_path())
        .args([
            &spotify_url,
            "--format",
            audio_fmt,
            "--output",
            &format!("{}/{{list-name}}/{{title}}.{{output-ext}}", expanded_dir),
        ])
        .current_dir(&expanded_dir)
        .output()
        .await
        .map_err(|e| {
            format!(
                "spotdl nicht gefunden. Bitte installieren: pip3 install spotdl\n\nFehler: {}",
                e
            )
        })?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    let mut lines: Vec<String> = stdout
        .lines()
        .chain(stderr.lines())
        .filter(|l| !l.trim().is_empty())
        .map(String::from)
        .collect();

    if !output.status.success() && lines.is_empty() {
        lines.push("spotdl Fehler. Überprüfe die Spotify URL.".to_string());
    }

    Ok(lines)
}

/// Fetch playlist tracks via Spotify embed page (no API auth needed)
#[tauri::command]
pub async fn fetch_spotify_tracks(playlist_id: String) -> Result<Vec<SpotifyTrack>, String> {
    let url = format!("https://open.spotify.com/embed/playlist/{}", playlist_id);

    let html = reqwest::get(&url)
        .await
        .map_err(|e| format!("Netzwerkfehler: {}", e))?
        .text()
        .await
        .map_err(|e| format!("Lesefehler: {}", e))?;

    // Extract JSON from <script id="__NEXT_DATA__">...</script>
    let marker = r#"<script id="__NEXT_DATA__" type="application/json">"#;
    let start = html.find(marker).ok_or("Embed-Daten nicht gefunden")?;
    let json_start = start + marker.len();
    let json_end = html[json_start..].find("</script>").ok_or("Script-Ende nicht gefunden")? + json_start;
    let json_str = &html[json_start..json_end];

    let data: serde_json::Value = serde_json::from_str(json_str)
        .map_err(|e| format!("JSON-Fehler: {}", e))?;

    let track_list = data["props"]["pageProps"]["state"]["data"]["entity"]["trackList"]
        .as_array()
        .ok_or("Keine Tracks in Embed-Daten gefunden")?;

    let tracks: Vec<SpotifyTrack> = track_list
        .iter()
        .map(|t| SpotifyTrack {
            title: t["title"].as_str().unwrap_or("").to_string(),
            artist: t["subtitle"].as_str().unwrap_or("").to_string(),
            duration_ms: t["duration"].as_u64().unwrap_or(0),
        })
        .collect();

    Ok(tracks)
}

/// Start a temporary HTTP server to capture the Spotify OAuth callback code
#[tauri::command]
pub async fn wait_for_oauth_callback(app: AppHandle) -> Result<String, String> {
    let listener = TcpListener::bind("127.0.0.1:8888")
        .await
        .map_err(|e| format!("Port 8888 belegt: {}", e))?;

    let (mut stream, _) = listener
        .accept()
        .await
        .map_err(|e| format!("Verbindung fehlgeschlagen: {}", e))?;

    let mut buf = vec![0u8; 4096];
    let n = stream
        .read(&mut buf)
        .await
        .map_err(|e| format!("Lesen fehlgeschlagen: {}", e))?;
    let request = String::from_utf8_lossy(&buf[..n]).to_string();

    // Extract code from GET /callback?code=...
    let code = request
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|path| {
            url::Url::parse(&format!("http://localhost{}", path)).ok()
        })
        .and_then(|url| {
            url.query_pairs()
                .find(|(k, _)| k == "code")
                .map(|(_, v)| v.to_string())
        })
        .ok_or_else(|| "Kein Code im Callback gefunden".to_string())?;

    // Send a nice response to the browser
    let html = r#"<!DOCTYPE html><html><body style="font-family:system-ui;text-align:center;padding:60px">
<h1>&#10003; Verbunden!</h1><p>Du kannst dieses Fenster schließen und zur App zurückkehren.</p>
</body></html>"#;
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        html.len(),
        html
    );
    let _ = stream.write_all(response.as_bytes()).await;
    let _ = stream.flush().await;

    // Emit the code to the frontend
    let _ = app.emit("spotify-oauth-code", &code);

    Ok(code)
}

/// Get audio stream URL, fetch first ~200KB as preview file, return local path
#[tauri::command]
pub async fn get_stream_url(query: String) -> Result<String, String> {
    // Step 1: Get direct stream URL from yt-dlp (fast, no download)
    let output = Command::new("yt-dlp")
        .env("PATH", extended_path())
        .args([
            "-f", "worstaudio[ext=m4a]/worstaudio",
            "--get-url",
            "--no-playlist",
            "--quiet",
            &query,
        ])
        .output()
        .await
        .map_err(|e| format!("yt-dlp Fehler: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Stream-URL nicht gefunden: {}", stderr.trim()));
    }

    let stream_url = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if stream_url.is_empty() {
        return Err("Keine Stream-URL gefunden".to_string());
    }

    // Step 2: Download only the first ~200KB using Range header
    let preview_dir = std::env::temp_dir().join("hoerbert-preview");
    std::fs::create_dir_all(&preview_dir).map_err(|e| e.to_string())?;

    // Clean up old previews
    if let Ok(entries) = std::fs::read_dir(&preview_dir) {
        for entry in entries.flatten() {
            let _ = std::fs::remove_file(entry.path());
        }
    }

    let client = reqwest::Client::new();
    let resp = client
        .get(&stream_url)
        .header("Range", "bytes=0-204799")
        .send()
        .await
        .map_err(|e| format!("Download-Fehler: {}", e))?;

    let ext = if stream_url.contains("mime=audio%2Fmp4") || stream_url.contains("mime=audio/mp4") {
        "m4a"
    } else if stream_url.contains("mime=audio%2Fwebm") || stream_url.contains("mime=audio/webm") {
        "webm"
    } else {
        "m4a"
    };

    let preview_path = preview_dir.join(format!("preview.{}", ext));
    let bytes = resp.bytes().await.map_err(|e| format!("Lesefehler: {}", e))?;
    std::fs::write(&preview_path, &bytes).map_err(|e| e.to_string())?;

    Ok(preview_path.to_string_lossy().to_string())
}

/// Open URL in default browser
#[tauri::command]
pub async fn open_url(url: String) -> Result<(), String> {
    Command::new("open")
        .arg(&url)
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Native folder picker dialog
#[tauri::command]
pub async fn select_folder(app: AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;

    // blocking_pick_folder runs the dialog synchronously
    let folder = app.dialog().file().blocking_pick_folder();

    Ok(folder.map(|p| p.to_string()))
}

/// Resolve a path (expand ~)
#[tauri::command]
pub async fn resolve_path(path: String) -> Result<String, String> {
    Ok(expand_tilde(&path))
}

/// List MP3/AAC files in a directory (recursively)
#[tauri::command]
pub async fn list_mp3s(dir: String) -> Result<Vec<String>, String> {
    let expanded = expand_tilde(&dir);
    let base = std::path::PathBuf::from(&expanded);
    let mut files = Vec::new();

    fn scan_dir(dir: &std::path::Path, base: &std::path::Path, files: &mut Vec<String>) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    scan_dir(&path, base, files);
                } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if matches!(ext.to_lowercase().as_str(), "mp3" | "aac" | "m4a" | "flac" | "opus" | "webm" | "ogg" | "wav") {
                        // Show relative path from base dir
                        let rel = path.strip_prefix(base)
                            .unwrap_or(&path)
                            .to_string_lossy()
                            .to_string();
                        files.push(rel);
                    }
                }
            }
        }
    }

    scan_dir(&base, &base, &mut files);
    files.sort();
    Ok(files)
}

/// Read existing files from all 9 hörbert slot folders (0-8), using hoerbert.xml for display names
#[tauri::command]
pub async fn read_hoerbert_slots(hoerbert_dir: String) -> Result<std::collections::HashMap<u8, Vec<String>>, String> {
    let expanded = expand_tilde(&hoerbert_dir);
    let mut slots = std::collections::HashMap::new();

    // Try to parse hoerbert.xml for display names
    // Structure: folder id -> sequence -> source filename
    let mut xml_names: std::collections::HashMap<u8, std::collections::BTreeMap<u32, String>> = std::collections::HashMap::new();

    let xml_path = format!("{}/hoerbert.xml", expanded);
    if let Ok(xml_content) = std::fs::read_to_string(&xml_path) {
        use quick_xml::events::Event;
        use quick_xml::Reader;

        let mut reader = Reader::from_str(&xml_content);
        let mut current_folder: Option<u8> = None;
        let mut current_sequence: Option<u32> = None;
        let mut in_sequence = false;
        let mut in_source = false;
        let mut in_user_label = false;
        let mut current_source = String::new();
        let mut current_label = String::new();

        loop {
            match reader.read_event() {
                Ok(Event::Start(e)) => {
                    match e.name().as_ref() {
                        b"folder" => {
                            for attr in e.attributes().flatten() {
                                if attr.key.as_ref() == b"id" {
                                    if let Ok(val) = String::from_utf8_lossy(&attr.value).parse::<u8>() {
                                        current_folder = Some(val);
                                    }
                                }
                            }
                        }
                        b"sequence" => in_sequence = true,
                        b"source" => in_source = true,
                        b"userLabel" => in_user_label = true,
                        _ => {}
                    }
                }
                Ok(Event::Text(e)) => {
                    if in_sequence {
                        if let Ok(text) = e.unescape() {
                            current_sequence = text.trim().parse().ok();
                        }
                    } else if in_user_label {
                        if let Ok(text) = e.unescape() {
                            current_label = text.trim().to_string();
                        }
                    } else if in_source {
                        if let Ok(text) = e.unescape() {
                            current_source = text.trim().to_string();
                        }
                    }
                }
                Ok(Event::End(e)) => {
                    match e.name().as_ref() {
                        b"sequence" => in_sequence = false,
                        b"source" => in_source = false,
                        b"userLabel" => in_user_label = false,
                        b"item" => {
                            if let (Some(folder), Some(seq)) = (current_folder, current_sequence) {
                                // Prefer userLabel for display, strip leading "..." prefix
                                // Fall back to extracting filename from source path
                                let display_name = if !current_label.is_empty() {
                                    let label = current_label.trim_start_matches("...");
                                    // Extract just the filename part from the label
                                    label.rsplit('/').next().unwrap_or(label).to_string()
                                } else {
                                    current_source
                                        .rsplit('/')
                                        .next()
                                        .unwrap_or(&current_source)
                                        .to_string()
                                };
                                xml_names
                                    .entry(folder)
                                    .or_default()
                                    .insert(seq, display_name);
                            }
                            current_sequence = None;
                            current_source.clear();
                            current_label.clear();
                        }
                        b"folder" => current_folder = None,
                        _ => {}
                    }
                }
                Ok(Event::Eof) => break,
                Err(_) => break,
                _ => {}
            }
        }
    }

    for slot in 0..=8u8 {
        let slot_dir = format!("{}/{}", expanded, slot);
        let mut files = Vec::new();

        if let Ok(entries) = std::fs::read_dir(&slot_dir) {
            // Collect WAV files and sort by numeric name
            let mut wav_files: Vec<(u32, String)> = Vec::new();
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if matches!(ext.to_lowercase().as_str(), "wav" | "mp3" | "aac" | "m4a" | "flac" | "ogg") {
                        if let Some(stem) = path.file_stem().and_then(|n| n.to_str()) {
                            let seq: u32 = stem.parse().unwrap_or(9999);
                            let name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                            wav_files.push((seq, name));
                        }
                    }
                }
            }
            wav_files.sort_by_key(|(seq, _)| *seq);

            // Map to display names from XML if available
            for (seq, raw_name) in &wav_files {
                let display = xml_names
                    .get(&slot)
                    .and_then(|m| m.get(seq))
                    .cloned()
                    .unwrap_or_else(|| raw_name.clone());
                files.push(display);
            }
        }

        // Map folder 0-8 to button 1-9 for the UI
        slots.insert(slot + 1, files);
    }

    Ok(slots)
}

/// Write files to a hörbert slot: convert to WAV, number sequentially, update XML
#[tauri::command]
pub async fn write_to_hoerbert(
    app: AppHandle,
    hoerbert_dir: String,
    slot: u8,
    files: Vec<String>,
    source_dir: String,
) -> Result<(), String> {
    let expanded_dir = expand_tilde(&hoerbert_dir);
    let expanded_src = expand_tilde(&source_dir);

    // UI uses 1-9, hörbert folders are 0-8
    let folder_num = if slot > 0 { slot - 1 } else { 0 };
    let slot_dir = format!("{}/{}", expanded_dir, folder_num);
    std::fs::create_dir_all(&slot_dir).map_err(|e| e.to_string())?;

    // Find the next available sequence number in the slot
    let mut next_seq: u32 = 0;
    if let Ok(entries) = std::fs::read_dir(&slot_dir) {
        for entry in entries.flatten() {
            if let Some(stem) = entry.path().file_stem().and_then(|s| s.to_str()) {
                if let Ok(n) = stem.parse::<u32>() {
                    if n >= next_seq {
                        next_seq = n + 1;
                    }
                }
            }
        }
    }

    // Convert each file to WAV and copy to slot
    let mut xml_items: Vec<u32> = Vec::new(); // track completed sequences

    for file in &files {
        // Check if cancelled
        {
            let downloads = active_downloads().lock().await;
            if downloads.is_empty() && xml_items.len() > 0 {
                // If active_downloads was drained (by cancel_all), stop
                // We check after at least one item so initial empty state doesn't trigger
            }
        }

        let src_path = format!("{}/{}", expanded_src, file);
        let dest_path = format!("{}/{}.WAV", slot_dir, next_seq);

        // Convert to WAV (16-bit, 32kHz, mono) for hörbert
        let child = Command::new("ffmpeg")
            .env("PATH", extended_path())
            .args([
                "-i", &src_path,
                "-acodec", "pcm_s16le",
                "-ar", "32000",
                "-ac", "1",
                "-y",
                &dest_path,
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("ffmpeg Fehler: {}", e))?;

        // Register ffmpeg PID so cancel_all_downloads can kill it
        let ffmpeg_event_id = format!("hoerbert-ffmpeg-{}", next_seq);
        let pid = child.id().unwrap_or(0);
        {
            let mut downloads = active_downloads().lock().await;
            downloads.insert(ffmpeg_event_id.clone(), pid);
        }

        let output = child.wait_with_output().await
            .map_err(|e| format!("ffmpeg Fehler: {}", e))?;

        // Remove from active downloads
        {
            let mut downloads = active_downloads().lock().await;
            downloads.remove(&ffmpeg_event_id);
        }

        if !output.status.success() {
            // If killed by signal (cancelled), return early without error alert
            if output.status.code().is_none() {
                return Err("Abgebrochen".to_string());
            }
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Konvertierung fehlgeschlagen für {}: {}", file, stderr.trim()));
        }

        // Get file size
        let byte_size = std::fs::metadata(&dest_path)
            .map(|m| m.len())
            .unwrap_or(0);

        // Update XML immediately after each file so progress is saved even on crash
        update_hoerbert_xml(&expanded_dir, folder_num, &[(next_seq, src_path, byte_size)])?;

        xml_items.push(next_seq);
        next_seq += 1;

        // Emit progress
        let file_index = xml_items.len();
        let total = files.len();
        let _ = app.emit("hoerbert-write-progress", serde_json::json!({
            "file": file,
            "slot": slot,
            "index": file_index,
            "total": total,
        }));
    }

    Ok(())
}

/// Update or create hoerbert.xml with new items for a folder
fn update_hoerbert_xml(
    hoerbert_dir: &str,
    folder: u8,
    new_items: &[(u32, String, u64)],
) -> Result<(), String> {
    let xml_path = format!("{}/hoerbert.xml", hoerbert_dir);

    // Read existing XML or create new
    let mut xml_content = std::fs::read_to_string(&xml_path).unwrap_or_else(|_| {
        String::from("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<hoerbert>\n\t<hoerbert_playlists>\n\t\t<version>1.0</version>\n\t\t<generator>ladebert</generator>\n\t\t<folders>\n\t\t</folders>\n\t</hoerbert_playlists>\n</hoerbert>")
    });

    // Build new item XML entries
    let mut items_xml = String::new();
    for (seq, source, size) in new_items {
        let guid = format!("{:08X}-{:04X}-{:04X}-{:04X}-{:012X}",
            rand_u32(), rand_u16(), rand_u16(), rand_u16(), rand_u64() & 0xFFFFFFFFFFFF);
        let label = if source.len() > 60 {
            format!("...{}", &source[source.len()-57..])
        } else {
            source.clone()
        };
        items_xml.push_str(&format!(
            "\t\t\t\t\t<item guid=\"{}\">\n\t\t\t\t\t\t<sequence>{}</sequence>\n\t\t\t\t\t\t<source>{}</source>\n\t\t\t\t\t\t<userLabel>{}</userLabel>\n\t\t\t\t\t\t<byteSize>{}</byteSize>\n\t\t\t\t\t</item>\n",
            guid, seq, source, label, size
        ));
    }

    // Check if folder already exists in XML
    let folder_tag = format!("<folder id=\"{}\">", folder);

    if let Some(folder_start_pos) = xml_content.find(&folder_tag) {
        // Find the </items> closing tag within this folder
        if let Some(items_end_pos) = xml_content[folder_start_pos..].find("</items>") {
            let insert_pos = folder_start_pos + items_end_pos;
            xml_content.insert_str(insert_pos, &items_xml);
        }
    } else {
        // Add new folder before </folders>
        let new_folder = format!(
            "\t\t\t<folder id=\"{}\">\n\t\t\t\t<items>\n{}\t\t\t\t</items>\n\t\t\t</folder>\n",
            folder, items_xml
        );
        if let Some(pos) = xml_content.find("</folders>") {
            xml_content.insert_str(pos, &new_folder);
        }
    }

    std::fs::write(&xml_path, &xml_content).map_err(|e| format!("XML schreiben fehlgeschlagen: {}", e))?;
    Ok(())
}

/// Remove folder entries from hoerbert.xml
/// If folder_id is None, remove all folders. If Some(id), remove only that folder.
fn clear_xml_folder(hoerbert_dir: &str, folder_id: Option<u8>) -> Result<(), String> {
    let xml_path = format!("{}/hoerbert.xml", hoerbert_dir);
    let mut xml_content = match std::fs::read_to_string(&xml_path) {
        Ok(c) => c,
        Err(_) => return Ok(()), // no XML, nothing to clear
    };

    match folder_id {
        Some(id) => {
            // Remove specific folder block
            let folder_tag = format!("<folder id=\"{}\">", id);
            if let Some(start) = xml_content.find(&folder_tag) {
                if let Some(end_offset) = xml_content[start..].find("</folder>") {
                    let end = start + end_offset + "</folder>".len();
                    // Also remove trailing whitespace/newline
                    let end = xml_content[end..].find('<').map(|p| end + p).unwrap_or(end);
                    xml_content.replace_range(start..end, "");
                }
            }
        }
        None => {
            // Remove all folder blocks
            if let Some(folders_start) = xml_content.find("<folders>") {
                if let Some(folders_end) = xml_content.find("</folders>") {
                    let inner_start = folders_start + "<folders>".len();
                    xml_content.replace_range(inner_start..folders_end, "\n\t\t");
                }
            }
        }
    }

    std::fs::write(&xml_path, &xml_content)
        .map_err(|e| format!("XML schreiben fehlgeschlagen: {}", e))?;
    Ok(())
}

// Simple random number helpers for GUID generation
fn rand_u32() -> u32 {
    use std::time::SystemTime;
    let t = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default();
    (t.as_nanos() as u32).wrapping_mul(2654435761)
}
fn rand_u16() -> u16 { rand_u32() as u16 }
fn rand_u64() -> u64 {
    let a = rand_u32() as u64;
    let b = rand_u32() as u64;
    (a << 32) | b
}

/// Clear all WAV files from all hörbert slots (keeps folder structure)
#[tauri::command]
pub async fn clear_all_slots(hoerbert_dir: String) -> Result<(), String> {
    let expanded_dir = expand_tilde(&hoerbert_dir);
    for folder in 0..=8u8 {
        let slot_dir = format!("{}/{}", expanded_dir, folder);
        if let Ok(entries) = std::fs::read_dir(&slot_dir) {
            for entry in entries.flatten() {
                if let Some(ext) = entry.path().extension().and_then(|e| e.to_str()) {
                    if ext.eq_ignore_ascii_case("wav") {
                        let _ = std::fs::remove_file(entry.path());
                    }
                }
            }
        }
    }
    // Clear the XML — remove all folder entries
    clear_xml_folder(&expanded_dir, None)?;
    Ok(())
}

/// Get disk space info for a path (total, used, free in bytes)
#[tauri::command]
pub async fn get_disk_space(path: String) -> Result<serde_json::Value, String> {
    let expanded = expand_tilde(&path);
    let stat = unsafe {
        let c_path = std::ffi::CString::new(expanded.as_bytes()).map_err(|e| e.to_string())?;
        let mut buf: libc::statvfs = std::mem::zeroed();
        if libc::statvfs(c_path.as_ptr(), &mut buf) != 0 {
            return Err("Speicherplatz konnte nicht gelesen werden".to_string());
        }
        buf
    };

    let block_size = stat.f_frsize as u64;
    let total = stat.f_blocks as u64 * block_size;
    let free = stat.f_bavail as u64 * block_size;
    let used = total - (stat.f_bfree as u64 * block_size);

    Ok(serde_json::json!({
        "total": total,
        "used": used,
        "free": free,
    }))
}

/// Eject a mounted disk (macOS: diskutil eject)
#[tauri::command]
pub async fn eject_disk(path: String) -> Result<(), String> {
    let output = Command::new("diskutil")
        .args(["eject", &path])
        .output()
        .await
        .map_err(|e| format!("diskutil Fehler: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Auswerfen fehlgeschlagen: {}", stderr.trim()));
    }
    Ok(())
}

/// Delete a single track from a hörbert slot and re-number remaining files
#[tauri::command]
pub async fn delete_slot_track(
    hoerbert_dir: String,
    slot: u8,
    track_index: u32,
) -> Result<(), String> {
    let expanded_dir = expand_tilde(&hoerbert_dir);
    let folder_num = if slot > 0 { slot - 1 } else { 0 };
    let slot_dir = format!("{}/{}", expanded_dir, folder_num);

    let target = format!("{}/{}.WAV", slot_dir, track_index);
    if std::path::Path::new(&target).exists() {
        std::fs::remove_file(&target).map_err(|e| format!("Löschen fehlgeschlagen: {}", e))?;
    }

    // Re-number remaining WAV files to keep sequential 0, 1, 2, ...
    let mut wavs: Vec<u32> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&slot_dir) {
        for entry in entries.flatten() {
            if let Some(ext) = entry.path().extension().and_then(|e| e.to_str()) {
                if ext.eq_ignore_ascii_case("wav") {
                    if let Some(stem) = entry.path().file_stem().and_then(|s| s.to_str()) {
                        if let Ok(n) = stem.parse::<u32>() {
                            wavs.push(n);
                        }
                    }
                }
            }
        }
    }
    wavs.sort();

    for (new_idx, old_idx) in wavs.iter().enumerate() {
        let new_num = new_idx as u32;
        if new_num != *old_idx {
            let old_path = format!("{}/{}.WAV", slot_dir, old_idx);
            let new_path = format!("{}/{}.WAV", slot_dir, new_num);
            std::fs::rename(&old_path, &new_path)
                .map_err(|e| format!("Umbenennen fehlgeschlagen: {}", e))?;
        }
    }

    Ok(())
}

/// Clear all tracks from a hörbert slot
#[tauri::command]
pub async fn clear_slot(
    hoerbert_dir: String,
    slot: u8,
) -> Result<(), String> {
    let expanded_dir = expand_tilde(&hoerbert_dir);
    let folder_num = if slot > 0 { slot - 1 } else { 0 };
    let slot_dir = format!("{}/{}", expanded_dir, folder_num);

    if let Ok(entries) = std::fs::read_dir(&slot_dir) {
        for entry in entries.flatten() {
            if let Some(ext) = entry.path().extension().and_then(|e| e.to_str()) {
                if ext.eq_ignore_ascii_case("wav") {
                    let _ = std::fs::remove_file(entry.path());
                }
            }
        }
    }

    clear_xml_folder(&expanded_dir, Some(folder_num))?;
    Ok(())
}

/// Copy a file to a Hörbert slot folder (legacy, kept for compatibility)
#[tauri::command]
pub async fn copy_to_hoerbert_slot(
    src: String,
    hoerbert_dir: String,
    slot: u8,
) -> Result<(), String> {
    let expanded_src = expand_tilde(&src);
    let expanded_dir = expand_tilde(&hoerbert_dir);

    let folder_num = if slot > 0 { slot - 1 } else { 0 };
    let slot_dir = format!("{}/{}", expanded_dir, folder_num);
    std::fs::create_dir_all(&slot_dir).map_err(|e| e.to_string())?;

    let filename = std::path::Path::new(&expanded_src)
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or("Ungültiger Dateiname")?;

    let dest = format!("{}/{}", slot_dir, filename);
    std::fs::copy(&expanded_src, &dest).map_err(|e| e.to_string())?;

    Ok(())
}
