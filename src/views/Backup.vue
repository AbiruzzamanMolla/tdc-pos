<script setup>
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { save, open } from '@tauri-apps/plugin-dialog';

const loading = ref(false);
const message = ref("");
const error = ref("");

async function backupDatabase() {
  loading.value = true;
  message.value = "";
  error.value = "";

  try {
    const filePath = await save({
      filters: [{
        name: 'TDC POS Backup',
        extensions: ['db']
      }],
      defaultPath: 'tdc-pos-backup.db'
    });

    if (!filePath) {
      loading.value = false;
      return;
    }

    await invoke('backup_db', { destinationPath: filePath });
    message.value = "Backup created successfully at: " + filePath;
  } catch (err) {
    console.error(err);
    error.value = "Backup failed: " + err;
  } finally {
    loading.value = false;
  }
}

async function restoreDatabase() {
  loading.value = true;
  message.value = "";
  error.value = "";

  try {
    const filePath = await open({
      filters: [{
        name: 'SQLite Database',
        extensions: ['db']
      }]
    });

    if (!filePath) {
      loading.value = false;
      return;
    }

    const confirm = await window.confirm("Restoring will overwrite the current database. This action is irreversible. The app will need to restart. Continue?");
    if (!confirm) {
      loading.value = false;
      return;
    }

    await invoke('restore_db', { sourcePath: filePath });
    message.value = "Restore prepared successfully. Please restart the application to apply changes.";
    alert("Restore pending. Please restart the application now.");
  } catch (err) {
    console.error(err);
    error.value = "Restore failed: " + err;
  } finally {
    loading.value = false;
  }
}
</script>

<template>
  <div class="h-full flex flex-col justify-center items-center max-w-2xl mx-auto text-center space-y-8">

    <div>
      <h1 class="text-3xl font-bold text-gray-800 mb-2">Backup & Restore</h1>
      <p class="text-gray-500">Manage your data securely. Create regular backups to avoid data loss.</p>
    </div>

    <!-- Actions -->
    <div class="grid grid-cols-1 md:grid-cols-2 gap-6 w-full">
      <!-- Backup -->
      <div
        class="bg-white p-8 rounded-xl shadow-lg border border-gray-100 hover:shadow-xl transition relative overflow-hidden group">
        <div class="absolute inset-0 bg-blue-500 opacity-0 group-hover:opacity-5 transition"></div>
        <div class="relative z-10">
          <div class="text-5xl mb-4">💾</div>
          <h3 class="text-xl font-bold text-gray-800 mb-2">Backup Database</h3>
          <p class="text-gray-500 text-sm mb-6">Save the current state of your sales, products, and inventory to a
            secure file.</p>
          <button @click="backupDatabase" :disabled="loading"
            class="bg-blue-600 hover:bg-blue-700 text-white font-bold py-3 px-6 rounded-lg w-full transition disabled:opacity-50">
            Create Backup
          </button>
        </div>
      </div>

      <!-- Restore -->
      <div
        class="bg-white p-8 rounded-xl shadow-lg border border-gray-100 hover:shadow-xl transition relative overflow-hidden group">
        <div class="absolute inset-0 bg-red-500 opacity-0 group-hover:opacity-5 transition"></div>
        <div class="relative z-10">
          <div class="text-5xl mb-4">♻️</div>
          <h3 class="text-xl font-bold text-gray-800 mb-2">Restore Database</h3>
          <p class="text-gray-500 text-sm mb-6">Restore data from a previously saved backup file. Existing data will be
            replaced.</p>
          <button @click="restoreDatabase" :disabled="loading"
            class="bg-white border-2 border-red-500 text-red-600 hover:bg-red-50 font-bold py-3 px-6 rounded-lg w-full transition disabled:opacity-50">
            Restore from File
          </button>
        </div>
      </div>
    </div>

    <!-- Status Messages -->
    <div v-if="loading" class="text-blue-600 font-medium animate-pulse">
      Processing... Please wait.
    </div>

    <div v-if="message" class="bg-green-100 border border-green-200 text-green-700 px-6 py-4 rounded-lg w-full">
      {{ message }}
    </div>

    <div v-if="error" class="bg-red-100 border border-red-200 text-red-700 px-6 py-4 rounded-lg w-full">
      {{ error }}
    </div>

  </div>
</template>
