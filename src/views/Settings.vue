<script setup>
import { ref, onMounted, reactive } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { logActivity } from '../utils/activityLogger';

const loading = ref(false);
const message = ref("");

const settings = reactive({
  store_name: "",
  store_address: "",
  store_phone: "",
  store_email: "",
  currency_symbol: "৳",
  tax_rate: "0"
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

onMounted(() => {
  loadSettings();
});
</script>

<template>
  <div class="max-w-4xl mx-auto">
    <h1 class="text-3xl font-bold text-gray-800 mb-6">Settings</h1>

    <div class="bg-white rounded-xl shadow border border-gray-100 p-8">
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

      <h2 class="text-xl font-bold text-gray-700 mb-6 border-b pb-2 mt-8">Configuration</h2>

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

      <div class="flex items-center justify-between mt-8 pt-6 border-t">
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
</template>
