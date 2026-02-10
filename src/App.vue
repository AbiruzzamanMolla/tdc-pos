<script setup>
import { RouterLink, RouterView } from 'vue-router'
</script>

<template>
  <div class="flex h-screen bg-gray-100 font-sans">
    <!-- Sidebar -->
    <aside class="w-64 bg-gray-900 text-white flex flex-col shadow-lg">
      <div class="p-6 border-b border-gray-800">
        <h1 class="text-2xl font-bold tracking-wider text-blue-400">TDC-POS</h1>
      </div>
      <nav class="flex-1 p-4 space-y-2">
        <RouterLink to="/" class="flex items-center px-4 py-3 rounded-lg hover:bg-gray-800 transition-colors"
          active-class="bg-blue-600 hover:bg-blue-700 text-white shadow-md">
          <span class="font-medium">Dashboard</span>
        </RouterLink>
        <RouterLink to="/products" class="flex items-center px-4 py-3 rounded-lg hover:bg-gray-800 transition-colors"
          active-class="bg-blue-600 hover:bg-blue-700 text-white shadow-md">
          <span class="font-medium">Products</span>
        </RouterLink>
        <RouterLink to="/stocks" class="flex items-center px-4 py-3 rounded-lg hover:bg-gray-800 transition-colors"
          active-class="bg-blue-600 hover:bg-blue-700 text-white shadow-md">
          <span class="font-medium">Stocks</span>
        </RouterLink>
        <RouterLink to="/orders" class="flex items-center px-4 py-3 rounded-lg hover:bg-gray-800 transition-colors"
          active-class="bg-blue-600 hover:bg-blue-700 text-white shadow-md">
          <span class="font-medium">Orders</span>
        </RouterLink>
        <RouterLink to="/reports" class="flex items-center px-4 py-3 rounded-lg hover:bg-gray-800 transition-colors"
          active-class="bg-blue-600 hover:bg-blue-700 text-white shadow-md">
          <span class="font-medium">Reports</span>
        </RouterLink>
        <RouterLink to="/backup" class="flex items-center px-4 py-3 rounded-lg hover:bg-gray-800 transition-colors"
          active-class="bg-blue-600 hover:bg-blue-700 text-white shadow-md">
          <span class="font-medium">Backup & Restore</span>
        </RouterLink>
        <RouterLink to="/settings" class="flex items-center px-4 py-3 rounded-lg hover:bg-gray-800 transition-colors"
          active-class="bg-blue-600 hover:bg-blue-700 text-white shadow-md">
          <span class="font-medium">Settings</span>
        </RouterLink>
      </nav>
      <div class="p-4 border-t border-gray-800 text-sm text-gray-400 text-center">
        TDC-POS v0.2.1
      </div>
    </aside>

    <!-- Main Content -->
    <main class="flex-1 overflow-auto p-8 bg-gray-50">
      <RouterView />
    </main>
  </div>
</template>

<style scoped></style>
