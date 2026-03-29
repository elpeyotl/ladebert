<template>
  <div class="hoerbert">
    <div class="header">
      <div class="header-left">
        <h1>Hörbert Manager</h1>
        <span class="badge badge-green" v-if="hoerbertDir">
          <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3"><path d="M20 6L9 17l-5-5"/></svg>
          SD-Karte
          <button class="eject-btn" @click.stop="ejectSdCard" title="SD-Karte auswerfen">
            <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
              <path d="M12 5l-8 10h16L12 5z"/><rect x="4" y="18" width="16" height="2" rx="1"/>
            </svg>
          </button>
        </span>
        <span class="disk-space" v-if="diskSpace">
          {{ formatBytes(diskSpace.free) }} frei von {{ formatBytes(diskSpace.total) }}
        </span>
      </div>
      <div class="header-actions">
        <button v-if="hoerbertDir && totalFiles > 0" class="btn btn-ghost btn-danger" @click="clearAllSlots">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
          </svg>
          Alles löschen
        </button>
        <button v-if="hoerbertDir" class="btn btn-ghost btn-danger" @click="formatSdCard" :disabled="formatting">
          <span v-if="formatting" class="spinner" />
          <svg v-else width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"/>
          </svg>
          {{ formatting ? 'Formatiere...' : 'Formatieren' }}
        </button>
        <button class="btn btn-ghost" @click="pickHoerbertDir">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
          </svg>
          {{ hoerbertDir ? 'SD-Karte wechseln' : 'SD-Karte wählen' }}
        </button>
      </div>
    </div>

    <!-- No folder selected -->
    <div v-if="!hoerbertDir" class="empty-screen">
      <div class="hoerbert-preview">
        <div class="device-body">
          <div class="device-screen">
            <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <path d="M9 18V5l12-2v13M9 9l12-2"/><circle cx="6" cy="18" r="3"/><circle cx="18" cy="16" r="3"/>
            </svg>
          </div>
          <div class="device-buttons">
            <div v-for="i in 9" :key="i" class="device-btn" :style="{ background: slotColors[i] }" />
          </div>
        </div>
      </div>
      <p>Wähle den Hörbert-Ordner auf der SD-Karte</p>
      <p class="hint">Normalerweise: <code>/Volumes/NO NAME</code></p>
      <button class="btn btn-primary" @click="pickHoerbertDir">Ordner wählen</button>
    </div>

    <!-- Main manager -->
    <div v-else class="manager">
      <!-- Left: Available MP3s -->
      <div class="source-panel">
        <div class="panel-header">
          <div class="section-title" :title="sourceDir">{{ sourceDirName }}</div>
          <div class="panel-actions">
            <button class="icon-btn" @click="refreshSourceFiles" title="Aktualisieren">
              <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round">
                <path d="M1 4v6h6M23 20v-6h-6"/><path d="M20.49 9A9 9 0 0 0 5.64 5.64L1 10m22 4l-4.64 4.36A9 9 0 0 1 3.51 15"/>
              </svg>
            </button>
            <button class="icon-btn" @click="pickSourceDir" title="Ordner wählen">
              <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
              </svg>
            </button>
          </div>
        </div>

        <!-- Search -->
        <div class="source-search">
          <svg class="search-icon" width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
            <circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/>
          </svg>
          <input
            class="search-input"
            v-model="fileSearch"
            placeholder="Dateien suchen..."
          />
        </div>

        <div v-if="selectedFiles.size > 0" class="selection-bar">
          <span>{{ selectedFiles.size }} ausgewählt</span>
          <button class="link-btn" @click="selectedFiles = new Set()">Aufheben</button>
        </div>
        <div v-else-if="filteredFiles.length > 0" class="selection-bar">
          <button class="link-btn" @click="selectAll">Alle auswählen</button>
        </div>

        <div v-if="!sourceFiles.length" class="mini-empty">
          <p>Keine MP3s gefunden</p>
        </div>
        <div v-else class="file-list">
          <!-- Group by folder -->
          <template v-for="(files, folder) in groupedFiles" :key="folder">
            <div v-if="folder !== '_root'" class="folder-header" @click="toggleFolder(folder as string)">
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path v-if="expandedFolders.has(folder as string)" d="M6 9l6 6 6-6"/>
                <path v-else d="M9 18l6-6-6-6"/>
              </svg>
              <span>{{ folder }}</span>
              <span class="folder-count">{{ files.length }}</span>
            </div>
            <template v-if="folder === '_root' || expandedFolders.has(folder as string)">
              <div
                v-for="file in files"
                :key="file.path"
                class="file-item"
                :class="{ 'file-selected': selectedFiles.has(file.path) }"
                @click="selectFile($event, file.path)"
              >
                <button class="play-btn" @click.stop="playSourceFile(file)" title="Vorhören">
                  <svg width="10" height="10" viewBox="0 0 24 24" fill="currentColor">
                    <path d="M8 5v14l11-7z"/>
                  </svg>
                </button>
                <span class="file-name">{{ file.name }}</span>
              </div>
            </template>
          </template>
        </div>
      </div>

      <!-- Right: Hörbert slots -->
      <div class="slots-panel">
        <div class="slots-toolbar">
          <div class="section-title">Slots</div>
          <div class="panel-actions">
            <button class="icon-btn" @click="expandAllSlots" title="Alle öffnen">
              <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><path d="M7 10l5 5 5-5"/></svg>
            </button>
            <button class="icon-btn" @click="collapseAllSlots" title="Alle schliessen">
              <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><path d="M7 14l5-5 5 5"/></svg>
            </button>
          </div>
        </div>
        <div v-if="loadingSlots" class="slots-loading">
          <div class="slots-loading-spinner" />
          <span>SD-Karte wird gelesen...</span>
        </div>
        <div v-else class="slots-list">
          <div
            v-for="slot in 9"
            :key="slot"
            class="slot"
            :class="{ 'expanded': expandedSlots.has(slot) }"
          >
            <div class="slot-header" @click="toggleSlot(slot)">
              <div class="slot-left">
                <div class="slot-num" :style="{ background: slotColors[slot] }" />
                <div class="slot-info">
                  <span class="slot-count">
                    {{ slotFiles[slot]?.length || 0 }} Tracks
                    <span v-if="slotFiles[slot]?.filter(t => t.isNew).length" class="new-count">(+{{ slotFiles[slot].filter(t => t.isNew).length }} neu)</span>
                  </span>
                </div>
              </div>
              <div class="slot-actions">
                <button
                  v-if="selectedFiles.size"
                  class="icon-btn icon-btn-sm add-btn"
                  @click.stop="addToSlot(slot)"
                  :title="`${selectedFiles.size} Datei(en) hinzufügen`"
                >
                  <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                    <path d="M12 5v14M5 12h14"/>
                  </svg>
                </button>
                <button
                  v-if="slotFiles[slot]?.length"
                  class="icon-btn icon-btn-sm"
                  @click.stop="clearSlot(slot)"
                  title="Slot leeren"
                >
                  <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                    <polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
                  </svg>
                </button>
                <svg class="slot-chevron" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path v-if="expandedSlots.has(slot)" d="M6 15l6-6 6 6"/>
                  <path v-else d="M6 9l6 6 6-6"/>
                </svg>
              </div>
            </div>
            <div v-if="expandedSlots.has(slot)" class="slot-body">
              <div v-if="!slotFiles[slot]?.length" class="slot-drop-hint">
                Dateien links auswählen, dann + klicken
              </div>
              <div v-else class="slot-files">
                <div
                  v-for="(track, i) in slotFiles[slot]"
                  :key="track.name + i"
                  class="slot-file"
                  :class="{ 'slot-file-new': track.isNew }"
                >
                  <span class="slot-file-num">{{ i + 1 }}</span>
                  <span v-if="track.isNew" class="new-badge">NEU</span>
                  <span class="slot-file-name">{{ track.name }}</span>
                  <button v-if="!track.isNew" class="play-btn" @click.stop="playSlotTrack(slot, i, track.name)" title="Abspielen">
                    <svg width="10" height="10" viewBox="0 0 24 24" fill="currentColor">
                      <path d="M8 5v14l11-7z"/>
                    </svg>
                  </button>
                  <button class="remove-btn" @click.stop="removeFromSlot(slot, track, i)">
                    <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                      <path d="M18 6L6 18M6 6l12 12"/>
                    </svg>
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- Write to card button -->
        <div class="write-bar">
          <div class="write-info">
            {{ totalFiles }} Dateien in {{ usedSlots }} Slots
            <span v-if="newFiles > 0" class="new-count">({{ newFiles }} neu)</span>
          </div>
          <button
            class="btn btn-primary"
            @click="writeToCard"
            :disabled="newFiles === 0 || writing"
          >
            <span v-if="writing" class="spinner" />
            <svg v-else width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
              <path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"/><polyline points="17 21 17 13 7 13 7 21"/><polyline points="7 3 7 8 15 8"/>
            </svg>
            Auf SD-Karte schreiben
          </button>
        </div>

        <div v-if="writeSuccess" class="success-banner">
          {{ totalFiles }} Dateien erfolgreich auf die SD-Karte geschrieben!
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, inject } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { ask } from '@tauri-apps/plugin-dialog'

const settings = inject('settings') as any
const downloadState = inject('downloadState') as any
const playFile = inject('playFile') as (title: string, artist: string, filePath: string) => void

const hoerbertDir = ref(localStorage.getItem('hoerbertDir') || '')
const defaultDownloadDir = '~/Downloads/Hörbert'
const diskSpace = ref<{ total: number; used: number; free: number } | null>(null)

function formatBytes(bytes: number): string {
  if (bytes < 1024) return bytes + ' B'
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(0) + ' KB'
  if (bytes < 1024 * 1024 * 1024) return (bytes / (1024 * 1024)).toFixed(1) + ' MB'
  return (bytes / (1024 * 1024 * 1024)).toFixed(1) + ' GB'
}

async function loadDiskSpace() {
  if (!hoerbertDir.value) { diskSpace.value = null; return }
  try {
    diskSpace.value = await invoke<{ total: number; used: number; free: number }>('get_disk_space', { path: hoerbertDir.value })
  } catch {
    diskSpace.value = null
  }
}

// hörbert button colors matching the physical device
const slotColors: Record<number, string> = {
  1: '#3d2b1f', // braun/schwarz
  2: '#e53935', // rot
  3: '#1e88e5', // blau
  4: '#43a047', // grün
  5: '#fdd835', // gelb
  6: '#00bcd4', // türkis
  7: '#81d4fa', // hellblau
  8: '#ff7043', // orange
  9: '#2e7d32', // dunkelgrün
}

function resolveSourceDir(): string {
  const fromSettings = settings.downloadDir
  if (fromSettings) return fromSettings
  const saved = localStorage.getItem('sourceDir') || ''
  if (saved && saved !== hoerbertDir.value) return saved
  return defaultDownloadDir
}

const sourceDir = ref(resolveSourceDir())
const sourceDirName = computed(() => {
  const dir = sourceDir.value
  if (!dir) return 'Downloads'
  const parts = dir.replace(/\/+$/, '').split('/')
  return parts[parts.length - 1] || 'Downloads'
})
const sourceFiles = ref<string[]>([])
const fileSearch = ref('')
const expandedFolders = ref(new Set<string>())
const expandedSlots = ref(new Set<number>())

interface SlotTrack {
  name: string
  isNew: boolean // true = vom User hinzugefügt, false = bereits auf SD-Karte
}

const slotFiles = ref<Record<number, SlotTrack[]>>({
  1: [], 2: [], 3: [], 4: [], 5: [], 6: [], 7: [], 8: [], 9: [],
})
const selectedFiles = ref(new Set<string>())
const writing = ref(false)
const writeSuccess = ref(false)
const loadingSlots = ref(false)

interface FileEntry {
  path: string   // relative path for drag/drop
  name: string   // display name (filename only)
  folder: string // parent folder or '_root'
}

const filteredFiles = computed(() => {
  const q = fileSearch.value.toLowerCase().trim()
  if (!q) return sourceFiles.value
  return sourceFiles.value.filter(f => f.toLowerCase().includes(q))
})

const groupedFiles = computed(() => {
  const groups: Record<string, FileEntry[]> = {}

  for (const filePath of filteredFiles.value) {
    const parts = filePath.split('/')
    const name = parts[parts.length - 1]
    const folder = parts.length > 1 ? parts.slice(0, -1).join('/') : '_root'

    if (!groups[folder]) groups[folder] = []
    groups[folder].push({ path: filePath, name, folder })
  }

  return groups
})

const totalFiles = computed(() =>
  Object.values(slotFiles.value).reduce((sum, tracks) => sum + tracks.length, 0)
)

const newFiles = computed(() =>
  Object.values(slotFiles.value).reduce((sum, tracks) => sum + tracks.filter(t => t.isNew).length, 0)
)

const usedSlots = computed(() =>
  Object.values(slotFiles.value).filter(f => f.length > 0).length
)

function toggleFolder(folder: string) {
  if (expandedFolders.value.has(folder)) {
    expandedFolders.value.delete(folder)
  } else {
    expandedFolders.value.add(folder)
  }
}

function toggleSlot(slot: number) {
  if (expandedSlots.value.has(slot)) {
    expandedSlots.value.delete(slot)
  } else {
    expandedSlots.value.add(slot)
  }
}

function expandAllSlots() {
  for (let i = 1; i <= 9; i++) expandedSlots.value.add(i)
}

function collapseAllSlots() {
  expandedSlots.value.clear()
}

async function clearAllSlots() {
  const yes = await ask(`Alle ${totalFiles.value} Tracks von der SD-Karte löschen?`, { title: 'Alles löschen', kind: 'warning' })
  if (!yes) return

  try {
    await invoke('clear_all_slots', { hoerbertDir: hoerbertDir.value })
    for (let i = 1; i <= 9; i++) slotFiles.value[i] = []
    saveSlots()
    await loadDiskSpace()
  } catch (e) {
    alert('Fehler beim Löschen: ' + e)
  }
}

const formatting = ref(false)

async function formatSdCard() {
  if (!hoerbertDir.value) return
  const yes = await ask(
    'SD-Karte als FAT32 formatieren?\n\nALLE DATEN werden unwiderruflich gelöscht!',
    { title: 'SD-Karte formatieren', kind: 'warning' }
  )
  if (!yes) return

  formatting.value = true
  try {
    await invoke('format_sd_card', { path: hoerbertDir.value })
    for (let i = 1; i <= 9; i++) slotFiles.value[i] = []
    saveSlots()
    // Reload after format – the mount path might be the same
    await loadSlotsFromCard()
    await loadDiskSpace()
  } catch (e) {
    alert('Formatierung fehlgeschlagen: ' + e)
  } finally {
    formatting.value = false
  }
}

async function ejectSdCard() {
  if (!hoerbertDir.value) return
  try {
    await invoke('eject_disk', { path: hoerbertDir.value })
    hoerbertDir.value = ''
    localStorage.removeItem('hoerbertDir')
    diskSpace.value = null
    for (let i = 1; i <= 9; i++) slotFiles.value[i] = []
  } catch (e) {
    alert('Auswerfen fehlgeschlagen: ' + e)
  }
}

async function pickHoerbertDir() {
  const dir = await invoke<string | null>('select_folder')
  if (dir) {
    hoerbertDir.value = dir
    localStorage.setItem('hoerbertDir', dir)
    await loadSlotsFromCard()
    await loadDiskSpace()
  }
}

async function loadSlotsFromCard() {
  if (!hoerbertDir.value) return
  loadingSlots.value = true
  try {
    const result = await invoke<Record<number, string[]>>('read_hoerbert_slots', { hoerbertDir: hoerbertDir.value })
    for (let i = 1; i <= 9; i++) {
      const existing = (result[i] || []).map(name => ({ name, isNew: false }))
      // Keep any new (unsaved) tracks that were added by the user
      const newTracks = (slotFiles.value[i] || []).filter(t => t.isNew)
      slotFiles.value[i] = [...existing, ...newTracks]
    }
    expandedSlots.value.clear()
  } catch (e) {
    console.error('Slots lesen fehlgeschlagen:', e)
  } finally {
    loadingSlots.value = false
  }
}

async function pickSourceDir() {
  const dir = await invoke<string | null>('select_folder')
  if (dir) {
    sourceDir.value = dir
    localStorage.setItem('sourceDir', dir)
    await loadSourceFiles()
  }
}

async function loadSourceFiles() {
  if (!sourceDir.value) return
  try {
    sourceFiles.value = await invoke<string[]>('list_mp3s', { dir: sourceDir.value })
  } catch (e) {
    console.error('list_mp3s error:', e)
  }
}

async function refreshSourceFiles() {
  await loadSourceFiles()
}

async function playSourceFile(file: FileEntry) {
  // Resolve the full path via backend (handles ~ expansion)
  const resolvedDir = await invoke<string>('resolve_path', { path: sourceDir.value })
  const fullPath = `${resolvedDir}/${file.path}`
  playFile(file.name, file.folder === '_root' ? 'Downloads' : file.folder, fullPath)
}

function selectFile(ev: MouseEvent, file: string) {
  if (ev.metaKey || ev.ctrlKey) {
    // Toggle individual file
    if (selectedFiles.value.has(file)) {
      selectedFiles.value.delete(file)
    } else {
      selectedFiles.value.add(file)
    }
    selectedFiles.value = new Set(selectedFiles.value) // trigger reactivity
  } else if (ev.shiftKey && selectedFiles.value.size > 0) {
    // Range select
    const allPaths = filteredFiles.value
    const lastSelected = [...selectedFiles.value].pop()!
    const startIdx = allPaths.indexOf(lastSelected)
    const endIdx = allPaths.indexOf(file)
    if (startIdx !== -1 && endIdx !== -1) {
      const [from, to] = startIdx < endIdx ? [startIdx, endIdx] : [endIdx, startIdx]
      for (let i = from; i <= to; i++) {
        selectedFiles.value.add(allPaths[i])
      }
      selectedFiles.value = new Set(selectedFiles.value)
    }
  } else {
    // Single click — toggle or replace
    if (selectedFiles.value.size === 1 && selectedFiles.value.has(file)) {
      selectedFiles.value = new Set()
    } else {
      selectedFiles.value = new Set([file])
    }
  }
}

function selectAll() {
  selectedFiles.value = new Set(filteredFiles.value)
}

function addToSlot(slot: number) {
  if (!selectedFiles.value.size) return
  for (const file of selectedFiles.value) {
    const alreadyExists = slotFiles.value[slot].some(t => t.name === file)
    if (!alreadyExists) {
      slotFiles.value[slot].push({ name: file, isNew: true })
    }
  }
  expandedSlots.value.add(slot)
  selectedFiles.value = new Set()
  saveSlots()
}

async function removeFromSlot(slot: number, track: SlotTrack, index: number) {
  const yes = await ask(`"${track.name}" aus Slot ${slot} löschen?`, { title: 'Track löschen', kind: 'warning' })
  if (!yes) return

  if (track.isNew) {
    // Not yet on SD card, just remove from local state
    slotFiles.value[slot].splice(index, 1)
    saveSlots()
  } else {
    // Delete from SD card
    const existingIndex = slotFiles.value[slot].filter((_, i) => i <= index).filter(t => !t.isNew).length - 1
    try {
      await invoke('delete_slot_track', {
        hoerbertDir: hoerbertDir.value,
        slot,
        trackIndex: existingIndex,
      })
      await loadSlotsFromCard()
      await loadDiskSpace()
    } catch (e) {
      alert('Fehler beim Löschen: ' + e)
    }
  }
}

async function clearSlot(slot: number) {
  const yes = await ask(`Alle ${slotFiles.value[slot].length} Tracks aus Slot ${slot} löschen?`, { title: 'Slot leeren', kind: 'warning' })
  if (!yes) return

  const hasExisting = slotFiles.value[slot].some(t => !t.isNew)
  try {
    if (hasExisting) {
      await invoke('clear_slot', {
        hoerbertDir: hoerbertDir.value,
        slot,
      })
    }
    slotFiles.value[slot] = []
    saveSlots()
    await loadDiskSpace()
  } catch (e) {
    alert('Fehler beim Leeren: ' + e)
  }
}

function playSlotTrack(slot: number, index: number, displayName: string) {
  if (!hoerbertDir.value) return
  const folder = slot - 1
  // Calculate the WAV file index (only count existing tracks before this one)
  const existingIndex = slotFiles.value[slot].slice(0, index + 1).filter(t => !t.isNew).length - 1
  const filePath = `${hoerbertDir.value}/${folder}/${existingIndex}.WAV`
  playFile(displayName, `Slot ${slot}`, filePath)
}

function saveSlots() {
  localStorage.setItem('slotFiles', JSON.stringify(slotFiles.value))
}

async function writeToCard() {
  if (!hoerbertDir.value || !sourceDir.value) return
  writing.value = true
  writeSuccess.value = false

  // Count total new tracks across all slots
  const totalNew = Object.values(slotFiles.value)
    .reduce((sum, tracks) => sum + tracks.filter(t => t.isNew).length, 0)
  if (totalNew === 0) { writing.value = false; return }

  let writtenTotal = 0

  downloadState.active = true
  downloadState.totalTracks = totalNew
  downloadState.currentIndex = 0
  downloadState.trackPercent = 0
  downloadState.speed = ''
  downloadState.eta = 'Konvertiere...'
  downloadState.currentTrack = ''

  const unlisten = await listen<any>('hoerbert-write-progress', (event) => {
    writtenTotal++
    downloadState.currentIndex = writtenTotal
    downloadState.trackPercent = 0
    downloadState.currentTrack = event.payload.file
    downloadState.eta = 'Konvertiere...'
  })

  try {
    for (const [slot, tracks] of Object.entries(slotFiles.value)) {
      if (downloadState.cancelled) break
      const newTracks = tracks.filter(t => t.isNew)
      if (!newTracks.length) continue
      await invoke('write_to_hoerbert', {
        hoerbertDir: hoerbertDir.value,
        slot: parseInt(slot),
        files: newTracks.map(t => t.name),
        sourceDir: sourceDir.value,
      })
    }

    if (!downloadState.cancelled) {
      writeSuccess.value = true
      downloadState.currentIndex = totalNew
      downloadState.currentTrack = 'Fertig!'
      downloadState.eta = ''

      for (let i = 1; i <= 9; i++) {
        slotFiles.value[i] = slotFiles.value[i].filter(t => !t.isNew)
      }
      await loadSlotsFromCard()
      await loadDiskSpace()
      setTimeout(() => writeSuccess.value = false, 4000)
    } else {
      downloadState.currentTrack = 'Abgebrochen'
      downloadState.eta = ''
    }
  } catch (e: any) {
    if (e !== 'Abgebrochen' && !downloadState.cancelled) {
      alert('Fehler beim Schreiben: ' + e)
    } else {
      downloadState.currentTrack = 'Abgebrochen'
      downloadState.eta = ''
    }
  } finally {
    unlisten()
    writing.value = false
    downloadState.cancelled = false
    setTimeout(() => { downloadState.active = false }, 3000)
  }
}

onMounted(async () => {
  if (sourceDir.value) loadSourceFiles()
  if (hoerbertDir.value) {
    await loadSlotsFromCard()
    await loadDiskSpace()
  }
})
</script>

<style scoped>
.hoerbert {
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

.disk-space {
  font-size: 12px;
  color: var(--text-muted);
  font-family: 'DM Mono', monospace;
}

.header-actions { display: flex; gap: 8px; }

/* Empty screen */
.empty-screen {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  text-align: center;
  color: var(--text-muted);
}

.device-body {
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: 16px;
  padding: 20px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
  margin-bottom: 8px;
}

.device-screen {
  width: 80px;
  height: 60px;
  background: var(--bg);
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--accent);
}

.device-buttons {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 6px;
}

.device-btn {
  width: 24px;
  height: 24px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 10px;
  font-weight: 700;
  font-family: 'DM Mono', monospace;
}

.hint { font-size: 12px; color: var(--text-faint); }
.hint code {
  font-family: 'DM Mono', monospace;
  background: var(--bg-card);
  padding: 2px 6px;
  border-radius: 4px;
}

/* Manager layout */
.manager {
  flex: 1;
  display: flex;
  overflow: hidden;
}

/* Source panel */
.source-panel {
  width: 320px;
  border-right: 1px solid var(--border);
  padding: 16px 12px;
  overflow-y: auto;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.panel-actions {
  display: flex;
  gap: 4px;
}

.icon-btn {
  width: 28px;
  height: 28px;
  border-radius: var(--radius);
  border: 1px solid var(--border);
  background: transparent;
  color: var(--text-muted);
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: all 0.15s;
}

.icon-btn:hover {
  background: var(--bg-hover);
  color: var(--text);
  border-color: var(--border-hover);
}

/* Source search */
.source-search {
  position: relative;
}

.source-search .search-icon {
  position: absolute;
  left: 10px;
  top: 50%;
  transform: translateY(-50%);
  color: var(--text-faint);
  pointer-events: none;
}

.source-search .search-input {
  width: 100%;
  padding: 7px 10px 7px 32px;
  border-radius: var(--radius);
  border: 1px solid var(--border);
  background: var(--bg);
  color: var(--text);
  font-size: 12px;
  font-family: inherit;
  outline: none;
  transition: border-color 0.15s;
}

.source-search .search-input:focus { border-color: var(--accent); }
.source-search .search-input::placeholder { color: var(--text-faint); }

.mini-empty {
  text-align: center;
  color: var(--text-faint);
  font-size: 12px;
  padding: 24px 8px;
}

/* Folder headers */
.folder-header {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 8px;
  margin-top: 4px;
  border-radius: 6px;
  cursor: pointer;
  font-size: 12px;
  font-weight: 600;
  color: var(--text-muted);
  transition: background 0.1s;
}

.folder-header:hover { background: var(--bg-hover); }

.folder-count {
  margin-left: auto;
  font-size: 10px;
  font-weight: 500;
  color: var(--text-faint);
  font-family: 'DM Mono', monospace;
}

.file-list { display: flex; flex-direction: column; gap: 1px; }

.file-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 5px 8px 5px 20px;
  border-radius: 6px;
  cursor: grab;
  transition: background 0.1s;
  color: var(--text-muted);
}

.file-item:hover { background: var(--bg-hover); color: var(--text); }
.selection-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 4px 8px;
  font-size: 11px;
  color: var(--text-muted);
}

.link-btn {
  background: none;
  border: none;
  color: var(--accent);
  cursor: pointer;
  font-size: 11px;
  padding: 2px 4px;
  border-radius: 3px;
}

.link-btn:hover { background: var(--accent-dim); }

.file-item .play-btn { opacity: 0; }
.file-item:hover .play-btn { opacity: 1; }
.file-item.file-selected { background: var(--accent-dim); color: var(--accent); border-left: 2px solid var(--accent); }

.file-name {
  flex: 1;
  font-size: 12px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

/* Slots panel */
.slots-panel {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.slots-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px 0;
  flex-shrink: 0;
}

.slots-loading {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 10px;
  padding: 40px 16px;
  color: var(--text-muted);
  font-size: 13px;
}

.slots-loading-spinner {
  width: 14px;
  height: 14px;
  border: 2px solid var(--border);
  border-top-color: var(--accent);
  border-radius: 50%;
  animation: spin 0.6s linear infinite;
}

.slots-list {
  flex: 1;
  padding: 12px 16px;
  display: flex;
  flex-direction: column;
  gap: 4px;
  overflow-y: auto;
  min-height: 0;
}

.slot {
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  transition: all 0.15s;
  flex-shrink: 0;
}

.slot-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 14px;
  cursor: pointer;
  transition: background 0.1s;
  min-height: 44px;
}

.slot-header:hover { background: var(--bg-hover); }

.slot-left {
  display: flex;
  align-items: center;
  gap: 10px;
}

.slot-num {
  width: 24px;
  height: 24px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 12px;
  font-weight: 700;
  font-family: 'DM Mono', monospace;
  flex-shrink: 0;
}

.slot-count {
  font-size: 12px;
  color: var(--text-muted);
}

.slot-actions {
  display: flex;
  align-items: center;
  gap: 4px;
}

.icon-btn-sm {
  width: 24px;
  height: 24px;
  opacity: 0;
  transition: all 0.15s;
}

.slot-header:hover .icon-btn-sm { opacity: 1; }
.icon-btn-sm:hover { color: var(--red) !important; border-color: var(--red) !important; }
.icon-btn-sm.add-btn { opacity: 1; }
.icon-btn-sm.add-btn:hover { color: var(--accent) !important; border-color: var(--accent) !important; }

.slot-chevron {
  color: var(--text-faint);
  flex-shrink: 0;
  transition: transform 0.15s;
}

.slot-body {
  padding: 0 12px 10px;
}

.slot-drop-hint {
  font-size: 11px;
  color: var(--text-faint);
  text-align: center;
  padding: 12px 0;
  border: 1px dashed var(--border);
  border-radius: 6px;
}

.slot-files { display: flex; flex-direction: column; gap: 2px; }

.slot-file {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 4px 8px;
  border-radius: 4px;
  transition: background 0.1s;
}

.slot-file:hover { background: var(--bg-hover); }
.slot-file-new { border-left: 2px solid var(--accent); }
.slot-file-new .slot-file-name { color: var(--accent); }

.new-badge {
  font-size: 9px;
  font-weight: 700;
  color: var(--accent);
  background: var(--accent-dim);
  padding: 1px 4px;
  border-radius: 3px;
  flex-shrink: 0;
}

.new-count { color: var(--accent); font-weight: 600; }

.btn-danger { color: var(--red) !important; }
.btn-danger:hover { background: var(--red-dim) !important; }

.eject-btn {
  background: none;
  border: none;
  color: inherit;
  cursor: pointer;
  padding: 1px;
  margin-left: 2px;
  border-radius: 3px;
  display: inline-flex;
  align-items: center;
  opacity: 0.6;
  transition: all 0.15s;
}

.eject-btn:hover { opacity: 1; background: rgba(0,0,0,0.1); }

.slot-file-num {
  font-size: 10px;
  font-family: 'DM Mono', monospace;
  color: var(--text-faint);
  width: 18px;
  text-align: right;
  flex-shrink: 0;
}

.slot-file-name {
  flex: 1;
  font-size: 12px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  color: var(--text-muted);
}

.play-btn {
  background: none;
  border: none;
  color: var(--text-faint);
  cursor: pointer;
  padding: 2px;
  border-radius: 4px;
  display: flex;
  align-items: center;
  opacity: 0;
  transition: all 0.1s;
}

.slot-file:hover .play-btn { opacity: 1; }
.play-btn:hover { color: var(--accent); }

.remove-btn {
  background: none;
  border: none;
  color: var(--text-faint);
  cursor: pointer;
  padding: 2px;
  border-radius: 4px;
  display: flex;
  align-items: center;
  opacity: 0;
  transition: all 0.1s;
}

.slot-file:hover .remove-btn { opacity: 1; }
.remove-btn:hover { color: var(--red); background: var(--red-dim); }

/* Write bar */
.write-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  border-top: 1px solid var(--border);
  background: var(--bg-card);
  flex-shrink: 0;
}

.write-info {
  font-size: 12px;
  color: var(--text-muted);
  font-family: 'DM Mono', monospace;
}

.success-banner {
  background: var(--green-dim);
  border-top: 1px solid rgba(34, 197, 94, 0.3);
  color: var(--green);
  padding: 10px 16px;
  font-size: 13px;
  font-weight: 500;
  flex-shrink: 0;
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

@keyframes spin { to { transform: rotate(360deg); } }
</style>
