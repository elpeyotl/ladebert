<template>
  <div class="app">
    <!-- Sidebar -->
    <aside class="sidebar">
      <div class="sidebar-logo">
        <svg width="32" height="32" viewBox="0 0 512 512" fill="none">
          <rect width="512" height="512" rx="80" fill="#c4935a"/>
          <rect x="32" y="50" width="448" height="420" rx="20" fill="#f5edd4"/>
          <!-- Speaker grille -->
          <circle cx="186" cy="250" r="120" fill="#e8dfc6"/>
          <g fill="#3d3225" opacity="0.6">
            <circle cx="136" cy="200" r="8"/><circle cx="176" cy="200" r="8"/><circle cx="216" cy="200" r="8"/><circle cx="256" cy="200" r="8"/>
            <circle cx="116" cy="240" r="8"/><circle cx="156" cy="240" r="8"/><circle cx="196" cy="240" r="8"/><circle cx="236" cy="240" r="8"/><circle cx="276" cy="240" r="8"/>
            <circle cx="136" cy="280" r="8"/><circle cx="176" cy="280" r="8"/><circle cx="216" cy="280" r="8"/><circle cx="256" cy="280" r="8"/>
          </g>
          <!-- 9 buttons 3x3 -->
          <circle cx="358" cy="150" r="24" fill="#e53935"/>
          <circle cx="408" cy="150" r="24" fill="#1e88e5"/>
          <circle cx="458" cy="150" r="24" fill="#43a047"/>
          <circle cx="358" cy="220" r="24" fill="#fdd835"/>
          <circle cx="408" cy="220" r="24" fill="#00bcd4"/>
          <circle cx="458" cy="220" r="24" fill="#81d4fa"/>
          <circle cx="358" cy="290" r="24" fill="#ff7043"/>
          <circle cx="408" cy="290" r="24" fill="#2e7d32"/>
          <circle cx="458" cy="290" r="24" fill="#8b5cf6"/>
        </svg>
      </div>

      <nav class="sidebar-nav">
        <button
          v-for="tab in tabs"
          :key="tab.id"
          class="nav-btn"
          :class="{ active: activeTab === tab.id }"
          @click="activeTab = tab.id"
          :title="tab.label"
        >
          <component :is="tab.icon" />
          <span class="nav-label">{{ tab.label }}</span>
        </button>
        <button
          class="nav-btn settings-btn"
          @click="showSettings = true"
          title="Einstellungen"
        >
          <IconSettings />
          <span class="nav-label">Settings</span>
        </button>
      </nav>
      <div class="sidebar-version">
        <template v-if="updateAvailable">
          <span class="update-link" title="Update wird installiert…">Aktualisiere…</span>
        </template>
        <template v-else>v{{ appVersion }}</template>
      </div>
    </aside>

    <!-- Main content -->
    <main class="main" :style="{ paddingBottom: bottomPadding }">
      <YouTube v-if="activeTab === 'youtube'" />
      <Spotify v-else-if="activeTab === 'spotify'" />
      <Hoerbert v-else-if="activeTab === 'hoerbert'" />
    </main>

    <!-- Audio player bar -->
    <Transition name="slide-up">
      <div v-if="playerState.active" class="player-bar">
        <div class="player-bar-content">
          <div class="player-bar-left">
            <button class="player-play-btn" @click="togglePlayback">
              <svg v-if="playerState.loading" class="player-spinner" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="var(--accent)" stroke-width="2.5">
                <circle cx="12" cy="12" r="10" opacity="0.3"/><path d="M12 2a10 10 0 0 1 10 10" stroke-linecap="round"/>
              </svg>
              <svg v-else-if="playerState.playing" width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
                <rect x="6" y="4" width="4" height="16" rx="1"/><rect x="14" y="4" width="4" height="16" rx="1"/>
              </svg>
              <svg v-else width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
                <path d="M8 5v14l11-7z"/>
              </svg>
            </button>
            <div class="player-info">
              <div class="player-track">{{ playerState.title }}</div>
              <div class="player-artist">{{ playerState.artist }}</div>
            </div>
          </div>
          <div class="player-seek-area">
            <span class="player-time">{{ formatTime(playerState.currentTime) }}</span>
            <div class="player-seek-bar" @click="seekTo($event)">
              <div class="player-seek-fill" :style="{ width: seekPercent + '%' }" />
              <div class="player-seek-thumb" :style="{ left: seekPercent + '%' }" />
            </div>
            <span class="player-time">{{ formatTime(playerState.duration) }}</span>
          </div>
          <button class="player-close-btn" @click="stopPlayer">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
              <path d="M18 6L6 18M6 6l12 12"/>
            </svg>
          </button>
        </div>
        <audio ref="audioEl" @ended="onAudioEnded" @play="playerState.playing = true" @pause="playerState.playing = false" @timeupdate="onTimeUpdate" @loadedmetadata="onMetadata" @error="onAudioError" />
      </div>
    </Transition>

    <!-- Donation toast -->
    <Transition name="slide-up">
      <div v-if="showDonationToast" class="donation-toast">
        <div class="donation-toast-icon">☕</div>
        <div class="donation-toast-body">
          <div class="donation-toast-title">Schon {{ downloadCount }} Tracks mit Ladebert geladen!</div>
          <div class="donation-toast-text">Wenn dir die App hilft, freu ich mich über einen Kaffee.</div>
        </div>
        <div class="donation-toast-actions">
          <a href="https://buymeacoffee.com/elpeyotl" target="_blank" class="donation-toast-btn primary" @click="onDonationClicked">Kaffee spendieren</a>
          <button class="donation-toast-btn" @click="dismissDonationToast">Später</button>
        </div>
      </div>
    </Transition>

    <!-- Download bar -->
    <Transition name="slide-up">
      <div v-if="downloadState.active" class="download-bar">
        <div class="download-bar-progress" :style="{ width: downloadProgress + '%' }" />
        <div class="download-bar-content">
          <div class="download-bar-left">
            <div class="download-bar-spinner" />
            <div class="download-bar-info">
              <div class="download-bar-track">{{ downloadState.currentTrack }}</div>
              <div class="download-bar-status">
                {{ downloadState.currentIndex }} / {{ downloadState.totalTracks }} Tracks
                <span v-if="downloadState.queue.length > 0"> · {{ downloadState.queue.length }} in Warteschlange</span>
                <span v-if="downloadState.speed"> · {{ downloadState.speed }}</span>
                <span v-if="downloadState.eta"> · {{ downloadState.eta }}</span>
              </div>
            </div>
          </div>
          <div class="download-bar-percent">{{ downloadProgress }}%</div>
          <button class="download-bar-cancel" @click="cancelDownloads" title="Abbrechen">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
              <path d="M18 6L6 18M6 6l12 12"/>
            </svg>
          </button>
        </div>
      </div>
    </Transition>

    <!-- Settings modal -->
    <Teleport to="body">
      <div v-if="showSettings" class="modal-backdrop" @click.self="showSettings = false">
        <div class="modal">
          <div class="modal-header">
            <h2>Einstellungen</h2>
            <button class="btn btn-ghost" @click="showSettings = false">✕</button>
          </div>
          <div class="modal-body">
            <label class="form-group">
              <span class="form-label">Standard Download-Ordner</span>
              <div class="input-row">
                <input class="input" :value="settings.downloadDir" readonly placeholder="~/Downloads/Hörbert" />
                <button class="btn btn-ghost" @click="pickDownloadDir">Wählen</button>
              </div>
            </label>
            <label class="form-group">
              <span class="form-label">Audio Format</span>
              <div class="radio-group">
                <label class="radio-btn" :class="{ active: settings.format === 'original' }">
                  <input type="radio" v-model="settings.format" value="original" hidden />
                  Original
                </label>
                <label class="radio-btn" :class="{ active: settings.format === 'mp3' }">
                  <input type="radio" v-model="settings.format" value="mp3" hidden />
                  MP3
                </label>
                <label class="radio-btn" :class="{ active: settings.format === 'aac' }">
                  <input type="radio" v-model="settings.format" value="aac" hidden />
                  AAC
                </label>
              </div>
              <span class="form-hint">
                {{ settings.format === 'original'
                  ? 'Schneller — keine Konvertierung beim Download. Ideal wenn die Dateien nur für den Hörbert genutzt werden.'
                  : 'Konvertiert beim Download zu ' + settings.format.toUpperCase() + '. Dauert etwas länger, Dateien sind aber universell abspielbar.'
                }}
              </span>
            </label>
            <label class="form-group">
              <span class="form-label">Suchsprache</span>
              <div class="radio-group">
                <label class="radio-btn" :class="{ active: settings.language === 'deutsch' }">
                  <input type="radio" v-model="settings.language" value="deutsch" hidden />
                  Deutsch
                </label>
                <label class="radio-btn" :class="{ active: settings.language === 'english' }">
                  <input type="radio" v-model="settings.language" value="english" hidden />
                  English
                </label>
                <label class="radio-btn" :class="{ active: settings.language === '' }">
                  <input type="radio" v-model="settings.language" value="" hidden />
                  Keine
                </label>
              </div>
              <span class="form-hint">Wird automatisch an YouTube-Suchen angehängt, damit Ergebnisse in der richtigen Sprache erscheinen.</span>
            </label>
          </div>
          <div class="modal-footer">
            <button class="btn btn-primary" @click="saveSettings">Speichern</button>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { ref, provide, reactive, h, computed, nextTick, onMounted } from 'vue'
import { invoke, convertFileSrc } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { check } from '@tauri-apps/plugin-updater'
import { relaunch } from '@tauri-apps/plugin-process'
import YouTube from './views/YouTube.vue'
import Spotify from './views/Spotify.vue'
import Hoerbert from './views/Hoerbert.vue'
import { version as appVersion } from '../package.json'

// Icons as render-function components (runtime-compatible)
const IconYoutube = {
  render: () => h('svg', { width: 20, height: 20, viewBox: '0 0 24 24', fill: 'currentColor' }, [
    h('path', { d: 'M19.59 6.69a4.83 4.83 0 0 1-3.77-2.73 12.73 12.73 0 0 0-8.44 0A4.83 4.83 0 0 1 3.61 6.69 28 28 0 0 0 3 12a28 28 0 0 0 .61 5.31 4.83 4.83 0 0 1 3.77 2.73 12.73 12.73 0 0 0 8.44 0 4.83 4.83 0 0 1 3.77-2.73A28 28 0 0 0 21 12a28 28 0 0 0-.41-5.31zM10 15V9l5 3z' })
  ])
}
const IconSpotify = {
  render: () => h('svg', { width: 20, height: 20, viewBox: '0 0 24 24', fill: 'currentColor' }, [
    h('path', { d: 'M12 2C6.477 2 2 6.477 2 12s4.477 10 10 10 10-4.477 10-10S17.523 2 12 2zm4.586 14.424a.622.622 0 0 1-.857.207c-2.348-1.435-5.304-1.76-8.785-.964a.623.623 0 0 1-.277-1.215c3.809-.87 7.077-.496 9.712 1.115.294.181.387.564.207.857zm1.223-2.722a.78.78 0 0 1-1.072.257c-2.687-1.652-6.785-2.131-9.965-1.166a.78.78 0 0 1-.973-.519.78.78 0 0 1 .52-.972c3.632-1.102 8.147-.568 11.234 1.328a.78.78 0 0 1 .256 1.072zm.105-2.835C14.692 8.95 9.375 8.775 6.297 9.71a.937.937 0 1 1-.543-1.794c3.532-1.072 9.404-.865 13.115 1.338a.937.937 0 0 1-.954 1.613z' })
  ])
}
const IconHoerbert = {
  render: () => h('svg', { width: 20, height: 20, viewBox: '0 0 24 24', fill: 'none', stroke: 'currentColor', 'stroke-width': 2, 'stroke-linecap': 'round' }, [
    h('rect', { x: 2, y: 6, width: 20, height: 14, rx: 2 }),
    h('path', { d: 'M8 6V4M16 6V4M7 14h.01M12 14h.01M17 14h.01M7 10h.01M12 10h.01M17 10h.01' })
  ])
}
const IconSettings = {
  render: () => h('svg', { width: 18, height: 18, viewBox: '0 0 24 24', fill: 'none', stroke: 'currentColor', 'stroke-width': 2, 'stroke-linecap': 'round' }, [
    h('circle', { cx: 12, cy: 12, r: 3 }),
    h('path', { d: 'M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z' })
  ])
}

const tabs = [
  { id: 'youtube', label: 'YouTube', icon: IconYoutube },
  { id: 'spotify', label: 'Spotify', icon: IconSpotify },
  { id: 'hoerbert', label: 'Hörbert', icon: IconHoerbert },
]

const activeTab = ref('youtube')
const showSettings = ref(false)

const settings = reactive({
  downloadDir: localStorage.getItem('downloadDir') || '',
  format: (localStorage.getItem('format') as 'mp3' | 'aac' | 'original') || 'original',
  language: (localStorage.getItem('language') ?? 'deutsch') as string,
})

async function pickDownloadDir() {
  const dir = await invoke<string | null>('select_folder')
  if (dir) settings.downloadDir = dir
}

function saveSettings() {
  localStorage.setItem('downloadDir', settings.downloadDir)
  localStorage.setItem('format', settings.format)
  localStorage.setItem('language', settings.language)
  showSettings.value = false
}

// Download queue system
interface QueueItem {
  id: string
  title: string
  url: string
  format: string
  outputDir: string
  eventId: string
}

const downloadState = reactive({
  active: false,
  cancelled: false,
  currentTrack: '',
  currentIndex: 0,
  totalTracks: 0,
  trackPercent: 0,
  speed: '',
  eta: '',
  log: [] as string[],
  currentEventId: '',
  queue: [] as QueueItem[],
  processing: false,
})

let queueProcessing = false

function addToQueue(items: QueueItem | QueueItem[]) {
  const newItems = Array.isArray(items) ? items : [items]
  downloadState.queue.push(...newItems)
  // +1 for the currently downloading track if queue is active
  const inProgress = queueProcessing ? 1 : 0
  downloadState.totalTracks = downloadState.currentIndex + downloadState.queue.length + inProgress
  if (!queueProcessing) {
    processQueue()
  }
}

async function processQueue() {
  if (queueProcessing) return
  queueProcessing = true
  downloadState.active = true
  downloadState.cancelled = false
  downloadState.currentIndex = 0
  downloadState.totalTracks = downloadState.queue.length

  while (downloadState.queue.length > 0) {
    if (downloadState.cancelled) break

    const item = downloadState.queue.shift()!
    downloadState.currentTrack = item.title
    downloadState.trackPercent = 0
    downloadState.speed = ''
    downloadState.eta = ''
    downloadState.currentEventId = item.eventId
    // totalTracks may have grown if user added more items
    downloadState.totalTracks = downloadState.currentIndex + downloadState.queue.length + 1

    const unlisten1 = await listen<any>(`download-progress-${item.eventId}`, (event) => {
      downloadState.trackPercent = event.payload.percent || 0
      downloadState.speed = event.payload.speed || ''
      downloadState.eta = event.payload.eta || ''
    })
    const unlisten2 = await listen(`download-done-${item.eventId}`, () => {
      unlisten1()
      unlisten2()
    })

    let succeeded = false
    try {
      await invoke('download_audio', {
        url: item.url,
        format: item.format,
        outputDir: item.outputDir,
        eventId: item.eventId,
      })
      succeeded = true
    } catch (e: any) {
      if (downloadState.cancelled) break
      console.error(`Download failed: ${item.title}`, e)
      downloadState.log.push(`✗ ${item.title}`)
    }

    unlisten1()
    unlisten2()
    downloadState.currentIndex++
    if (succeeded) recordSuccessfulDownload()
  }

  downloadState.currentTrack = downloadState.cancelled ? 'Abgebrochen' : 'Fertig!'
  downloadState.totalTracks = downloadState.currentIndex

  setTimeout(() => {
    downloadState.active = false
    downloadState.cancelled = false
    downloadState.currentIndex = 0
    downloadState.totalTracks = 0
    downloadState.log = []
    queueProcessing = false
  }, 3000)
}

async function cancelDownloads() {
  downloadState.cancelled = true
  downloadState.queue.length = 0
  try {
    await invoke('cancel_all_downloads')
  } catch (e) {
    console.error('Cancel failed:', e)
  }
}

const downloadProgress = computed(() => {
  if (downloadState.totalTracks <= 0) return 0
  const completedPart = (downloadState.currentIndex / downloadState.totalTracks) * 100
  const currentPart = (downloadState.trackPercent / downloadState.totalTracks)
  return Math.min(Math.round(completedPart + currentPart), 100)
})

// Global audio player state
const audioEl = ref<HTMLAudioElement | null>(null)
const playerState = reactive({
  active: false,
  playing: false,
  loading: false,
  title: '',
  artist: '',
  streamUrl: '',
  currentTime: 0,
  duration: 0,
})

const seekPercent = computed(() =>
  playerState.duration > 0 ? (playerState.currentTime / playerState.duration) * 100 : 0
)

function formatTime(sec: number) {
  const m = Math.floor(sec / 60)
  const s = Math.floor(sec % 60)
  return `${m}:${s.toString().padStart(2, '0')}`
}

function onTimeUpdate() {
  if (audioEl.value) playerState.currentTime = audioEl.value.currentTime
}

function onMetadata() {
  if (audioEl.value) playerState.duration = audioEl.value.duration
}

function seekTo(event: MouseEvent) {
  const bar = event.currentTarget as HTMLElement
  const rect = bar.getBoundingClientRect()
  const pct = (event.clientX - rect.left) / rect.width
  if (audioEl.value && playerState.duration) {
    audioEl.value.currentTime = pct * playerState.duration
  }
}

async function playTrack(title: string, artist: string, searchQuery: string) {
  playerState.active = true
  playerState.loading = true
  playerState.playing = false
  playerState.title = title
  playerState.artist = artist

  try {
    const filePath = await invoke<string>('get_stream_url', { query: searchQuery })
    const url = convertFileSrc(filePath)
    playerState.streamUrl = url
    playerState.loading = false

    playerState.currentTime = 0
    playerState.duration = 0

    await nextTick()
    if (audioEl.value) {
      audioEl.value.src = url
      try {
        await audioEl.value.play()
      } catch (playErr) {
        console.error('Play Fehler:', playErr)
      }
    }
  } catch (e: any) {
    console.error('Stream-URL Fehler:', e)
    playerState.loading = false
    playerState.active = false
  }
}

async function playFile(title: string, artist: string, filePath: string) {
  playerState.active = true
  playerState.loading = false
  playerState.playing = false
  playerState.title = title
  playerState.artist = artist
  playerState.currentTime = 0
  playerState.duration = 0

  const url = convertFileSrc(filePath)
  playerState.streamUrl = url

  await nextTick()
  if (audioEl.value) {
    audioEl.value.src = url
    try {
      await audioEl.value.play()
    } catch (playErr) {
      console.error('Play Fehler:', playErr)
    }
  }
}

function togglePlayback() {
  if (!audioEl.value || playerState.loading) return
  if (playerState.playing) {
    audioEl.value.pause()
  } else {
    audioEl.value.play()
  }
}

function stopPlayer() {
  if (audioEl.value) {
    audioEl.value.pause()
    audioEl.value.src = ''
  }
  playerState.active = false
  playerState.playing = false
  playerState.streamUrl = ''
}

function onAudioEnded() {
  playerState.playing = false
}

function onAudioError() {
  const el = audioEl.value
  const code = el?.error?.code
  const msg = el?.error?.message || 'Unbekannter Fehler'
  console.error('Audio Fehler:', code, msg, 'URL:', el?.src?.substring(0, 100))
  playerState.playing = false
  playerState.loading = false
}

// Bottom padding to prevent fixed bars from covering content
const bottomPadding = computed(() => {
  let px = 0
  if (playerState.active) px += 52
  if (downloadState.active) px += 56
  return px ? `${px}px` : '0'
})

// Auto-update: silent download + install, relaunch when ready
const updateAvailable = ref(false)
const updateVersion = ref('')

async function checkForUpdate() {
  try {
    const update = await check()
    if (update) {
      updateAvailable.value = true
      updateVersion.value = update.version
      await update.downloadAndInstall()
      await relaunch()
    }
  } catch {
    // Silently ignore – no network, no problem
  }
}

// Donation nudge
const DONATION_THRESHOLD = 30
const DONATION_DISMISS_DAYS = 60
const downloadCount = ref(parseInt(localStorage.getItem('ladebert:download_count') || '0', 10))
const showDonationToast = ref(false)

function shouldShowDonationToast(): boolean {
  if (localStorage.getItem('ladebert:donation_supported') === '1') return false
  if (downloadCount.value < DONATION_THRESHOLD) return false
  const dismissedAt = localStorage.getItem('ladebert:donation_dismissed_at')
  if (dismissedAt) {
    const daysSince = (Date.now() - new Date(dismissedAt).getTime()) / (1000 * 60 * 60 * 24)
    if (daysSince < DONATION_DISMISS_DAYS) return false
  }
  return true
}

function recordSuccessfulDownload() {
  downloadCount.value++
  localStorage.setItem('ladebert:download_count', String(downloadCount.value))
  if (!showDonationToast.value && shouldShowDonationToast()) {
    showDonationToast.value = true
  }
}

function dismissDonationToast() {
  localStorage.setItem('ladebert:donation_dismissed_at', new Date().toISOString())
  showDonationToast.value = false
}

function onDonationClicked() {
  localStorage.setItem('ladebert:donation_supported', '1')
  showDonationToast.value = false
}

onMounted(() => {
  checkForUpdate()
})

// Provide settings and download state to child components
provide('settings', settings)
provide('downloadState', downloadState)
provide('addToQueue', addToQueue)
provide('playTrack', playTrack)
provide('playFile', playFile)
</script>

<style scoped>
.app {
  display: flex;
  height: 100vh;
  overflow: hidden;
}

/* Sidebar */
.sidebar {
  width: var(--sidebar-w);
  background: var(--bg-card);
  border-right: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 16px 0;
  flex-shrink: 0;
}

.sidebar-logo {
  margin-bottom: 28px;
}

.sidebar-nav {
  display: flex;
  flex-direction: column;
  gap: 4px;
  flex: 1;
}

.sidebar-version {
  font-size: 10px;
  color: var(--text-faint);
  text-align: center;
  padding: 8px 0;
  font-family: 'DM Mono', monospace;
}

.update-link {
  color: var(--accent);
  text-decoration: none;
  font-weight: 600;
  animation: pulse-update 2s ease-in-out infinite;
}
.update-link:hover {
  text-decoration: underline;
}

@keyframes pulse-update {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}

.settings-btn {
  margin-top: 12px;
  border-top: 1px solid var(--border);
  padding-top: 14px !important;
  color: var(--text) !important;
}

.nav-btn {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  width: 52px;
  padding: 10px 6px;
  background: transparent;
  border: none;
  border-radius: var(--radius);
  color: var(--text-muted);
  cursor: pointer;
  transition: all 0.15s;
}

.nav-btn:hover {
  background: var(--bg-hover);
  color: var(--text);
}

.nav-btn.active {
  background: var(--accent-dim);
  color: var(--accent);
}

.nav-label {
  font-size: 9px;
  font-weight: 600;
  letter-spacing: 0.04em;
  text-transform: uppercase;
}

/* Main */
.main {
  flex: 1;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

/* Modal */
.modal-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.7);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
}

.modal {
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  width: 480px;
  max-width: 90vw;
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 20px 24px 0;
}

.modal-header h2 {
  font-size: 16px;
  font-weight: 600;
}

.modal-body {
  padding: 20px 24px;
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.modal-footer {
  padding: 0 24px 20px;
  display: flex;
  justify-content: flex-end;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.form-hint {
  font-size: 12px;
  color: var(--text-muted);
  line-height: 1.4;
}

.form-label {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-muted);
  letter-spacing: 0.04em;
  text-transform: uppercase;
}

.input-row {
  display: flex;
  gap: 8px;
}

.radio-group {
  display: flex;
  gap: 8px;
}

.radio-btn {
  padding: 6px 16px;
  border-radius: var(--radius);
  border: 1px solid var(--border);
  color: var(--text-muted);
  cursor: pointer;
  font-size: 13px;
  font-weight: 500;
  transition: all 0.15s;
}

.radio-btn.active {
  background: var(--accent-dim);
  border-color: var(--accent);
  color: var(--accent);
}

/* Player bar */
.player-bar {
  position: fixed;
  bottom: 0;
  left: var(--sidebar-w);
  right: 0;
  height: 52px;
  background: var(--bg-card);
  border-top: 1px solid var(--border);
  z-index: 49;
  transition: bottom 0.3s ease;
}

.player-bar-content {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 100%;
  padding: 0 16px;
}

.player-bar-left {
  display: flex;
  align-items: center;
  gap: 12px;
  min-width: 0;
}

.player-play-btn {
  width: 34px;
  height: 34px;
  border-radius: 50%;
  border: none;
  background: var(--accent-dim);
  color: var(--accent);
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  flex-shrink: 0;
  transition: background 0.15s;
}

.player-play-btn:hover {
  background: var(--accent);
  color: #000;
}

.player-spinner {
  animation: spin 0.8s linear infinite;
}

.player-info {
  min-width: 0;
}

.player-track {
  font-size: 13px;
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 400px;
}

.player-artist {
  font-size: 11px;
  color: var(--text-muted);
  margin-top: 1px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 300px;
}

.player-seek-area {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 1;
  max-width: 360px;
  margin: 0 16px;
}

.player-time {
  font-family: 'DM Mono', monospace;
  font-size: 11px;
  color: var(--text-muted);
  min-width: 32px;
  text-align: center;
}

.player-seek-bar {
  flex: 1;
  height: 4px;
  background: var(--border);
  border-radius: 2px;
  cursor: pointer;
  position: relative;
}

.player-seek-bar:hover {
  height: 6px;
}

.player-seek-fill {
  height: 100%;
  background: var(--accent);
  border-radius: 2px;
  pointer-events: none;
}

.player-seek-thumb {
  position: absolute;
  top: 50%;
  width: 10px;
  height: 10px;
  background: var(--accent);
  border-radius: 50%;
  transform: translate(-50%, -50%);
  opacity: 0;
  transition: opacity 0.15s;
  pointer-events: none;
}

.player-seek-bar:hover .player-seek-thumb {
  opacity: 1;
}

.player-close-btn {
  background: none;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  padding: 6px;
  border-radius: 4px;
  transition: all 0.15s;
}

.player-close-btn:hover {
  color: var(--text);
  background: var(--bg-hover);
}

/* Donation toast */
.donation-toast {
  position: fixed;
  bottom: 72px;
  right: 20px;
  max-width: 360px;
  display: flex;
  gap: 12px;
  padding: 14px 16px;
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: 12px;
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.35);
  z-index: 60;
  align-items: flex-start;
}

.donation-toast-icon {
  font-size: 24px;
  line-height: 1;
  padding-top: 2px;
}

.donation-toast-body {
  flex: 1;
  min-width: 0;
}

.donation-toast-title {
  font-weight: 600;
  font-size: 14px;
  margin-bottom: 4px;
}

.donation-toast-text {
  font-size: 13px;
  color: var(--text-dim);
  line-height: 1.4;
}

.donation-toast-actions {
  display: flex;
  flex-direction: column;
  gap: 6px;
  flex-shrink: 0;
}

.donation-toast-btn {
  padding: 6px 12px;
  font-size: 12px;
  border-radius: 6px;
  border: 1px solid var(--border);
  background: transparent;
  color: var(--text);
  cursor: pointer;
  text-decoration: none;
  text-align: center;
  white-space: nowrap;
  transition: background 0.15s;
}

.donation-toast-btn:hover {
  background: var(--bg-hover, rgba(255, 255, 255, 0.05));
}

.donation-toast-btn.primary {
  background: var(--accent, #c4935a);
  border-color: var(--accent, #c4935a);
  color: #1a1410;
  font-weight: 600;
}

.donation-toast-btn.primary:hover {
  filter: brightness(1.1);
}

/* Download bar */
.download-bar {
  position: fixed;
  bottom: 0;
  left: var(--sidebar-w);
  right: 0;
  height: 56px;
  background: var(--bg-card);
  border-top: 1px solid var(--border);
  z-index: 50;
  overflow: hidden;
}

.download-bar-progress {
  position: absolute;
  top: 0;
  left: 0;
  height: 100%;
  background: var(--accent-dim);
  transition: width 0.4s ease;
}

.download-bar-content {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 100%;
  padding: 0 20px;
}

.download-bar-left {
  display: flex;
  align-items: center;
  gap: 14px;
  min-width: 0;
}

.download-bar-spinner {
  width: 20px;
  height: 20px;
  border: 2.5px solid var(--border);
  border-top-color: var(--accent);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
  flex-shrink: 0;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.download-bar-info {
  min-width: 0;
}

.download-bar-track {
  font-size: 13px;
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 400px;
}

.download-bar-status {
  font-size: 11px;
  color: var(--text-muted);
  margin-top: 2px;
}

.download-bar-percent {
  font-size: 14px;
  font-weight: 600;
  color: var(--accent);
  flex-shrink: 0;
}

.download-bar-cancel {
  background: none;
  border: 1px solid rgba(255, 255, 255, 0.2);
  border-radius: 6px;
  color: var(--text-muted);
  cursor: pointer;
  padding: 4px 6px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  transition: all 0.15s;
  margin-left: 8px;
}

.download-bar-cancel:hover {
  color: var(--red, #ef4444);
  border-color: var(--red, #ef4444);
  background: rgba(239, 68, 68, 0.1);
}

.slide-up-enter-active,
.slide-up-leave-active {
  transition: transform 0.3s ease, opacity 0.3s ease;
}

.slide-up-enter-from,
.slide-up-leave-to {
  transform: translateY(100%);
  opacity: 0;
}
</style>
