<script setup>
import { RouterLink, RouterView, useRoute, useRouter } from 'vue-router'
import { useAuthStore } from './stores/auth'
import { computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const route = useRoute()
const router = useRouter()
const auth = useAuthStore()

const showSidebar = computed(() => route.name !== 'Login')

function logout() {
  auth.logout()
  router.push('/login')
}

onMounted(async () => {
  try {
    await invoke('check_and_auto_backup');
  } catch (err) {
    console.error("Auto-backup failed", err);
  }
});
</script>

<template>
  <div class="flex h-screen bg-gray-100 font-sans">
    <!-- Sidebar -->
    <aside v-if="showSidebar" class="w-64 bg-gray-900 text-white flex flex-col shadow-xl flex-shrink-0 z-20">
      <div class="p-6 border-b border-gray-800 flex items-center space-x-3">
        <div
          class="bg-blue-600 text-white w-10 h-10 rounded-lg flex items-center justify-center font-black text-xl shadow-lg shadow-blue-500/20">
          T
        </div>
        <h1 class="text-xl font-black tracking-tight text-white uppercase italic">TDC-POS</h1>
      </div>

      <div class="p-4 border-b border-gray-800 bg-gray-800/20">
        <div class="flex items-center space-x-3">
          <div class="w-8 h-8 rounded-full bg-blue-500 flex items-center justify-center text-xs font-bold uppercase">
            {{ auth.user?.username?.charAt(0) || 'U' }}
          </div>
          <div class="flex-1 min-w-0">
            <p class="text-sm font-bold truncate">{{ auth.user?.username || 'Guest' }}</p>
            <p class="text-[10px] text-gray-400 uppercase font-black tracking-widest">{{ auth.user?.role || 'User' }}
            </p>
          </div>
        </div>
      </div>

      <nav class="flex-1 p-4 space-y-1.5 overflow-y-auto">
        <RouterLink to="/" class="flex items-center px-4 py-3 rounded-xl hover:bg-gray-800 transition-all text-sm group"
          active-class="bg-blue-600 hover:bg-blue-700 text-white shadow-xl shadow-blue-600/20">
          <span class="font-bold opacity-70 group-[.active-class]:opacity-100 italic mr-2">/</span>
          <span class="font-medium">Dashboard</span>
        </RouterLink>
        <RouterLink to="/products"
          class="flex items-center px-4 py-3 rounded-xl hover:bg-gray-800 transition-all text-sm group"
          active-class="bg-blue-600 hover:bg-blue-700 text-white shadow-xl shadow-blue-600/20">
          <span class="font-bold opacity-70 group-[.active-class]:opacity-100 italic mr-2">P</span>
          <span class="font-medium">Products</span>
        </RouterLink>
        <RouterLink to="/stocks"
          class="flex items-center px-4 py-3 rounded-xl hover:bg-gray-800 transition-all text-sm group"
          active-class="bg-blue-600 hover:bg-blue-700 text-white shadow-xl shadow-blue-600/20">
          <span class="font-bold opacity-70 group-[.active-class]:opacity-100 italic mr-2">S</span>
          <span class="font-medium text-blue-400 group-[.active-class]:text-white">Stock List</span>
        </RouterLink>
        <div class="px-4 pt-4 pb-1 text-[10px] font-black text-gray-500 uppercase tracking-[0.2em]">Transaction</div>
        <RouterLink to="/buying"
          class="flex items-center px-4 py-3 rounded-xl hover:bg-gray-800 transition-all text-sm group"
          active-class="bg-blue-600 hover:bg-blue-700 text-white shadow-xl shadow-blue-600/20">
          <span class="font-bold opacity-70 group-[.active-class]:opacity-100 italic mr-2">B</span>
          <span class="font-medium">Buying</span>
        </RouterLink>
        <RouterLink to="/selling"
          class="flex items-center px-4 py-3 rounded-xl hover:bg-gray-800 transition-all text-sm group"
          active-class="bg-blue-600 hover:bg-blue-700 text-white shadow-xl shadow-blue-600/20">
          <span class="font-bold opacity-70 group-[.active-class]:opacity-100 italic mr-2">S</span>
          <span class="font-medium">Selling</span>
        </RouterLink>
        <div class="px-4 pt-4 pb-1 text-[10px] font-black text-gray-500 uppercase tracking-[0.2em]">Utilities</div>
        <RouterLink to="/reports"
          class="flex items-center px-4 py-3 rounded-xl hover:bg-gray-800 transition-all text-sm group"
          active-class="bg-blue-600 hover:bg-blue-700 text-white shadow-xl shadow-blue-600/20">
          <span class="font-bold opacity-70 group-[.active-class]:opacity-100 italic mr-2">R</span>
          <span class="font-medium">Reports</span>
        </RouterLink>
        <RouterLink v-if="auth.isAdmin" to="/backup"
          class="flex items-center px-4 py-3 rounded-xl hover:bg-gray-800 transition-all text-sm group"
          active-class="bg-blue-600 hover:bg-blue-700 text-white shadow-xl shadow-blue-600/20">
          <span class="font-bold opacity-70 group-[.active-class]:opacity-100 italic mr-2">B</span>
          <span class="font-medium">Backup</span>
        </RouterLink>
        <RouterLink v-if="auth.isAdmin" to="/settings"
          class="flex items-center px-4 py-3 rounded-xl hover:bg-gray-800 transition-all text-sm group"
          active-class="bg-blue-600 hover:bg-blue-700 text-white shadow-xl shadow-blue-600/20">
          <span class="font-bold opacity-70 group-[.active-class]:opacity-100 italic mr-2">S</span>
          <span class="font-medium">Settings</span>
        </RouterLink>
      </nav>

      <div class="p-4 border-t border-gray-800">
        <button @click="logout"
          class="flex items-center w-full px-4 py-3 rounded-xl bg-gray-800 hover:bg-red-600/20 text-gray-400 hover:text-red-400 transition-all text-sm font-bold">
          <span class="mr-2">Logout</span>
        </button>
      </div>

      <div class="p-3 bg-black/20 text-[10px] text-gray-500 text-center font-black">
        TDC-POS v0.4.0
      </div>
    </aside>

    <!-- Main Content -->
    <main class="flex-1 overflow-auto bg-gray-50 relative">
      <div :class="{ 'p-4 md:p-8': showSidebar }">
        <RouterView />
      </div>
    </main>
  </div>
</template>

<style>
/* Global styles */
::-webkit-scrollbar {
  width: 6px;
  height: 6px;
}

::-webkit-scrollbar-track {
  background: transparent;
}

::-webkit-scrollbar-thumb {
  background: #cbd5e1;
  border-radius: 10px;
}

::-webkit-scrollbar-thumb:hover {
  background: #94a3b8;
}
</style>
