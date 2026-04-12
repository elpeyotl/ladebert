mod commands;

use commands::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .invoke_handler(tauri::generate_handler![
            search_youtube,
            search_youtube_playlists,
            get_playlist_info,
            download_audio,
            download_spotify_playlist,
            get_stream_url,
            open_url,
            fetch_spotify_tracks,
            wait_for_oauth_callback,
            select_folder,
            resolve_path,
            list_mp3s,
            read_hoerbert_slots,
            write_to_hoerbert,
            copy_to_hoerbert_slot,
            clear_all_slots,
            eject_disk,
            format_sd_card,
            delete_slot_track,
            clear_slot,
            cancel_download,
            cancel_all_downloads,
            get_disk_space,
        ])
        .run(tauri::generate_context!())
        .expect("Fehler beim Starten der App");
}
