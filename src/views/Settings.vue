<script setup>
import { ref, onMounted, reactive } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { logActivity } from '../utils/activityLogger';
import { useAuthStore } from '../stores/auth';

const auth = useAuthStore();
const loading = ref(false);
const message = ref("");
const cleanupMsg = ref("");

const cleanupOptions = reactive({
  cleanSales: false,
  cleanPurchases: false,
  cleanProducts: false,
  cleanLogs: false
});

const settings = reactive({
  store_name: "",
  store_address: "",
  store_phone: "",
  store_email: "",
  currency_symbol: "৳",
  tax_rate: "0",
  google_ai_key: ""
});

async function loadSettings() {
  try {
    const data = await invoke('get_settings');
    Object.keys(settings).forEach(key => {
      if (data[key]) {
        settings[key] = data[key];
      }
    });
  } catch (error) {
    console.error("Failed to load settings:", error);
  }
}

async function saveSettings() {
  loading.value = true;
  message.value = "";

  try {
    // Convert reactive object to plain map
    const settingsMap = { ...settings };
    await invoke('update_settings', { settings: settingsMap });
    await logActivity('SETTINGS', 'Settings', null, `Settings updated: ${Object.keys(settingsMap).join(', ')}`);
    message.value = "Settings saved successfully!";
    setTimeout(() => message.value = "", 3000);
  } catch (error) {
    console.error("Failed to save settings:", error);
    alert("Error saving settings: " + error);
  } finally {
    loading.value = false;
  }
}

async function handleCleanup() {
  if (!cleanupOptions.cleanSales && !cleanupOptions.cleanPurchases &&
    !cleanupOptions.cleanProducts && !cleanupOptions.cleanLogs) {
    alert("Please select at least one option to clean.");
    return;
  }

  const code = Math.floor(1000 + Math.random() * 9000);
  const input = prompt(`DANGER ZONE: You are about to PERMANENTLY DELETE data.\n\nType "${code}" to confirm:`);

  if (input !== String(code)) {
    alert("Incorrect confirmation code. Aborted.");
    return;
  }

  try {
    loading.value = true;
    cleanupMsg.value = "";
    await invoke('cleanup_database', { ...cleanupOptions });
    await logActivity('CLEANUP', 'System', null, `Database cleanup executed. Options: ${JSON.stringify(cleanupOptions)}`);
    cleanupMsg.value = "Database cleanup successful.";

    // Reset options
    Object.keys(cleanupOptions).forEach(k => cleanupOptions[k] = false);

    setTimeout(() => cleanupMsg.value = "", 5000);
  } catch (err) {
    console.error("Cleanup failed:", err);
    alert("Cleanup failed: " + err);
  } finally {
    loading.value = false;
  }
}

onMounted(() => {
  loadSettings();
});
</script>

<template>
  <div class="max-w-4xl mx-auto">
    <h1 class="text-3xl font-bold text-gray-800 mb-6">Settings</h1>

    <div class="bg-white rounded-xl shadow border border-gray-100 p-8 space-y-8">

      <!-- Store Info Section -->
      <div>
        <h2 class="text-xl font-bold text-gray-700 mb-6 border-b pb-2">Store Information</h2>

        <div class="grid grid-cols-1 md:grid-cols-2 gap-6 mb-6">
          <div>
            <label class="block text-sm font-medium text-gray-700 mb-1">Store Name</label>
            <input v-model="settings.store_name" type="text"
              class="w-full border border-gray-300 rounded-lg px-4 py-2 focus:ring-blue-500 focus:outline-none">
          </div>

          <div>
            <label class="block text-sm font-medium text-gray-700 mb-1">Phone Number</label>
            <input v-model="settings.store_phone" type="text"
              class="w-full border border-gray-300 rounded-lg px-4 py-2 focus:ring-blue-500 focus:outline-none">
          </div>

          <div class="md:col-span-2">
            <label class="block text-sm font-medium text-gray-700 mb-1">Address</label>
            <input v-model="settings.store_address" type="text"
              class="w-full border border-gray-300 rounded-lg px-4 py-2 focus:ring-blue-500 focus:outline-none">
          </div>

          <div>
            <label class="block text-sm font-medium text-gray-700 mb-1">Email</label>
            <input v-model="settings.store_email" type="email"
              class="w-full border border-gray-300 rounded-lg px-4 py-2 focus:ring-blue-500 focus:outline-none">
          </div>
        </div>
      </div>

      <!-- Config Section -->
      <div>
        <h2 class="text-xl font-bold text-gray-700 mb-6 border-b pb-2">Configuration</h2>

        <div class="grid grid-cols-1 md:grid-cols-2 gap-6 mb-6">
          <div>
            <label class="block text-sm font-medium text-gray-700 mb-1">Currency Symbol</label>
            <input v-model="settings.currency_symbol" type="text"
              class="w-full border border-gray-300 rounded-lg px-4 py-2 focus:ring-blue-500 focus:outline-none">
          </div>

          <div>
            <label class="block text-sm font-medium text-gray-700 mb-1">Default Tax Rate (%)</label>
            <input v-model="settings.tax_rate" type="number" step="0.01"
              class="w-full border border-gray-300 rounded-lg px-4 py-2 focus:ring-blue-500 focus:outline-none">
          </div>
        </div>

        <!-- AI Settings Section -->
        <div>
          <h2 class="text-xl font-bold text-gray-700 mb-6 border-b pb-2 flex items-center gap-2">
            <span>🤖</span> AI Configuration
          </h2>

          <div class="mb-6">
            <label class="block text-sm font-medium text-gray-700 mb-1">Google AI Studio API Key</label>
            <div class="relative">
              <input v-model="settings.google_ai_key" type="password"
                placeholder="Enter your API key (starts with AIza...)"
                class="w-full border border-gray-300 rounded-lg px-4 py-2 focus:ring-blue-500 focus:outline-none">
            </div>
            <p class="text-xs text-gray-500 mt-1">
              Required for the Chat Assistant. Get your key from
              <a href="https://aistudio.google.com/" target="_blank" class="text-blue-600 hover:underline">Google AI
                Studio</a>.
            </p>
          </div>

          <!-- Save Button Section -->
          <div class="flex items-center justify-between pt-6 border-t">
            <div v-if="message" class="text-green-600 font-medium bg-green-50 px-3 py-1 rounded">
              {{ message }}
            </div>
            <div v-else></div> <!-- Spacer -->

            <button @click="saveSettings" :disabled="loading"
              class="bg-blue-600 hover:bg-blue-700 text-white font-bold py-3 px-8 rounded-lg shadow-lg transition disabled:opacity-50">
              {{ loading ? 'Saving...' : 'Save Settings' }}
            </button>
          </div>
        </div>

      </div>

    </div>

    <!-- Database Cleanup (Super Admin Only) -->
    <div v-if="auth.isSuperAdmin" class="mt-8 bg-white rounded-xl shadow border border-red-200 p-8">
      <h2 class="text-xl font-bold text-red-600 mb-4 flex items-center gap-2">
        <span>⚠️</span> Database Cleanup
      </h2>
      <div class="bg-red-50 border border-red-100 rounded-xl p-6">
        <p class="text-sm text-red-700 mb-4 font-medium">
          Select data to permanently delete. This action cannot be undone.
        </p>
        <div class="grid grid-cols-1 sm:grid-cols-2 gap-4 mb-6">
          <label
            class="flex items-center space-x-3 bg-white p-3 rounded-lg border border-red-100 cursor-pointer hover:bg-red-50/50 transition">
            <input v-model="cleanupOptions.cleanSales" type="checkbox"
              class="w-5 h-5 text-red-600 rounded focus:ring-red-500 border-gray-300">
            <span class="text-gray-800 font-bold">Sales History</span>
          </label>
          <label
            class="flex items-center space-x-3 bg-white p-3 rounded-lg border border-red-100 cursor-pointer hover:bg-red-50/50 transition">
            <input v-model="cleanupOptions.cleanPurchases" type="checkbox"
              class="w-5 h-5 text-red-600 rounded focus:ring-red-500 border-gray-300">
            <span class="text-gray-800 font-bold">Purchase History</span>
          </label>
          <label
            class="flex items-center space-x-3 bg-white p-3 rounded-lg border border-red-100 cursor-pointer hover:bg-red-50/50 transition">
            <input v-model="cleanupOptions.cleanProducts" type="checkbox"
              class="w-5 h-5 text-red-600 rounded focus:ring-red-500 border-gray-300">
            <div>
              <span class="text-gray-800 font-bold block">All Inventory</span>
              <span class="text-xs text-red-500 font-bold uppercase tracking-wide">Deletes Sales & Purchases
                too</span>
            </div>
          </label>
          <label
            class="flex items-center space-x-3 bg-white p-3 rounded-lg border border-red-100 cursor-pointer hover:bg-red-50/50 transition">
            <input v-model="cleanupOptions.cleanLogs" type="checkbox"
              class="w-5 h-5 text-red-600 rounded focus:ring-red-500 border-gray-300">
            <span class="text-gray-800 font-bold">Activity Logs</span>
          </label>
        </div>

        <div class="flex justify-between items-center">
          <span class="text-sm font-bold text-green-600">{{ cleanupMsg }}</span>
          <button @click="handleCleanup"
            class="bg-red-600 hover:bg-red-700 text-white font-black py-2.5 px-6 rounded-lg shadow-lg transition transform active:scale-95 flex items-center gap-2">
            <span>🗑️</span> Clean Database
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
