<template>
  <div class="spotify">
    <div class="header">
      <div class="header-left">
        <h1>Spotify</h1>
        <span class="badge badge-green" v-if="isConnected">Verbunden</span>
      </div>
      <button v-if="isConnected" class="btn btn-ghost" @click="disconnect">Abmelden</button>
    </div>

    <!-- Not connected -->
    <div v-if="!isConnected" class="connect-screen">
      <div class="connect-card">
        <div class="spotify-logo">
          <svg width="48" height="48" viewBox="0 0 24 24" fill="#22c55e">
            <path d="M12 2C6.477 2 2 6.477 2 12s4.477 10 10 10 10-4.477 10-10S17.523 2 12 2zm4.586 14.424a.622.622 0 0 1-.857.207c-2.348-1.435-5.304-1.76-8.785-.964a.623.623 0 0 1-.277-1.215c3.809-.87 7.077-.496 9.712 1.115.294.181.387.564.207.857zm1.223-2.722a.78.78 0 0 1-1.072.257c-2.687-1.652-6.785-2.131-9.965-1.166a.78.78 0 0 1-.973-.519.78.78 0 0 1 .52-.972c3.632-1.102 8.147-.568 11.234 1.328a.78.78 0 0 1 .256 1.072zm.105-2.835C14.692 8.95 9.375 8.775 6.297 9.71a.937.937 0 1 1-.543-1.794c3.532-1.072 9.404-.865 13.115 1.338a.937.937 0 0 1-.954 1.613z"/>
          </svg>
        </div>
        <h2>Mit Spotify verbinden</h2>
        <p>Importiere deine Playlists und lade sie als MP3 herunter.</p>

        <button
          class="btn btn-primary connect-btn"
          @click="startOAuth"
          :disabled="connecting"
        >
          <span v-if="connecting" class="spinner-green" />
          Mit Spotify verbinden
        </button>

        <div v-if="authCode" class="code-input">
          <p class="code-label">Nach dem Login im Browser, füge den Code hier ein:</p>
          <div class="input-row">
            <input class="input" v-model="manualCode" placeholder="Code aus der URL (nach ?code=)" />
            <button class="btn btn-primary" @click="exchangeCode" :disabled="!manualCode">OK</button>
          </div>
        </div>
      </div>
    </div>

    <!-- Connected -->
    <div v-else class="connected-layout">
      <!-- Playlists sidebar -->
      <div class="playlists-panel">
        <div class="section-title">Meine Playlists</div>
        <div class="search-box">
          <svg class="search-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/>
          </svg>
          <input
            class="search-input"
            v-model="playlistSearch"
            placeholder="Playlist suchen..."
          />
        </div>
        <div v-if="loadingPlaylists" class="loading-rows">
          <div v-for="i in 5" :key="i" class="skeleton-row" />
        </div>
        <div v-else class="playlists-list">
          <button
            v-for="pl in filteredPlaylists"
            :key="pl.id"
            class="playlist-item"
            :class="{ active: selectedPlaylist?.id === pl.id }"
            @click="selectPlaylist(pl)"
          >
            <img v-if="pl.image" :src="pl.image" class="pl-image" />
            <div v-else class="pl-image-placeholder">♪</div>
            <div class="pl-info">
              <div class="pl-name">{{ pl.name }}</div>
              <div class="pl-count">{{ pl.tracks }} Tracks</div>
            </div>
          </button>
        </div>
      </div>

      <!-- Tracks panel -->
      <div class="tracks-panel">
        <div v-if="!selectedPlaylist" class="empty-state">
          <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <path d="M9 18V5l12-2v13M9 9l12-2"/><circle cx="6" cy="18" r="3"/><circle cx="18" cy="16" r="3"/>
          </svg>
          <p>Wähle eine Playlist aus</p>
        </div>

        <template v-else>
          <div class="tracks-header">
            <div>
              <h2>{{ selectedPlaylist.name }}</h2>
              <div class="tracks-count">{{ selectedPlaylist.tracks }} Tracks</div>
            </div>
            <button
              class="btn btn-primary"
              @click="downloadPlaylist"
              :disabled="downloading || downloadState.active"
            >
              <span v-if="downloading" class="spinner" />
              <svg v-else width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4M7 10l5 5 5-5M12 15V3"/>
              </svg>
              Alle herunterladen
            </button>
          </div>

          <div v-if="loadingTracks" class="loading-rows">
            <div v-for="i in 8" :key="i" class="skeleton-row" />
          </div>

          <div v-else-if="tracks.length" class="tracks-list">
            <div v-for="(track, i) in tracks" :key="track.id" class="track-row">
              <span class="track-num">{{ i + 1 }}</span>
              <img v-if="track.image" :src="track.image" class="track-thumb-sm" alt="" />
              <div class="track-info">
                <div class="track-name">{{ track.name }}</div>
                <div class="track-artist">{{ track.artists }}</div>
              </div>
              <span class="track-dur">{{ track.duration }}</span>
              <button
                class="btn btn-ghost track-play-btn"
                @click="playTrack(track.name, track.artists, `ytsearch1:${track.artists} - ${track.name}`)"
                title="Vorhören"
              >
                <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M8 5v14l11-7z"/>
                </svg>
              </button>
              <button
                class="btn btn-primary track-dl-btn"
                @click="downloadSingleTrack(track)"
                :disabled="downloadState.active"
              >
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                  <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4M7 10l5 5 5-5M12 15V3"/>
                </svg>
              </button>
            </div>
          </div>

          <!-- Download progress -->
          <div v-if="downloadLog.length" class="download-log">
            <div class="section-title" style="margin-bottom: 8px">Download Log</div>
            <div class="log-output">
              <div v-for="(line, i) in downloadLog" :key="i" class="log-line">{{ line }}</div>
            </div>
          </div>
        </template>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, inject, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

const settings = inject('settings') as any
const downloadState = inject('downloadState') as any
const playTrack = inject('playTrack') as (title: string, artist: string, query: string) => void

const isConnected = ref(false)
const connecting = ref(false)
const authCode = ref(false)
const manualCode = ref('')
const accessToken = ref('')
const refreshToken = ref('')

const playlists = ref<any[]>([])
const loadingPlaylists = ref(false)
const selectedPlaylist = ref<any>(null)
const tracks = ref<any[]>([])
const loadingTracks = ref(false)
const downloading = ref(false)
const downloadLog = ref<string[]>([])
const playlistSearch = ref('')

const clientId = ref(import.meta.env.VITE_SPOTIFY_CLIENT_ID || '')

const filteredPlaylists = computed(() => {
  const q = playlistSearch.value.toLowerCase().trim()
  if (!q) return playlists.value
  return playlists.value.filter((p: any) => p.name.toLowerCase().includes(q))
})

function formatMs(ms: number) {
  const s = Math.floor(ms / 1000)
  const m = Math.floor(s / 60)
  return `${m}:${(s % 60).toString().padStart(2, '0')}`
}

// PKCE helpers
function generateCodeVerifier(): string {
  const array = new Uint8Array(64)
  crypto.getRandomValues(array)
  return btoa(String.fromCharCode(...array))
    .replace(/\+/g, '-')
    .replace(/\//g, '_')
    .replace(/=/g, '')
    .slice(0, 128)
}

async function generateCodeChallenge(verifier: string): Promise<string> {
  const data = new TextEncoder().encode(verifier)
  const digest = await crypto.subtle.digest('SHA-256', data)
  return btoa(String.fromCharCode(...new Uint8Array(digest)))
    .replace(/\+/g, '-')
    .replace(/\//g, '_')
    .replace(/=/g, '')
}

async function startOAuth() {
  connecting.value = true
  const verifier = generateCodeVerifier()
  const challenge = await generateCodeChallenge(verifier)
  localStorage.setItem('spotify_verifier', verifier)

  const params = new URLSearchParams({
    response_type: 'code',
    client_id: clientId.value,
    scope: 'playlist-read-private playlist-read-collaborative',
    redirect_uri: 'http://127.0.0.1:8888/callback',
    code_challenge_method: 'S256',
    code_challenge: challenge,
  })

  const url = `https://accounts.spotify.com/authorize?${params}`

  // Start callback server, then open browser
  const codePromise = invoke<string>('wait_for_oauth_callback')

  await invoke('open_url', { url }).catch(() => {
    navigator.clipboard.writeText(url)
  })

  try {
    const code = await codePromise
    manualCode.value = code
    await exchangeCode()
  } catch {
    // Server failed — fall back to manual code entry
    authCode.value = true
  }
  connecting.value = false
}

async function exchangeCode() {
  const verifier = localStorage.getItem('spotify_verifier') || ''
  const code = manualCode.value.trim()

  const res = await fetch('https://accounts.spotify.com/api/token', {
    method: 'POST',
    headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
    body: new URLSearchParams({
      client_id: clientId.value,
      grant_type: 'authorization_code',
      code,
      redirect_uri: 'http://127.0.0.1:8888/callback',
      code_verifier: verifier,
    }),
  })

  const data = await res.json()
  if (data.access_token) {
    accessToken.value = data.access_token
    refreshToken.value = data.refresh_token
    localStorage.setItem('spotify_access', data.access_token)
    localStorage.setItem('spotify_refresh', data.refresh_token)
    isConnected.value = true
    authCode.value = false
    await loadPlaylists()
  } else {
    alert('Fehler beim Token-Austausch: ' + JSON.stringify(data))
  }
}

async function refreshAccessToken(): Promise<boolean> {
  const refresh = refreshToken.value || localStorage.getItem('spotify_refresh')
  if (!refresh || !clientId.value) return false

  const res = await fetch('https://accounts.spotify.com/api/token', {
    method: 'POST',
    headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
    body: new URLSearchParams({
      client_id: clientId.value,
      grant_type: 'refresh_token',
      refresh_token: refresh,
    }),
  })

  const data = await res.json()
  if (data.access_token) {
    accessToken.value = data.access_token
    localStorage.setItem('spotify_access', data.access_token)
    if (data.refresh_token) {
      refreshToken.value = data.refresh_token
      localStorage.setItem('spotify_refresh', data.refresh_token)
    }
    return true
  }
  return false
}

async function loadPlaylists() {
  loadingPlaylists.value = true
  let res = await fetch('https://api.spotify.com/v1/me/playlists?limit=50', {
    headers: { Authorization: `Bearer ${accessToken.value}` },
  })

  // Token abgelaufen — refreshen und nochmal versuchen
  if (res.status === 401) {
    const refreshed = await refreshAccessToken()
    if (refreshed) {
      res = await fetch('https://api.spotify.com/v1/me/playlists?limit=50', {
        headers: { Authorization: `Bearer ${accessToken.value}` },
      })
    } else {
      disconnect()
      loadingPlaylists.value = false
      return
    }
  }

  const data = await res.json()
  playlists.value = (data.items || [])
    .filter((p: any) => p && p.id)
    .map((p: any) => ({
      id: p.id,
      name: p.name || 'Unbenannt',
      tracks: p.items?.total ?? p.tracks?.total ?? 0,
      tracksHref: p.items?.href || p.tracks?.href || '',
      image: p.images?.[0]?.url || '',
      spotifyUrl: p.external_urls?.spotify || '',
    }))
  loadingPlaylists.value = false
}

async function selectPlaylist(pl: any) {
  selectedPlaylist.value = pl
  downloadLog.value = []
  loadingTracks.value = true
  tracks.value = []

  try {
    const result = await invoke<any[]>('fetch_spotify_tracks', { playlistId: pl.id })
    tracks.value = result.map((t: any) => ({
      id: t.title,
      name: t.title,
      artists: t.artist,
      duration: formatMs(t.duration_ms),
      image: '',
    }))
  } catch (e) {
    console.error('Track-Laden fehlgeschlagen:', e)
  }

  loadingTracks.value = false
}

async function downloadPlaylist() {
  if (!selectedPlaylist.value || !tracks.value.length) return
  if (downloadState.active) return

  downloading.value = true
  downloadLog.value = []

  const outputDir = settings.downloadDir || '~/Downloads/Hörbert'
  const fmt = settings.format || 'mp3'
  const playlistName = selectedPlaylist.value.name
  const total = tracks.value.length

  downloadState.active = true
  downloadState.cancelled = false
  downloadState.totalTracks = total
  downloadState.currentIndex = 0
  downloadState.trackPercent = 0
  downloadState.speed = ''
  downloadState.eta = ''
  downloadState.currentTrack = ''
  downloadState.log = []

  for (let i = 0; i < total; i++) {
    if (downloadState.cancelled) break

    const track = tracks.value[i]
    const query = `${track.artists} - ${track.name}`

    downloadState.currentIndex = i
    downloadState.trackPercent = 0
    downloadState.currentTrack = query

    try {
      const eventId = `spotify-${Date.now()}-${i}`
      downloadState.currentEventId = eventId

      const unlisten1 = await listen<any>(`download-progress-${eventId}`, (event: any) => {
        downloadState.trackPercent = event.payload.percent || 0
        downloadState.speed = event.payload.speed || ''
        downloadState.eta = event.payload.eta || ''
      })
      const unlisten2 = await listen(`download-done-${eventId}`, () => {
        unlisten1()
        unlisten2()
      })

      await invoke('download_audio', {
        url: `ytsearch1:${query}`,
        format: fmt,
        outputDir: `${outputDir}/${playlistName}`,
        eventId,
      })
      downloadLog.value.push(`✓ ${query}`)
    } catch (e: any) {
      if (downloadState.cancelled) break
      downloadLog.value.push(`✗ ${query}: ${e}`)
    }
  }

  downloadState.currentIndex = total
  downloadState.currentTrack = downloadState.cancelled ? 'Abgebrochen' : 'Fertig!'
  if (!downloadState.cancelled) {
    downloadLog.value.push(`Fertig! ${total} Tracks verarbeitet.`)
  } else {
    downloadLog.value.push(`Abgebrochen nach ${downloadState.currentIndex} von ${total} Tracks.`)
  }

  setTimeout(() => {
    downloadState.active = false
    downloadState.cancelled = false
  }, 3000)

  downloading.value = false
}

async function downloadSingleTrack(track: any) {
  if (downloadState.active) return

  const outputDir = settings.downloadDir || '~/Downloads/Hörbert'
  const fmt = settings.format || 'mp3'
  const query = `${track.artists} - ${track.name}`
  const playlistName = selectedPlaylist.value?.name || 'Spotify'

  downloadState.active = true
  downloadState.cancelled = false
  downloadState.totalTracks = 1
  downloadState.currentIndex = 0
  downloadState.trackPercent = 0
  downloadState.speed = ''
  downloadState.eta = ''
  downloadState.currentTrack = query

  try {
    const eventId = `spotify-single-${Date.now()}`

    downloadState.currentEventId = eventId

    const unlisten1 = await listen<any>(`download-progress-${eventId}`, (event) => {
      downloadState.trackPercent = event.payload.percent || 0
      downloadState.speed = event.payload.speed || ''
      downloadState.eta = event.payload.eta || ''
    })
    const unlisten2 = await listen(`download-done-${eventId}`, () => {
      unlisten1()
      unlisten2()
    })

    await invoke('download_audio', {
      url: `ytsearch1:${query}`,
      format: fmt,
      outputDir: `${outputDir}/${playlistName}`,
      eventId,
    })
    downloadState.currentIndex = 1
    downloadState.currentTrack = downloadState.cancelled ? 'Abgebrochen' : 'Fertig!'
  } catch (e: any) {
    if (!downloadState.cancelled) {
      console.error(`Download failed: ${query}`, e)
    }
  }

  setTimeout(() => {
    downloadState.active = false
    downloadState.cancelled = false
  }, 3000)
}

function disconnect() {
  accessToken.value = ''
  refreshToken.value = ''
  localStorage.removeItem('spotify_access')
  localStorage.removeItem('spotify_refresh')
  isConnected.value = false
  playlists.value = []
  selectedPlaylist.value = null
}

onMounted(() => {
  const token = localStorage.getItem('spotify_access')
  if (token) {
    accessToken.value = token
    isConnected.value = true
    loadPlaylists()
  }
})
</script>

<style scoped>
.spotify {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

.header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 20px 24px 16px;
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 10px;
}

.header-left h1 { font-size: 18px; font-weight: 600; }

/* Connect screen */
.connect-screen {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
}

.connect-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
  padding: 48px;
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  max-width: 460px;
  text-align: center;
}

.connect-card h2 { font-size: 20px; font-weight: 600; }
.connect-card p { color: var(--text-muted); font-size: 14px; }

.connect-btn {
  padding: 12px 32px;
  font-size: 15px;
  margin-top: 8px;
}

.client-id-hint {
  background: var(--accent-dim);
  border: 1px solid rgba(245, 158, 11, 0.3);
  border-radius: var(--radius);
  padding: 16px;
  font-size: 13px;
  color: var(--accent);
  text-align: left;
}

.hint-sub {
  color: var(--text-muted);
  margin-top: 8px;
  font-size: 12px;
}

.hint-sub code {
  font-family: 'DM Mono', monospace;
  background: var(--bg-hover);
  padding: 2px 6px;
  border-radius: 4px;
}

.code-input {
  width: 100%;
  text-align: left;
}

.code-label {
  font-size: 12px;
  color: var(--text-muted);
  margin-bottom: 8px;
}

.input-row { display: flex; gap: 8px; }

/* Connected layout */
.connected-layout {
  flex: 1;
  display: flex;
  overflow: hidden;
}

.playlists-panel {
  width: 240px;
  border-right: 1px solid var(--border);
  padding: 20px 12px;
  overflow-y: auto;
  flex-shrink: 0;
}

.search-box {
  position: relative;
  margin: 10px 0;
}

.search-icon {
  position: absolute;
  left: 10px;
  top: 50%;
  transform: translateY(-50%);
  color: var(--text-faint);
  pointer-events: none;
}

.search-input {
  width: 100%;
  padding: 7px 10px 7px 32px;
  border-radius: var(--radius);
  border: 1px solid var(--border);
  background: var(--bg);
  color: var(--text);
  font-size: 12px;
  outline: none;
  transition: border-color 0.15s;
}

.search-input:focus {
  border-color: var(--accent);
}

.search-input::placeholder {
  color: var(--text-faint);
}

.playlists-list { display: flex; flex-direction: column; gap: 2px; }

.playlist-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 10px;
  border-radius: var(--radius);
  border: none;
  background: transparent;
  color: var(--text);
  cursor: pointer;
  transition: all 0.15s;
  text-align: left;
  width: 100%;
}

.playlist-item:hover { background: var(--bg-hover); }
.playlist-item.active { background: var(--accent-dim); color: var(--accent); }

.pl-image {
  width: 36px;
  height: 36px;
  border-radius: 4px;
  object-fit: cover;
  flex-shrink: 0;
}

.pl-image-placeholder {
  width: 36px;
  height: 36px;
  background: var(--border);
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-faint);
  flex-shrink: 0;
}

.pl-name {
  font-size: 13px;
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.pl-count { font-size: 11px; color: var(--text-muted); margin-top: 2px; }

/* Tracks panel */
.tracks-panel {
  flex: 1;
  overflow-y: auto;
  padding: 20px 24px;
  display: flex;
  flex-direction: column;
}

.tracks-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 20px;
}

.tracks-header h2 { font-size: 18px; font-weight: 600; }
.tracks-count { font-size: 12px; color: var(--text-muted); margin-top: 4px; }

.tracks-list { display: flex; flex-direction: column; gap: 2px; }

.track-row {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 10px;
  border-radius: var(--radius);
  transition: background 0.15s;
}

.track-row:hover { background: var(--bg-card); }

.track-row:hover .track-play-btn,
.track-row:hover .track-dl-btn {
  opacity: 1;
}

.track-num {
  width: 24px;
  text-align: right;
  font-family: 'DM Mono', monospace;
  font-size: 12px;
  color: var(--text-faint);
  flex-shrink: 0;
}

.track-thumb-sm {
  width: 36px;
  height: 36px;
  border-radius: 4px;
  object-fit: cover;
  flex-shrink: 0;
}

.track-info { flex: 1; min-width: 0; }
.track-name {
  font-size: 14px;
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.track-artist { font-size: 12px; color: var(--text-muted); margin-top: 2px; }

.track-dur {
  font-family: 'DM Mono', monospace;
  font-size: 12px;
  color: var(--text-muted);
  flex-shrink: 0;
}

.track-play-btn {
  padding: 6px 8px;
  font-size: 12px;
  flex-shrink: 0;
  opacity: 0;
  transition: opacity 0.15s;
  color: var(--accent);
  border-color: var(--border);
}

.track-play-btn:hover:not(:disabled) {
  color: var(--accent);
  background: var(--accent-dim);
  border-color: var(--accent);
}

.track-dl-btn {
  padding: 6px 10px;
  font-size: 12px;
  flex-shrink: 0;
  opacity: 0;
  transition: opacity 0.15s;
}

/* Download log */
.download-log {
  margin-top: 24px;
  padding: 16px;
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: var(--radius);
}

.log-output {
  max-height: 200px;
  overflow-y: auto;
  font-family: 'DM Mono', monospace;
  font-size: 11px;
  color: var(--text-muted);
}

.log-line { padding: 1px 0; }

/* Skeleton loader */
.loading-rows { display: flex; flex-direction: column; gap: 8px; }
.skeleton-row {
  height: 44px;
  border-radius: var(--radius);
  background: linear-gradient(90deg, var(--bg-card) 25%, var(--bg-hover) 50%, var(--bg-card) 75%);
  background-size: 200% 100%;
  animation: shimmer 1.2s infinite;
}

@keyframes shimmer {
  0% { background-position: 200% 0; }
  100% { background-position: -200% 0; }
}

.spinner {
  width: 12px;
  height: 12px;
  border: 2px solid rgba(0,0,0,0.3);
  border-top-color: #000;
  border-radius: 50%;
  animation: spin 0.6s linear infinite;
  display: inline-block;
}

.spinner-green {
  width: 14px;
  height: 14px;
  border: 2px solid rgba(0,0,0,0.2);
  border-top-color: #000;
  border-radius: 50%;
  animation: spin 0.6s linear infinite;
  display: inline-block;
}

@keyframes spin { to { transform: rotate(360deg); } }
</style>
