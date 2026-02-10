<script setup>
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';

const stats = ref({
  total_sales: 0,
  sales_today: 0,
  sales_month: 0,
  total_purchases: 0,
  total_profit: 0,
  low_stock_count: 0,
  order_count: 0,
  product_count: 0
});

const currencySymbol = ref('$');

async function loadStats() {
  try {
    const [statsData, settingsData] = await Promise.all([
      invoke('get_dashboard_stats'),
      invoke('get_settings')
    ]);
    stats.value = statsData;
    if (settingsData && settingsData.currency_symbol) {
      currencySymbol.value = settingsData.currency_symbol;
    }
  } catch (error) {
    console.error("Failed to load dashboard data:", error);
  }
}

onMounted(() => {
  loadStats();
});
</script>

<template>
  <div class="h-full flex flex-col space-y-8">
    <div class="flex justify-between items-center">
      <div>
        <h1 class="text-3xl font-bold text-gray-800">Dashboard</h1>
        <p class="text-gray-500">Business Overview</p>
      </div>
      <button @click="loadStats" class="bg-gray-100 hover:bg-gray-200 text-gray-600 px-4 py-2 rounded-lg transition">
        Refresh
      </button>
    </div>

    <!-- Key Metrics -->
    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">

      <!-- Sales Today -->
      <div class="bg-white p-6 rounded-xl shadow-sm border border-gray-100">
        <div class="text-gray-500 text-sm font-medium uppercase tracking-wider">Sales Today</div>
        <div class="text-3xl font-bold text-gray-800 mt-2">{{ currencySymbol }}{{ stats.sales_today.toFixed(2) }}</div>
      </div>

      <!-- Sales This Month -->
      <div class="bg-white p-6 rounded-xl shadow-sm border border-gray-100">
        <div class="text-gray-500 text-sm font-medium uppercase tracking-wider">Sales This Month</div>
        <div class="text-3xl font-bold text-blue-600 mt-2">{{ currencySymbol }}{{ stats.sales_month.toFixed(2) }}</div>
      </div>

      <!-- Total Profit -->
      <div class="bg-white p-6 rounded-xl shadow-sm border border-gray-100">
        <div class="text-gray-500 text-sm font-medium uppercase tracking-wider">Total Profit</div>
        <div class="text-3xl font-bold text-green-600 mt-2">{{ currencySymbol }}{{ stats.total_profit.toFixed(2) }}
        </div>
      </div>

      <!-- Low Stock -->
      <div class="bg-white p-6 rounded-xl shadow-sm border border-gray-100">
        <div class="text-gray-500 text-sm font-medium uppercase tracking-wider">Low Stock Items</div>
        <div class="text-3xl font-bold text-red-500 mt-2">{{ stats.low_stock_count }}</div>
        <div class="text-xs text-gray-400 mt-1">Products with stock ≤ 5</div>
      </div>
    </div>

    <!-- Secondary Metrics -->
    <div class="grid grid-cols-1 md:grid-cols-4 gap-6">

      <div class="bg-white p-6 rounded-xl shadow-sm border border-gray-100 flex flex-col justify-center text-center">
        <div class="text-2xl font-bold text-gray-700">{{ stats.order_count }}</div>
        <div class="text-gray-400 text-sm">Total Orders</div>
      </div>

      <div class="bg-white p-6 rounded-xl shadow-sm border border-gray-100 flex flex-col justify-center text-center">
        <div class="text-2xl font-bold text-gray-700">{{ stats.product_count }}</div>
        <div class="text-gray-400 text-sm">Active Products</div>
      </div>

      <div class="bg-white p-6 rounded-xl shadow-sm border border-gray-100 flex flex-col justify-center text-center">
        <div class="text-2xl font-bold text-gray-700">{{ currencySymbol }}{{ stats.total_purchases.toFixed(2) }}</div>
        <div class="text-gray-400 text-sm">Total Stock Cost</div>
      </div>

      <div class="bg-white p-6 rounded-xl shadow-sm border border-gray-100 flex flex-col justify-center text-center">
        <div class="text-2xl font-bold text-gray-700">{{ currencySymbol }}{{ stats.total_sales.toFixed(2) }}</div>
        <div class="text-gray-400 text-sm">Lifetime Sales</div>
      </div>
    </div>

    <!-- Quick Actions -->
    <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
      <router-link to="/selling"
        class="bg-blue-600 hover:bg-blue-700 text-white p-6 rounded-xl shadow flex items-center justify-center text-lg font-bold transition transform hover:scale-105">
        Go to Selling
      </router-link>
      <router-link to="/products"
        class="bg-indigo-600 hover:bg-indigo-700 text-white p-6 rounded-xl shadow flex items-center justify-center text-lg font-bold transition transform hover:scale-105">
        Manage Products
      </router-link>
      <router-link to="/buying"
        class="bg-purple-600 hover:bg-purple-700 text-white p-6 rounded-xl shadow flex items-center justify-center text-lg font-bold transition transform hover:scale-105">
        New Buying Entry
      </router-link>
    </div>
  </div>
</template>
