<script setup>
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { open, save } from '@tauri-apps/plugin-dialog';

const loading = ref(false);
const backups = ref([]);
const backupSettings = ref({
  auto_backup: 'false',
  backup_schedule: 'daily',
  keep_backups: '5',
  backup_dir: ''
});

function formatSize(bytes) {
  if (bytes === 0) return '0 Bytes';
  const k = 1024;
  const sizes = ['Bytes', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
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
  const selected = await open({
    directory: true,
    multiple: false,
  });
  if (selected) {
    backupSettings.value.backup_dir = selected;
    await updateSetting('backup_dir', selected);
    await loadBackups();
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
      alert("Backup completed successfully!");
    }
  } catch (error) {
    alert("Backup failed: " + error);
  } finally {
    loading.value = false;
  }
}

async function restoreBackup(path) {
  if (!confirm("RESTORE DATABASE?\n\nThis will OVERWRITE your current data with the selected backup.\nThe app will restart after restore.")) return;

  try {
    loading.value = true;
    await invoke('restore_db', { sourcePath: path });
    alert("Database restored. Please restart the application.");
  } catch (error) {
    alert("Restore failed: " + error);
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
    <div class="flex justify-between items-center">
      <h1 class="text-3xl font-bold text-gray-800">Backup & Restore</h1>
      <button @click="runManualBackup" :disabled="loading"
        class="bg-blue-600 hover:bg-blue-700 text-white px-6 py-2 rounded-xl shadow-lg shadow-blue-500/30 transition-all font-bold disabled:opacity-50">
        {{ loading ? 'Processing...' : 'Create Manual Backup' }}
      </button>
    </div>

    <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
      <!-- Settings Panel -->
      <div class="lg:col-span-1 space-y-6">
        <div class="bg-white p-6 rounded-2xl shadow-sm border border-gray-100 space-y-4">
          <h2 class="text-lg font-bold text-gray-800 border-b pb-2">Auto-Backup Settings</h2>

          <div class="space-y-4">
            <div class="flex items-center justify-between">
              <label class="text-sm font-medium text-gray-600">Enable Auto-Backup</label>
              <button
                @click="backupSettings.auto_backup = (backupSettings.auto_backup === 'true' ? 'false' : 'true'); updateSetting('auto_backup', backupSettings.auto_backup)"
                class="relative inline-flex h-6 w-11 items-center rounded-full transition-colors focus:outline-none"
                :class="backupSettings.auto_backup === 'true' ? 'bg-blue-600' : 'bg-gray-200'">
                <span class="inline-block h-4 w-4 transform rounded-full bg-white transition-transform"
                  :class="backupSettings.auto_backup === 'true' ? 'translate-x-6' : 'translate-x-1'"></span>
              </button>
            </div>

            <div>
              <label class="block text-xs font-bold text-gray-400 uppercase tracking-widest mb-2">Schedule</label>
              <select v-model="backupSettings.backup_schedule"
                @change="updateSetting('backup_schedule', backupSettings.backup_schedule)"
                class="w-full bg-gray-50 border border-gray-200 rounded-lg px-3 py-2 text-sm focus:ring-2 focus:ring-blue-500 outline-none">
                <option value="daily">Daily</option>
                <option value="weekly">Weekly</option>
              </select>
            </div>

            <div>
              <label class="block text-xs font-bold text-gray-400 uppercase tracking-widest mb-2">Retention (Last
                N)</label>
              <input v-model.number="backupSettings.keep_backups" type="number" min="1"
                @change="updateSetting('keep_backups', backupSettings.keep_backups)"
                class="w-full bg-gray-50 border border-gray-200 rounded-lg px-3 py-2 text-sm focus:ring-2 focus:ring-blue-500 outline-none">
            </div>

            <div>
              <label class="block text-xs font-bold text-gray-400 uppercase tracking-widest mb-2">Backup
                Directory</label>
              <div class="flex space-x-2">
                <input :value="backupSettings.backup_dir" readonly placeholder="No directory selected"
                  class="flex-1 bg-gray-100 border border-gray-200 rounded-lg px-3 py-2 text-[10px] truncate">
                <button @click="selectBackupDir"
                  class="bg-gray-800 text-white px-3 py-2 rounded-lg text-xs font-bold hover:bg-gray-700">Browse</button>
              </div>
            </div>
          </div>
        </div>

        <div class="bg-blue-50 p-6 rounded-2xl border border-blue-100">
          <h3 class="text-blue-800 font-bold mb-2 text-sm">Pro Tip</h3>
          <p class="text-blue-600 text-xs leading-relaxed">
            Always keep your backups on an external drive or cloud-synced folder for maximum safety.
          </p>
          <button @click="handleRestoreFromFile"
            class="mt-4 w-full bg-white border border-blue-200 text-blue-600 py-2 rounded-lg text-xs font-bold hover:bg-blue-100 transition-colors">
            Restore from external file
          </button>
        </div>
      </div>

      <!-- Backups List -->
      <div class="lg:col-span-2 bg-white rounded-2xl shadow-sm border border-gray-100 flex flex-col">
        <div class="p-6 border-b border-gray-50 flex justify-between items-center">
          <h2 class="text-lg font-bold text-gray-800">Recent Backups</h2>
          <span class="text-xs text-gray-400">{{ backups.length }} found</span>
        </div>

        <div class="flex-1 overflow-y-auto max-h-[600px]">
          <table class="w-full text-left">
            <thead class="bg-gray-50/50 text-[10px] font-black text-gray-400 uppercase tracking-widest sticky top-0">
              <tr>
                <th class="px-6 py-4">Filename</th>
                <th class="px-6 py-4">Size</th>
                <th class="px-6 py-4 text-right">Actions</th>
              </tr>
            </thead>
            <tbody class="divide-y divide-gray-50">
              <tr v-for="b in backups" :key="b.path" class="hover:bg-blue-50/20 group transition-colors">
                <td class="px-6 py-4">
                  <p class="text-sm font-bold text-gray-700">{{ b.name }}</p>
                  <p class="text-[10px] text-gray-400 font-mono mt-0.5">{{ b.created_at }}</p>
                </td>
                <td class="px-6 py-4">
                  <span class="text-xs font-bold text-blue-600 bg-blue-50 px-2 py-0.5 rounded-full">
                    {{ formatSize(b.size) }}
                  </span>
                </td>
                <td class="px-6 py-4 text-right">
                  <button @click="restoreBackup(b.path)"
                    class="text-xs font-black text-gray-400 hover:text-red-600 opacity-0 group-hover:opacity-100 transition-all uppercase tracking-widest">
                    Restore
                  </button>
                </td>
              </tr>
              <tr v-if="backups.length === 0">
                <td colspan="3" class="px-6 py-10 text-center italic text-gray-400 text-sm">
                  No backups found in selected directory.
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </div>
  </div>
</template>
