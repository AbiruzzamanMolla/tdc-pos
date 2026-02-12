<script setup>
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { open, save } from '@tauri-apps/plugin-dialog';
import { logActivity } from '../utils/activityLogger';

const loading = ref(false);
const statusMessage = ref('');
const statusType = ref(''); // 'success', 'error'
const backups = ref([]);
const backupSettings = ref({
  auto_backup: 'false',
  backup_schedule: 'daily',
  keep_backups: '5',
  backup_dir: ''
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

async function loadBackups() {
  if (!backupSettings.value.backup_dir) return;
  try {
    backups.value = await invoke('list_backups', { directory: backupSettings.value.backup_dir });
  } catch (err) {
    console.error("Failed to list backups", err);
  }
}

async function loadSettings() {
  try {
    const s = await invoke('get_settings');
    if (s.auto_backup) backupSettings.value.auto_backup = s.auto_backup;
    if (s.backup_schedule) backupSettings.value.backup_schedule = s.backup_schedule;
    if (s.keep_backups) backupSettings.value.keep_backups = s.keep_backups;
    if (s.backup_dir) backupSettings.value.backup_dir = s.backup_dir;
    await loadBackups();
  } catch (err) {
    console.error("Failed to load settings", err);
  }
}

async function updateSetting(key, value) {
  try {
    await invoke('update_settings', { settings: { [key]: value.toString() } });
  } catch (err) {
    console.error("Failed to update setting", err);
  }
}

async function selectBackupDir() {
  const selected = await open({ directory: true, multiple: false });
  if (selected) {
    backupSettings.value.backup_dir = selected;
    await updateSetting('backup_dir', selected);
    await loadBackups();
    showStatus('Backup directory updated');
  }
}

async function runManualBackup() {
  try {
    loading.value = true;
    const now = new Date().toISOString().replace(/[:.]/g, '-');
    const defaultName = `tdc-pos-manual-${now}.db`;

    let dest;
    if (backupSettings.value.backup_dir) {
      dest = `${backupSettings.value.backup_dir}/${defaultName}`;
    } else {
      dest = await save({
        defaultPath: defaultName,
        filters: [{ name: 'SQLite Database', extensions: ['db'] }]
      });
    }

    if (dest) {
      await invoke('backup_db', { destinationPath: dest });
      if (backupSettings.value.backup_dir) {
        await invoke('prune_backups', {
          directory: backupSettings.value.backup_dir,
          keepN: parseInt(backupSettings.value.keep_backups)
        });
      }
      await loadBackups();
      await logActivity('BACKUP', 'Backup', null, `Manual backup created: ${defaultName}`);
      showStatus('Backup created successfully!');
    }
  } catch (error) {
    showStatus('Backup failed: ' + error, 'error');
  } finally {
    loading.value = false;
  }
}

async function restoreBackup(path) {
  if (!confirm("⚠️ RESTORE DATABASE?\n\nThis will OVERWRITE your current data with the selected backup.\nThe app will restart after restore.\n\nAre you sure?")) return;

  try {
    loading.value = true;
    await invoke('restore_db', { sourcePath: path });
    await logActivity('RESTORE', 'Backup', null, `Database restored from: ${path}`);
    showStatus('Database restored! Please restart the application.');
  } catch (error) {
    showStatus('Restore failed: ' + error, 'error');
  } finally {
    loading.value = false;
  }
}

async function handleRestoreFromFile() {
  const selected = await open({
    multiple: false,
    filters: [{ name: 'SQLite Database', extensions: ['db', 'bak'] }]
  });
  if (selected) {
    await restoreBackup(selected);
  }
}

onMounted(loadSettings);
</script>

<template>
  <div class="h-full flex flex-col space-y-6">
    <!-- Header -->
    <div class="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-3">
      <div>
        <h1 class="text-3xl font-black text-gray-900 tracking-tight">Backup & Restore</h1>
        <p class="text-gray-400 text-sm font-medium">Protect your business data</p>
      </div>
      <button @click="runManualBackup" :disabled="loading"
        class="bg-gradient-to-r from-blue-600 to-indigo-600 hover:from-blue-700 hover:to-indigo-700 text-white px-8 py-3 rounded-2xl shadow-lg shadow-blue-500/20 transition-all font-bold text-sm disabled:opacity-50 active:scale-95 flex items-center gap-2">
        <span v-if="!loading">💾</span>
        <span v-else class="animate-spin">⏳</span>
        {{ loading ? 'Processing...' : 'Create Backup Now' }}
      </button>
    </div>

    <!-- Status Toast -->
    <div v-if="statusMessage"
      class="p-4 rounded-2xl text-sm font-bold flex items-center gap-2 animate-in slide-in-from-top-4 duration-300"
      :class="statusType === 'error' ? 'bg-red-50 text-red-700 border border-red-200' : 'bg-green-50 text-green-700 border border-green-200'">
      <span>{{ statusType === 'error' ? '❌' : '✅' }}</span>
      {{ statusMessage }}
    </div>

    <div class="grid grid-cols-1 lg:grid-cols-3 gap-6 flex-1 min-h-0">
      <!-- Settings Panel -->
      <div class="lg:col-span-1 space-y-6">
        <div class="bg-white p-6 rounded-2xl shadow-sm border border-gray-100 space-y-5">
          <h2 class="text-sm font-black text-gray-900 uppercase tracking-widest border-b border-gray-50 pb-3">
            Configuration</h2>

          <!-- Auto-Backup Toggle -->
          <div class="flex items-center justify-between">
            <div>
              <label class="text-sm font-bold text-gray-700">Auto-Backup</label>
              <p class="text-[10px] text-gray-400 font-medium">Automatically protect your data</p>
            </div>
            <button
              @click="backupSettings.auto_backup = (backupSettings.auto_backup === 'true' ? 'false' : 'true'); updateSetting('auto_backup', backupSettings.auto_backup)"
              class="relative inline-flex h-7 w-12 items-center rounded-full transition-colors focus:outline-none"
              :class="backupSettings.auto_backup === 'true' ? 'bg-blue-600' : 'bg-gray-200'">
              <span class="inline-block h-5 w-5 transform rounded-full bg-white shadow transition-transform"
                :class="backupSettings.auto_backup === 'true' ? 'translate-x-6' : 'translate-x-1'"></span>
            </button>
          </div>

          <!-- Schedule -->
          <div>
            <label class="block text-[10px] font-black text-gray-400 uppercase tracking-widest mb-2">Schedule</label>
            <select v-model="backupSettings.backup_schedule"
              @change="updateSetting('backup_schedule', backupSettings.backup_schedule)"
              class="w-full bg-gray-50 border border-gray-200 rounded-xl px-3 py-2.5 text-sm font-bold focus:ring-2 focus:ring-blue-500 outline-none">
              <option value="daily">Daily</option>
              <option value="weekly">Weekly</option>
            </select>
          </div>

          <!-- Retention -->
          <div>
            <label class="block text-[10px] font-black text-gray-400 uppercase tracking-widest mb-2">Keep Latest
              Backups</label>
            <input v-model.number="backupSettings.keep_backups" type="number" min="1" max="50"
              @change="updateSetting('keep_backups', backupSettings.keep_backups)"
              class="w-full bg-gray-50 border border-gray-200 rounded-xl px-3 py-2.5 text-sm font-bold focus:ring-2 focus:ring-blue-500 outline-none">
          </div>

          <!-- Directory -->
          <div>
            <label class="block text-[10px] font-black text-gray-400 uppercase tracking-widest mb-2">Backup
              Directory</label>
            <div class="flex space-x-2">
              <input :value="backupSettings.backup_dir || 'No directory selected'" readonly
                class="flex-1 bg-gray-100 border border-gray-200 rounded-xl px-3 py-2.5 text-[10px] truncate text-gray-500 font-mono">
              <button @click="selectBackupDir"
                class="bg-gray-900 text-white px-4 py-2.5 rounded-xl text-xs font-black hover:bg-gray-800 transition-colors active:scale-95 whitespace-nowrap">
                Browse
              </button>
            </div>
          </div>
        </div>

        <!-- Restore Section -->
        <div class="bg-gradient-to-br from-amber-50 to-orange-50 p-6 rounded-2xl border border-amber-200">
          <h3 class="text-amber-800 font-black text-sm mb-2 flex items-center gap-2">
            <span>🔄</span> Restore Database
          </h3>
          <p class="text-amber-700 text-xs leading-relaxed mb-4">
            Restore from a previously exported <code class="text-[10px] bg-amber-100 px-1 py-0.5 rounded">.db</code>
            file. This will replace ALL current data.
          </p>
          <button @click="handleRestoreFromFile" :disabled="loading"
            class="w-full bg-white border-2 border-amber-300 text-amber-700 py-2.5 rounded-xl text-xs font-black hover:bg-amber-100 transition-colors active:scale-95 disabled:opacity-50">
            Select File & Restore
          </button>
        </div>

        <!-- Tip -->
        <div class="bg-blue-50 p-5 rounded-2xl border border-blue-100">
          <h3 class="text-blue-800 font-black text-xs mb-1">💡 Best Practice</h3>
          <p class="text-blue-600 text-[11px] leading-relaxed">
            Keep your backup directory on an external drive or a cloud-synced folder (Google Drive, Dropbox) for maximum
            safety against hardware failure.
          </p>
        </div>
      </div>

      <!-- Backups List -->
      <div class="lg:col-span-2 bg-white rounded-2xl shadow-sm border border-gray-100 flex flex-col min-h-0">
        <div class="p-6 border-b border-gray-50 flex justify-between items-center">
          <h2 class="text-sm font-black text-gray-900 uppercase tracking-widest">Backup History</h2>
          <span class="text-xs font-bold text-gray-400 bg-gray-50 px-3 py-1 rounded-full">{{ backups.length }}
            found</span>
        </div>

        <div class="flex-1 overflow-y-auto">
          <table v-if="backups.length > 0" class="w-full text-left">
            <thead class="bg-gray-50/50 text-[10px] font-black text-gray-400 uppercase tracking-widest sticky top-0">
              <tr>
                <th class="px-6 py-4">File</th>
                <th class="px-6 py-4">Size</th>
                <th class="px-6 py-4 text-right">Actions</th>
              </tr>
            </thead>
            <tbody class="divide-y divide-gray-50">
              <tr v-for="(b, index) in backups" :key="b.path" class="hover:bg-blue-50/20 group transition-colors">
                <td class="px-6 py-4">
                  <div class="flex items-center gap-3">
                    <span class="text-lg" :class="index === 0 ? 'animate-pulse' : ''">{{ index === 0 ? '🟢' : '💾'
                    }}</span>
                    <div>
                      <p class="text-sm font-bold text-gray-700">{{ b.name }}</p>
                      <p class="text-[10px] text-gray-400 font-mono mt-0.5">{{ b.created_at }}</p>
                    </div>
                  </div>
                </td>
                <td class="px-6 py-4">
                  <span class="text-xs font-bold text-blue-600 bg-blue-50 px-2.5 py-1 rounded-full">{{
                    formatSize(b.size) }}</span>
                </td>
                <td class="px-6 py-4 text-right">
                  <button @click="restoreBackup(b.path)" :disabled="loading"
                    class="text-xs font-black text-gray-400 hover:text-red-600 opacity-0 group-hover:opacity-100 transition-all uppercase tracking-widest bg-gray-50 hover:bg-red-50 px-3 py-1.5 rounded-lg disabled:opacity-50">
                    Restore
                  </button>
                </td>
              </tr>
            </tbody>
          </table>

          <div v-else class="flex-1 flex items-center justify-center py-20">
            <div class="text-center">
              <div class="text-5xl mb-4">📁</div>
              <p class="text-gray-400 font-bold text-sm">No backups found</p>
              <p class="text-gray-300 text-xs mt-1">{{ backupSettings.backup_dir ? 'Selected directory is empty' :
                'Select a backup directory to get started' }}</p>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
