<template>
  <div class="youtube">
    <!-- Header -->
    <div class="header">
      <div class="header-left">
        <h1>YouTube</h1>
      </div>
      <div class="mode-toggle">
        <button
          v-for="m in modes"
          :key="m.id"
          class="mode-btn"
          :class="{ active: mode === m.id }"
          @click="mode = m.id as any"
        >{{ m.label }}</button>
      </div>
    </div>

    <!-- Search bar -->
    <div class="search-bar">
      <div class="search-input-wrap">
        <svg class="search-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="11" cy="11" r="8"/><path d="m21 21-4.35-4.35"/>
        </svg>
        <input
          class="input search-input"
          v-model="query"
          :placeholder="mode === 'search' ? searchPlaceholder : 'YouTube URL oder Playlist URL einfügen'"
          @keyup.enter="mode === 'search' ? doSearch() : loadUrl()"
        />
        <button
          v-if="query"
          class="clear-btn"
          @click="query = ''; results = []; playlistTracks = []"
        >✕</button>
      </div>
      <button
        class="btn btn-primary"
        @click="mode === 'search' ? doSearch() : loadUrl()"
        :disabled="loading || !canSearch()"
      >
        <span v-if="loading" class="spinner" />
        <span>{{ mode === 'search' ? 'Suchen' : 'Laden' }}</span>
      </button>
    </div>

    <!-- Filters (only in search mode) -->
    <div v-if="mode === 'search'" class="filters-bar">
      <div class="filter-row">
        <button class="chip" :class="{ active: searchType === 'videos' }" @click="searchType = 'videos'">Videos</button>
        <button class="chip" :class="{ active: searchType === 'playlists' }" @click="searchType = 'playlists'">Playlisten</button>
        <span class="filter-divider" />
        <button
          v-for="cat in categories"
          :key="cat.id"
          class="chip"
          :class="{ active: activeCategory === cat.id }"
          @click="toggleCategory(cat.id)"
        >{{ cat.label }}</button>
      </div>
      <div v-if="searchType === 'videos'" class="filter-row">
        <button
          v-for="dur in durations"
          :key="dur.id"
          class="chip chip-sm"
          :class="{ active: activeDuration === dur.id }"
          @click="activeDuration = dur.id"
        >{{ dur.label }}</button>
      </div>
      <div v-if="activeCategory && !query.trim()" class="filter-hint">
        Klicke auf «Suchen» um allgemein nach {{ categories.find(c => c.id === activeCategory)?.label }} zu stöbern.
      </div>
    </div>

    <!-- Error -->
    <div v-if="error" class="error-banner">
      ⚠️ {{ error }}
    </div>

    <!-- Content area -->
    <div class="content">
      <!-- Left: Results -->
      <div class="results-panel">
        <!-- Search results -->
        <template v-if="mode === 'search' && searchType === 'playlists'">
          <div v-if="loading" class="empty-state">
            <div class="search-spinner" />
            <p>Suche nach Playlisten...</p>
          </div>
          <div v-else-if="!playlistResults.length" class="empty-state">
            <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <path d="M4 6h16M4 10h16M4 14h8"/>
            </svg>
            <p>Suche nach Alben, Playlisten oder Sammlungen</p>
          </div>
          <div v-else>
            <div class="results-header">
              <span class="results-count">{{ playlistResults.length }} Playlisten</span>
            </div>
            <div class="results-grid">
              <div v-for="pl in playlistResults" :key="pl.id" class="track-card playlist-card" @click="loadPlaylistFromSearch(pl)">
                <div class="track-info" style="flex:1">
                  <div class="track-title">{{ pl.title }}</div>
                  <div class="track-meta">
                    <span v-if="pl.channel">{{ pl.channel }}</span>
                    <span v-if="pl.channel && pl.count" class="dot">·</span>
                    <span v-if="pl.count">{{ pl.count }} Tracks</span>
                  </div>
                </div>
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="flex-shrink:0;opacity:0.4">
                  <path d="M9 18l6-6-6-6"/>
                </svg>
              </div>
            </div>
          </div>
        </template>

        <template v-else-if="mode === 'search'">
          <div v-if="loading && !results.length" class="empty-state">
            <div class="search-spinner" />
            <p>Suche läuft...</p>
          </div>
          <div v-else-if="!results.length && !loading" class="empty-state">
            <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <path d="M9 18V5l12-2v13M9 9l12-2"/>
              <circle cx="6" cy="18" r="3"/><circle cx="18" cy="16" r="3"/>
            </svg>
            <p>Suche nach Songs, Künstlern oder Alben</p>
            <div class="quick-searches">
              <button
                v-for="qs in quickSearches"
                :key="qs"
                class="quick-search-btn"
                @click="query = qs; doSearch()"
              >{{ qs }}</button>
            </div>
          </div>
          <div v-else>
            <div class="results-header">
              <span class="results-count">{{ filteredResults.length }} Ergebnisse</span>
              <button
                class="btn btn-primary btn-sm"
                @click="downloadAllResults"
              >
                <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                  <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4M7 10l5 5 5-5M12 15V3"/>
                </svg>
                Alle herunterladen
              </button>
            </div>
            <div class="results-grid">
            <div
              v-for="track in filteredResults"
              :key="track.id"
              class="track-card"
            >
              <img :src="track.thumbnail" :alt="track.title" class="track-thumb" />
              <div class="track-info">
                <div class="track-title">{{ track.title }}</div>
                <div class="track-meta">
                  <span>{{ track.channel }}</span>
                  <span class="dot">·</span>
                  <span>{{ track.duration }}</span>
                </div>
              </div>
              <button
                class="btn btn-ghost track-play-btn"
                @click="playTrack(track.title, track.channel, `ytsearch1:${track.channel} - ${track.title}`)"
                title="Vorhören"
              >
                <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M8 5v14l11-7z"/>
                </svg>
              </button>
              <button
                class="btn btn-primary track-dl-btn"
                @click="downloadTrack(track)"
                :disabled="isDownloading(track.id)"
              >
                <span v-if="isDownloading(track.id)" class="spinner" />
                <svg v-else width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                  <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4M7 10l5 5 5-5M12 15V3"/>
                </svg>
                {{ isDownloading(track.id) ? 'Lädt...' : (settings.format === 'original' ? 'DL' : settings.format.toUpperCase()) }}
              </button>
            </div>
            </div>
            <button
              class="btn btn-ghost load-more-btn"
              @click="loadMore"
              :disabled="loadingMore"
            >
              <span v-if="loadingMore" class="load-more-spinner" />
              {{ loadingMore ? 'Laden...' : 'Mehr Ergebnisse laden' }}
            </button>
          </div>
        </template>

        <!-- URL / Playlist mode -->
        <template v-else>
          <div v-if="!playlistTracks.length && !loading" class="empty-state">
            <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <path d="M4 6h16M4 10h16M4 14h8"/>
            </svg>
            <p>YouTube Video- oder Playlist-URL einfügen</p>
          </div>
          <template v-else>
            <div class="playlist-header">
              <div>
                <div class="playlist-title">{{ playlistTitle || 'Playlist' }}</div>
                <div class="playlist-count">{{ playlistTracks.length }} Tracks</div>
              </div>
              <button
                class="btn btn-primary"
                @click="downloadAll"
                :disabled="downloadingAll"
              >
                <span v-if="downloadingAll" class="spinner" />
                <svg v-else width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                  <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4M7 10l5 5 5-5M12 15V3"/>
                </svg>
                Alle laden
              </button>
            </div>
            <div class="results-grid">
              <div
                v-for="(track, i) in playlistTracks"
                :key="track.id"
                class="track-card"
              >
                <div class="track-num">{{ i + 1 }}</div>
                <div class="track-info">
                  <div class="track-title">{{ track.title }}</div>
                  <div class="track-meta">
                    <span>{{ track.channel }}</span>
                    <span v-if="track.duration" class="dot">·</span>
                    <span v-if="track.duration">{{ track.duration }}</span>
                  </div>
                </div>
                <button
                  class="btn btn-ghost track-play-btn"
                  @click="playTrack(track.title, track.channel, `ytsearch1:${track.channel} - ${track.title}`)"
                  title="Vorhören"
                >
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                    <path d="M8 5v14l11-7z"/>
                  </svg>
                </button>
                <button
                  class="btn btn-primary track-dl-btn"
                  @click="downloadTrack(track)"
                  :disabled="isDownloading(track.id)"
                >
                  <span v-if="isDownloading(track.id)" class="spinner" />
                  <svg v-else width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                    <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4M7 10l5 5 5-5M12 15V3"/>
                  </svg>
                </button>
              </div>
            </div>
          </template>
        </template>
      </div>

    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, inject, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const settings = inject('settings') as any
const downloadState = inject('downloadState') as any
const addToQueue = inject('addToQueue') as (items: any) => void
const playTrack = inject('playTrack') as (title: string, artist: string, query: string) => void

type Mode = 'search' | 'url'
type SearchType = 'videos' | 'playlists'
const mode = ref<Mode>('search')
const searchType = ref<SearchType>('videos')
const modes = [
  { id: 'search', label: 'Suche' },
  { id: 'url', label: 'URL / Playlist' },
]

const query = ref('')
const loading = ref(false)
const error = ref('')

// Playlist search results
const playlistResults = ref<any[]>([])

// ─── Categories ────────────────────────────────────────
const categories = [
  { id: 'hoerspiel', label: 'Hörspiel', suffix: 'hörspiel komplett', placeholder: 'z.B. Asterix, Bibi Blocksberg, TKKG...' },
  { id: 'kinderlieder', label: 'Kinderlieder', suffix: 'kinderlieder', placeholder: 'z.B. Schlaflieder, Bewegungslieder...' },
  { id: 'gutenacht', label: 'Gute-Nacht', suffix: 'gute nacht geschichten für kinder', placeholder: 'z.B. Sandmännchen, Einschlafgeschichten...' },
  { id: 'maerchen', label: 'Märchen', suffix: 'märchen hörspiel', placeholder: 'z.B. Rotkäppchen, Rapunzel, Froschkönig...' },
  { id: 'wissen', label: 'Wissen', suffix: 'kinder wissen erklärt', placeholder: 'z.B. Dinosaurier, Weltraum, Körper...' },
]
const activeCategory = ref('')

function toggleCategory(id: string) {
  activeCategory.value = activeCategory.value === id ? '' : id
}

const searchPlaceholder = computed(() => {
  const cat = categories.find(c => c.id === activeCategory.value)
  if (cat) return cat.placeholder
  return 'Song, Artist oder Album suchen...'
})

// ─── Duration filter ───────────────────────────────────
type DurationFilter = 'all' | 'short' | 'medium' | 'long'
const durations: { id: DurationFilter; label: string }[] = [
  { id: 'all', label: 'Alle' },
  { id: 'short', label: '< 5 Min' },
  { id: 'medium', label: '5–30 Min' },
  { id: 'long', label: '> 30 Min' },
]
const activeDuration = ref<DurationFilter>('all')

function parseDurationToSeconds(dur: string): number {
  const parts = dur.split(':').map(Number)
  if (parts.length === 3) return parts[0] * 3600 + parts[1] * 60 + parts[2]
  if (parts.length === 2) return parts[0] * 60 + parts[1]
  return 0
}

const filteredResults = computed(() => {
  if (activeDuration.value === 'all') return results.value
  return results.value.filter(t => {
    const sec = parseDurationToSeconds(t.duration)
    if (activeDuration.value === 'short') return sec < 300
    if (activeDuration.value === 'medium') return sec >= 300 && sec <= 1800
    if (activeDuration.value === 'long') return sec > 1800
    return true
  })
})

// ─── Quick searches ────────────────────────────────────
const quickSearches = [
  'Bibi Blocksberg', 'Benjamin Blümchen', 'Die drei ???',
  'Conni', 'Asterix', 'Pettersson und Findus',
  'Schlaflieder', 'Kinderlieder zum Mitsingen',
]

// ─── Download dir based on category ────────────────────
function getOutputDir(): string {
  const base = settings.downloadDir || '~/Downloads/Hörbert'
  const cat = categories.find(c => c.id === activeCategory.value)
  if (cat) return `${base}/${cat.label}`
  return base
}

// ─── Search query builder ──────────────────────────────
function buildSearchQuery(): string {
  const parts: string[] = []
  if (query.value.trim()) parts.push(query.value.trim())
  const cat = categories.find(c => c.id === activeCategory.value)
  if (cat) {
    parts.push(cat.suffix)
    // Only add language for category searches (not for free text / playlist searches)
    if (settings.language) parts.push(settings.language)
  }
  return parts.join(' ')
}

interface Track {
  id: string
  title: string
  channel: string
  duration: string
  thumbnail: string
  url: string
}

const results = ref<Track[]>([])
const playlistTracks = ref<Track[]>([])
const playlistTitle = ref('')
const downloadingAll = ref(false)
const loadingMore = ref(false)
const searchResultCount = ref(10)

function canSearch(): boolean {
  return !!(query.value.trim() || activeCategory.value)
}

async function doSearch() {
  if (!canSearch()) return
  loading.value = true
  error.value = ''

  if (searchType.value === 'playlists') {
    playlistResults.value = []
    try {
      playlistResults.value = await invoke<any[]>('search_youtube_playlists', { query: buildSearchQuery(), limit: 10 })
    } catch (e: any) {
      error.value = e
    } finally {
      loading.value = false
    }
    return
  }

  results.value = []
  searchResultCount.value = 10
  try {
    results.value = await invoke<Track[]>('search_youtube', { query: buildSearchQuery(), limit: 10 })
  } catch (e: any) {
    error.value = e
  } finally {
    loading.value = false
  }
}

async function loadMore() {
  if (!canSearch()) return
  loadingMore.value = true
  const newCount = searchResultCount.value + 10
  try {
    const allResults = await invoke<Track[]>('search_youtube', { query: buildSearchQuery(), limit: newCount })
    results.value = allResults
    searchResultCount.value = newCount
  } catch (e: any) {
    error.value = e
  } finally {
    loadingMore.value = false
  }
}

function downloadAllResults() {
  const tracks = filteredResults.value
  if (!tracks.length) return

  const outputDir = getOutputDir()
  const fmt = settings.format

  const items = tracks.map((track, i) => ({
    id: track.id,
    title: track.title,
    url: track.url,
    format: fmt,
    outputDir,
    eventId: `yt-all-${Date.now()}-${i}`,
  }))

  addToQueue(items)
}

function loadPlaylistFromSearch(pl: any) {
  query.value = pl.url
  mode.value = 'url'
  loadUrl()
}

async function loadUrl() {
  if (!query.value.trim()) return
  loading.value = true
  error.value = ''
  playlistTracks.value = []
  try {
    const info = await invoke<{ tracks: any[], title?: string }>('get_playlist_info', { url: query.value })
    playlistTitle.value = info.title || ''
    playlistTracks.value = info.tracks.map((t: any) => ({
      id: t.id,
      title: t.title || 'Unbekannt',
      channel: t.uploader || t.channel || '',
      duration: t.duration ? formatDuration(t.duration) : '',
      thumbnail: t.thumbnail || '',
      url: t.url || `https://www.youtube.com/watch?v=${t.id}`,
    }))
  } catch (e: any) {
    error.value = e
  } finally {
    loading.value = false
  }
}

function formatDuration(seconds: number) {
  const m = Math.floor(seconds / 60)
  const s = Math.floor(seconds % 60)
  return `${m}:${s.toString().padStart(2, '0')}`
}

function isDownloading(id: string) {
  return downloadState.queue.some((q: any) => q.id === id) ||
    (downloadState.active && downloadState.currentEventId.includes(id))
}

function downloadTrack(track: Track) {
  let outputDir = getOutputDir()
  if (mode.value === 'url' && playlistTitle.value) {
    const base = settings.downloadDir || '~/Downloads/Hörbert'
    outputDir = `${base}/Playlists/${playlistTitle.value}`
  }
  const eventId = `yt-${track.id}-${Date.now()}`

  addToQueue({
    id: track.id,
    title: track.title,
    url: track.url,
    format: settings.format,
    outputDir,
    eventId,
  })
}

function downloadAll() {
  if (!playlistTracks.value.length) return

  const base = settings.downloadDir || '~/Downloads/Hörbert'
  const folderName = playlistTitle.value || 'Playlist'
  const outputDir = `${base}/Playlists/${folderName}`
  const fmt = settings.format

  const items = playlistTracks.value.map((track, i) => ({
    id: track.id,
    title: track.title,
    url: track.url,
    format: fmt,
    outputDir,
    eventId: `yt-pl-${Date.now()}-${i}`,
  }))

  addToQueue(items)
}

</script>

<style scoped>
.youtube {
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

.header-left h1 {
  font-size: 18px;
  font-weight: 600;
}

.mode-toggle {
  display: flex;
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 3px;
  gap: 2px;
}

.mode-btn {
  padding: 5px 14px;
  border-radius: 5px;
  border: none;
  background: transparent;
  color: var(--text-muted);
  font-family: inherit;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s;
}

.mode-btn.active {
  background: var(--accent);
  color: #000;
}

.search-bar {
  display: flex;
  gap: 10px;
  padding: 16px 24px;
  flex-shrink: 0;
}

.filters-bar {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 0 24px 14px;
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
}

.filter-row {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.chip {
  padding: 5px 14px;
  border-radius: 20px;
  border: 1px solid var(--border);
  background: transparent;
  color: var(--text-muted);
  font-family: inherit;
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s;
}

.chip:hover {
  border-color: var(--accent);
  color: var(--text);
}

.chip.active {
  background: var(--accent-dim);
  border-color: var(--accent);
  color: var(--accent);
}

.chip-sm {
  padding: 3px 10px;
  font-size: 11px;
}

.filter-hint {
  font-size: 12px;
  color: var(--text-muted);
  font-style: italic;
}

.search-input-wrap {
  position: relative;
  flex: 1;
  display: flex;
  align-items: center;
}

.search-icon {
  position: absolute;
  left: 12px;
  color: var(--text-muted);
  pointer-events: none;
}

.search-input {
  padding-left: 38px;
  padding-right: 32px;
}

.clear-btn {
  position: absolute;
  right: 10px;
  background: none;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  font-size: 12px;
  padding: 4px;
}

.error-banner {
  background: var(--red-dim);
  border-bottom: 1px solid rgba(239, 68, 68, 0.3);
  color: var(--red);
  padding: 10px 24px;
  font-size: 13px;
  flex-shrink: 0;
}

.search-spinner {
  width: 28px;
  height: 28px;
  border: 3px solid var(--border);
  border-top-color: var(--accent);
  border-radius: 50%;
  animation: spin 0.6s linear infinite;
}

.results-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
  padding-bottom: 12px;
  border-bottom: 1px solid var(--border);
}

.results-count {
  font-size: 13px;
  color: var(--text-muted);
}

.btn-sm {
  padding: 5px 12px;
  font-size: 12px;
  gap: 5px;
}

.load-more-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  width: 100%;
  margin-top: 12px;
  padding: 10px;
  font-size: 13px;
}

.load-more-spinner {
  width: 14px;
  height: 14px;
  border: 2px solid var(--border);
  border-top-color: var(--accent);
  border-radius: 50%;
  animation: spin 0.6s linear infinite;
  display: inline-block;
}

.quick-searches {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  justify-content: center;
  margin-top: 16px;
  max-width: 480px;
}

.quick-search-btn {
  padding: 6px 14px;
  border-radius: 20px;
  border: 1px solid var(--border);
  background: var(--bg-card);
  color: var(--text);
  font-family: inherit;
  font-size: 12px;
  cursor: pointer;
  transition: all 0.15s;
}

.quick-search-btn:hover {
  border-color: var(--accent);
  background: var(--accent-dim);
  color: var(--accent);
}

.content {
  flex: 1;
  display: flex;
  overflow: hidden;
}

.results-panel {
  flex: 1;
  overflow-y: auto;
  padding: 20px 24px;
}

.results-grid {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.filter-divider {
  width: 1px;
  height: 20px;
  background: var(--border);
  margin: 0 4px;
}

.playlist-card {
  cursor: pointer;
}
.playlist-card:hover {
  background: var(--bg-hover);
}

.track-card {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 12px;
  border-radius: var(--radius);
  border: 1px solid transparent;
  transition: all 0.15s;
}

.track-card:hover {
  background: var(--bg-card);
  border-color: var(--border);
}

.track-thumb {
  width: 56px;
  height: 42px;
  border-radius: 4px;
  object-fit: cover;
  background: var(--border);
  flex-shrink: 0;
}

.track-num {
  width: 32px;
  text-align: right;
  color: var(--text-faint);
  font-family: 'DM Mono', monospace;
  font-size: 12px;
  flex-shrink: 0;
}

.track-info {
  flex: 1;
  min-width: 0;
}

.track-title {
  font-size: 14px;
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.track-meta {
  font-size: 12px;
  color: var(--text-muted);
  display: flex;
  align-items: center;
  gap: 4px;
  margin-top: 2px;
}

.dot { color: var(--text-faint); }

.track-dl-btn {
  padding: 6px 12px;
  font-size: 12px;
  flex-shrink: 0;
  opacity: 0;
  transition: opacity 0.15s;
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

.track-card:hover .track-dl-btn,
.track-card:hover .track-play-btn {
  opacity: 1;
}

/* Playlist header */
.playlist-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 16px;
  padding-bottom: 16px;
  border-bottom: 1px solid var(--border);
}

.playlist-title {
  font-size: 16px;
  font-weight: 600;
}

.playlist-count {
  font-size: 12px;
  color: var(--text-muted);
  margin-top: 2px;
}

/* Downloads panel */
.downloads-panel {
  width: 260px;
  border-left: 1px solid var(--border);
  padding: 20px 16px;
  overflow-y: auto;
  flex-shrink: 0;
}

.downloads-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.download-item {
  padding: 12px;
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: var(--radius);
}

.download-item.done { border-color: rgba(34, 197, 94, 0.3); }
.download-item.error { border-color: rgba(239, 68, 68, 0.3); }

.dl-title {
  font-size: 12px;
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  margin-bottom: 8px;
}

.dl-progress { display: flex; flex-direction: column; gap: 4px; }

.dl-meta {
  display: flex;
  gap: 8px;
  font-size: 11px;
  font-family: 'DM Mono', monospace;
  color: var(--text-muted);
}

.dl-status {
  font-size: 12px;
  font-weight: 500;
}

.dl-status.done { color: var(--green); }
.dl-status.error { color: var(--red); }

/* Spinner */
.spinner {
  width: 12px;
  height: 12px;
  border: 2px solid rgba(0,0,0,0.3);
  border-top-color: #000;
  border-radius: 50%;
  animation: spin 0.6s linear infinite;
  display: inline-block;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}
</style>
