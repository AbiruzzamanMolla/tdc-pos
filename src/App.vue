<script setup>
import { RouterLink, RouterView } from 'vue-router'
</script>

<template>
  <div class="flex h-screen bg-gray-100 font-sans">
    <!-- Sidebar -->
    <aside class="w-56 bg-gray-900 text-white flex flex-col shadow-lg flex-shrink-0">
      <div class="p-5 border-b border-gray-800">
        <h1 class="text-xl font-bold tracking-wider text-blue-400">TDC-POS</h1>
      </div>
      <nav class="flex-1 p-3 space-y-1">
        <RouterLink to="/" class="flex items-center px-4 py-3 rounded-lg hover:bg-gray-800 transition-colors text-sm"
          active-class="bg-blue-600 hover:bg-blue-700 text-white shadow-md">
          <span class="font-medium">Dashboard</span>
        </RouterLink>
        <RouterLink to="/products"
          class="flex items-center px-4 py-3 rounded-lg hover:bg-gray-800 transition-colors text-sm"
          active-class="bg-blue-600 hover:bg-blue-700 text-white shadow-md">
          <span class="font-medium">Products</span>
        </RouterLink>
        <RouterLink to="/buying"
          class="flex items-center px-4 py-3 rounded-lg hover:bg-gray-800 transition-colors text-sm"
          active-class="bg-blue-600 hover:bg-blue-700 text-white shadow-md">
          <span class="font-medium">Buying</span>
        </RouterLink>
        <RouterLink to="/selling"
          class="flex items-center px-4 py-3 rounded-lg hover:bg-gray-800 transition-colors text-sm"
          active-class="bg-blue-600 hover:bg-blue-700 text-white shadow-md">
          <span class="font-medium">Selling</span>
        </RouterLink>
        <RouterLink to="/reports"
          class="flex items-center px-4 py-3 rounded-lg hover:bg-gray-800 transition-colors text-sm"
          active-class="bg-blue-600 hover:bg-blue-700 text-white shadow-md">
          <span class="font-medium">Reports</span>
        </RouterLink>
        <RouterLink to="/backup"
          class="flex items-center px-4 py-3 rounded-lg hover:bg-gray-800 transition-colors text-sm"
          active-class="bg-blue-600 hover:bg-blue-700 text-white shadow-md">
          <span class="font-medium">Backup & Restore</span>
        </RouterLink>
        <RouterLink to="/settings"
          class="flex items-center px-4 py-3 rounded-lg hover:bg-gray-800 transition-colors text-sm"
          active-class="bg-blue-600 hover:bg-blue-700 text-white shadow-md">
          <span class="font-medium">Settings</span>
        </RouterLink>
      </nav>
      <div class="p-3 border-t border-gray-800 text-xs text-gray-400 text-center">
        TDC-POS v0.3.0
      </div>
    </aside>

    <!-- Main Content -->
    <main class="flex-1 overflow-auto p-4 md:p-6 bg-gray-50">
      <RouterView />
    </main>
  </div>
</template>

<style scoped></style>
