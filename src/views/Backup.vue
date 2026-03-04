<script setup>
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { readFile, BaseDirectory, writeFile, exists, readDir, mkdir, remove } from '@tauri-apps/plugin-fs';
import { logActivity } from '../utils/activityLogger';
import JSZip from 'jszip';
import { useAuthStore } from '../stores/auth';

const auth = useAuthStore();

const loading = ref(false);
const statusMessage = ref('');
const statusType = ref(''); // 'success', 'error'
const backups = ref([]);

// Configuration object simplified as we mostly connect to the API now
const backupSettings = ref({
  auto_backup: 'false',
  backup_schedule: 'daily',
  keep_backups: '4' // The server manages the 4 items limit natively, this is just for local context if we ever need it
});

function formatSize(bytes) {
  if (!bytes || bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
}

function showStatus(msg, type = 'success') {
  statusMessage.value = msg;
  statusType.value = type;
  setTimeout(() => { statusMessage.value = ''; }, 5000);
}

// ----------------- CLOUD API LOGIC -------------------

async function loadCloudBackups() {
  const token = localStorage.getItem('tdc_license_token');
  if (!token) return;

  try {
    loading.value = true;
    const response = await fetch(`https://audiobookbangla.com/api/tdc/zips?token=${token}`, {
      method: 'GET',
      headers: { 'Accept': 'application/json' }
    });

    if (response.ok) {
      const data = await response.json();
      if (data.status === 'success') {
        backups.value = data.files.map(f => ({
          name: f.name,
          url: f.url,
          size: f.size,
          created_at: f.last_modified
        })).sort((a, b) => new Date(b.created_at) - new Date(a.created_at));
      }
    } else {
      console.warn('Failed to fetch backup list', response.status);
    }
  } catch (err) {
    console.error("Failed to list cloud backups", err);
  } finally {
    loading.value = false;
  }
}

async function runManualBackup() {
  if (auth.isDemo) {
    alert("View-only account: Cannot create backups.");
    return;
  }

  const token = localStorage.getItem('tdc_license_token');
  if (!token) {
    showStatus('License token missing! Cannot connect to cloud.', 'error');
    return;
  }

  try {
    loading.value = true;
    showStatus('Packaging backup locally...');

    // 1. Export DB to internal temp
    await invoke('backup_db', { destinationPath: "INTERNAL_TEMP" });
    const dbBytes = await readFile('temp_backup.db', { baseDir: BaseDirectory.AppData });

    // 2. Wrap DB and images in a ZIP
    const zip = new JSZip();
    zip.file('backup.db', dbBytes);

    try {
      const hasImages = await exists('images', { baseDir: BaseDirectory.AppData });
      if (hasImages) {
        const imageEntries = await readDir('images', { baseDir: BaseDirectory.AppData });
        const imgFolder = zip.folder('images');
        for (const entry of imageEntries) {
          if (entry.isFile) {
            const imgBytes = await readFile(`images/${entry.name}`, { baseDir: BaseDirectory.AppData });
            imgFolder.file(entry.name, imgBytes);
          }
        }
      }
    } catch (e) {
      console.log("No images folder found or error reading it:", e);
    }

    const zipBytes = await zip.generateAsync({ type: 'uint8array', compression: "DEFLATE" });
    await remove('temp_backup.db', { baseDir: BaseDirectory.AppData }).catch(e => console.warn(e));

    // 3. Convert generated ZIP array to a Blob
    showStatus('Uploading backup to TDC cloud...');
    const blob = new Blob([zipBytes], { type: 'application/zip' });
    const formData = new FormData();
    formData.append('token', token);
    formData.append('file', blob, `tdc-pos-cloud-${new Date().toISOString().replace(/[:.]/g, '-')}.zip`);

    // 4. Send to API endpoint
    const response = await fetch('https://audiobookbangla.com/api/tdc/upload-zip', {
      method: 'POST',
      headers: { 'Accept': 'application/json' },
      body: formData
    });

    const data = await response.json();

    if (response.ok && data.status === 'success') {
      await logActivity('BACKUP', 'Backup', null, `Cloud backup uploaded: ${data.file_name}`);
      showStatus('Cloud Backup uploaded successfully!');
      await loadCloudBackups();
    } else {
      throw new Error(data.message || 'Upload API rejected the request');
    }

  } catch (error) {
    showStatus('Backup failed: ' + error.message, 'error');
  } finally {
    loading.value = false;
  }
}

async function restoreCloudBackup(item) {
  if (auth.isDemo) {
    alert("View-only account: Cannot restore from backup.");
    return;
  }
  if (!confirm(`⚠️ RESTORE FROM CLOUD: ${item.name}?\n\nThis will OVERWRITE your current data with the selected cloud backup.\nThe app will restart after restore.\n\nAre you sure?`)) return;

  const token = localStorage.getItem('tdc_license_token');
  if (!token) {
    showStatus('License token missing! Cannot connect to cloud.', 'error');
    return;
  }

  try {
    loading.value = true;
    showStatus(`Downloading ${item.name}...`);

    // 1. Fetch file as arrayBuffer strictly overriding Accept
    const downloadUrl = `https://audiobookbangla.com/api/tdc/zips/${item.name}?token=${token}`;
    const response = await fetch(downloadUrl);
    if (!response.ok) throw new Error(`HTTP ${response.status} when downloading zip`);

    // Convert to uint8 array to simulate readFile behavior natively
    const arrayBuffer = await response.arrayBuffer();
    const contents = new Uint8Array(arrayBuffer);

    await restoreFromBytes(contents, `Cloud: ${item.name}`);
  } catch (error) {
    showStatus('Restore failed: ' + error.message, 'error');
  } finally {
    loading.value = false;
  }
}

async function restoreFromBytes(contents, sourceName) {
  showStatus(`Extracting archive...`);

  if (contents[0] === 0x50 && contents[1] === 0x4B) {
    const zip = await JSZip.loadAsync(contents);

    const dbFile = zip.file('backup.db') || zip.file('temp_backup.db');
    if (dbFile) {
      const dbBytes = await dbFile.async('uint8array');
      await writeFile('restore.db', dbBytes, { baseDir: BaseDirectory.AppData });
    } else {
      throw new Error("Invalid backup: No database file found in the archive.");
    }

    const imgFolder = zip.folder('images');
    if (imgFolder) {
      // ensure images directory exists
      const hasImages = await exists('images', { baseDir: BaseDirectory.AppData });
      if (!hasImages) {
        await mkdir('images', { baseDir: BaseDirectory.AppData });
      }
      for (const relativePath in zip.files) {
        if (relativePath.startsWith('images/') && !zip.files[relativePath].dir) {
          const imgBytes = await zip.files[relativePath].async('uint8array');
          const filename = relativePath.substring(7);
          if (filename) {
            await writeFile(`images/${filename}`, imgBytes, { baseDir: BaseDirectory.AppData });
          }
        }
      }
    }
  } else {
    // Old DB format backward compatibility support layer just in case people try to load raw .db files locally
    await writeFile('restore.db', contents, { baseDir: BaseDirectory.AppData });
  }

  await logActivity('RESTORE', 'Backup', null, `Database restored from ${sourceName}`);
  showStatus('Data and images restored! Please restart the application.');
}

async function handleRestoreFromFile() {
  const selected = await open({
    multiple: false
  });
  if (selected) {
    if (auth.isDemo) {
      alert("View-only account: Cannot restore from backup.");
      return;
    }
    if (!confirm(`⚠️ RESTORE FROM LOCAL FILE: ${selected}?\n\nThis will OVERWRITE your current data with the selected file.\nThe app will restart after restore.\n\nAre you sure?`)) return;

    try {
      loading.value = true;
      showStatus('Reading local file...');
      const contents = await readFile(selected);
      await restoreFromBytes(contents, `Local File: ${selected.split(/[\/\\]/).pop() || selected}`);
    } catch (err) {
      showStatus('Local Restore failed: ' + err.message, 'error');
    } finally {
      loading.value = false;
    }
  }
}

// ----------------- LOCAL CONFIG MANAGEMENT -------------------

async function loadSettings() {
  try {
    const s = await invoke('get_settings');
    if (s.auto_backup) backupSettings.value.auto_backup = s.auto_backup;
    if (s.backup_schedule) backupSettings.value.backup_schedule = s.backup_schedule;
  } catch (err) {
    console.error("Failed to load settings", err);
  }
}

async function updateSetting(key, value) {
  if (auth.isDemo) {
    alert("View-only account: Cannot update backup settings.");
    return;
  }
  try {
    await invoke('update_settings', { settings: { [key]: value.toString() } });
  } catch (err) {
    console.error("Failed to update setting", err);
  }
}

onMounted(async () => {
  await loadSettings();
  await loadCloudBackups();
});
</script>

<template>
  <div class="h-full flex flex-col space-y-6">
    <!-- Header -->
    <div class="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-3">
      <div>
        <h1 class="text-3xl font-black text-gray-900 tracking-tight">Cloud Backup & Restore</h1>
        <p class="text-gray-400 text-sm font-medium">Protect and synchronize your business data securely</p>
      </div>
      <button v-if="!auth.isDemo" @click="runManualBackup" :disabled="loading"
        class="bg-gradient-to-r from-indigo-500 to-purple-600 hover:from-indigo-600 hover:to-purple-700 text-white px-8 py-3 rounded-2xl shadow-lg shadow-indigo-500/20 transition-all font-bold text-sm disabled:opacity-50 active:scale-95 flex items-center gap-2">
        <span v-if="!loading">☁️</span>
        <span v-else class="animate-spin">⏳</span>
        {{ loading ? 'Uploading...' : 'Upload Cloud Backup' }}
      </button>
    </div>

    <!-- Status Toast -->
    <div v-if="statusMessage"
      class="p-4 rounded-2xl text-sm font-bold flex items-center gap-2 animate-in slide-in-from-top-4 duration-300"
      :class="statusType === 'error' ? 'bg-red-50 text-red-700 border border-red-200' : 'bg-green-50 text-green-700 border border-green-200'">
      <span v-if="loading && statusType !== 'error'" class="animate-spin text-lg">⚙️</span>
      <span v-else>{{ statusType === 'error' ? '❌' : '✅' }}</span>
      {{ statusMessage }}
    </div>

    <div class="grid grid-cols-1 lg:grid-cols-3 gap-6 flex-1 min-h-0">
      <!-- Settings Panel -->
      <div class="lg:col-span-1 space-y-6">
        <div class="bg-white p-6 rounded-2xl shadow-sm border border-gray-100 space-y-5">
          <h2 class="text-sm font-black text-gray-900 uppercase tracking-widest border-b border-gray-50 pb-3">Automated
            Protection</h2>

          <!-- Auto-Backup Toggle -->
          <div class="flex items-center justify-between">
            <div>
              <label class="text-sm font-bold text-gray-700">Cloud Sync</label>
              <p class="text-[10px] text-gray-400 font-medium">Automatically upload daily backups</p>
            </div>
            <button
              @click="backupSettings.auto_backup = (backupSettings.auto_backup === 'true' ? 'false' : 'true'); updateSetting('auto_backup', backupSettings.auto_backup)"
              class="relative inline-flex h-7 w-12 items-center rounded-full transition-colors focus:outline-none"
              :class="backupSettings.auto_backup === 'true' ? 'bg-indigo-600' : 'bg-gray-200'">
              <span class="inline-block h-5 w-5 transform rounded-full bg-white shadow transition-transform"
                :class="backupSettings.auto_backup === 'true' ? 'translate-x-6' : 'translate-x-1'"></span>
            </button>
          </div>

          <!-- Description Box -->
          <div class="mt-4 p-4 rounded-xl bg-gray-50 border border-gray-100">
            <h4 class="text-xs font-black text-gray-800 uppercase tracking-wide mb-1 flex items-center gap-2">
              <span class="text-base">🔐</span> TDC Managed Access
            </h4>
            <p class="text-[10px] leading-relaxed text-gray-600 mt-2">
              Your data is automatically synchronized and protected through the TDC enterprise network via an isolated
              snapshot capability. Older backups are automatically cycled reducing disk footprint safely.
            </p>
          </div>
        </div>

        <div class="bg-gradient-to-br from-indigo-50 to-purple-50 p-6 rounded-2xl border border-indigo-200">
          <h3 class="text-indigo-800 font-black text-sm mb-2 flex items-center gap-2">
            <span>📡</span> Remote Restore
          </h3>
          <p class="text-indigo-700 text-xs leading-relaxed mb-4">
            Select an archive from the Server Backups list opposite to immediately retrieve and overwrite your local
            dataset.
          </p>
          <div class="border-t border-indigo-200/50 pt-4 mt-2">
            <h3 class="text-indigo-800 font-black text-sm mb-2 flex items-center gap-2">
              <span>📁</span> Local Fallback
            </h3>
            <p class="text-indigo-700 text-xs leading-relaxed mb-4">
              Restore from a downloaded <code class="text-[10px] bg-indigo-100 px-1 py-0.5 rounded">.zip</code> or <code
                class="text-[10px] bg-indigo-100 px-1 py-0.5 rounded">.db</code> file.
            </p>
            <button @click="handleRestoreFromFile" :disabled="loading"
              class="w-full bg-white border-2 border-indigo-300 text-indigo-700 py-2.5 rounded-xl text-xs font-black hover:bg-indigo-100 transition-colors active:scale-95 disabled:opacity-50">
              Select Local File & Restore
            </button>
          </div>
        </div>
      </div>

      <!-- Cloud Backups List -->
      <div class="lg:col-span-2 bg-white rounded-2xl shadow-sm border border-gray-100 flex flex-col min-h-0">
        <div class="p-6 border-b border-gray-50 flex justify-between items-center bg-gray-50/20">
          <h2 class="text-sm font-black text-gray-900 uppercase tracking-widest flex items-center gap-2">
            Server Archives
            <span v-if="loading" class="animate-spin text-indigo-500">⚙️</span>
          </h2>
          <span
            class="text-[10px] font-black tracking-widest text-indigo-500 bg-indigo-50 px-3 py-1.5 rounded-full ring-1 ring-indigo-500/20">
            {{ backups.length }} Cloud Snapshots
          </span>
        </div>

        <div class="flex-1 overflow-y-auto">
          <table v-if="backups.length > 0" class="w-full text-left">
            <thead
              class="bg-white text-[10px] font-black text-gray-400 uppercase tracking-widest sticky top-0 shadow-sm z-10">
              <tr>
                <th class="px-6 py-4">Archive Details</th>
                <th class="px-6 py-4">Payload Size</th>
                <th class="px-6 py-4 text-right">Verification</th>
              </tr>
            </thead>
            <tbody class="divide-y divide-gray-50">
              <tr v-for="(b, index) in backups" :key="b.name" class="hover:bg-indigo-50/20 group transition-colors">
                <td class="px-6 py-4">
                  <div class="flex items-center gap-3">
                    <span class="text-xl"
                      :class="index === 0 ? 'opacity-100 drop-shadow-sm' : 'opacity-60 saturate-0'">{{ index === 0 ?
                        '🌩️' : '☁️' }}</span>
                    <div>
                      <p class="text-sm font-bold text-gray-800 break-all">{{ b.name }}</p>
                      <p class="text-[10px] text-gray-400 font-mono mt-1 text-indigo-500/80">{{ b.created_at }}</p>
                    </div>
                  </div>
                </td>
                <td class="px-6 py-4">
                  <span
                    class="text-[10px] font-black tracking-widest text-gray-600 bg-gray-50 px-2.5 py-1 rounded-full uppercase">{{
                      formatSize(b.size) }}</span>
                </td>
                <td class="px-6 py-4 text-right">
                  <button v-if="!auth.isDemo" @click="restoreCloudBackup(b)" :disabled="loading"
                    class="text-[10px] font-black text-indigo-600 opacity-0 group-hover:opacity-100 transition-all uppercase tracking-widest ring-1 ring-indigo-500/30 hover:bg-indigo-600 hover:text-white hover:ring-indigo-600 shadow-sm px-4 py-2 rounded-xl disabled:opacity-50 inline-flex items-center gap-2">
                    ⬇️ Retrieve
                  </button>
                </td>
              </tr>
            </tbody>
          </table>

          <div v-else class="flex-1 flex items-center justify-center py-24 bg-gray-50/30 h-full">
            <div class="text-center max-w-sm px-6">
              <div class="text-6xl mb-6 opacity-30 drop-shadow-sm">🧊</div>
              <p class="text-gray-900 font-black text-lg mb-1 tracking-tight">Vast Emptiness</p>
              <p class="text-gray-500 text-xs leading-relaxed mt-2" v-if="!loading">No server backups are present for
                your token yet. Press "Upload Cloud Backup" to initialize your first sync.</p>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
