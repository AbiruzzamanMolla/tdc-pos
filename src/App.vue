<script setup>
import { RouterLink, RouterView, useRoute, useRouter } from 'vue-router'
import { useAuthStore } from './stores/auth'
import { useThemeStore } from './stores/theme'
import { computed, onMounted, ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { APP_VERSION } from './version'
import FloatingCalculator from './components/FloatingCalculator.vue'

const route = useRoute()
const router = useRouter()
const auth = useAuthStore()
const theme = useThemeStore()

// Navigation & Sidebar Logic
const isSidebarOpen = ref(window.innerWidth >= 1024)
const isMobile = ref(window.innerWidth < 1024)

// Handle window resize to adjust responsiveness
onMounted(async () => {
  window.addEventListener('resize', handleResize)

  theme.initTheme();
  try {
    await invoke('check_and_auto_backup');
  } catch (err) {
    console.error("Auto-backup failed", err);
  }
});

function handleResize() {
  const wasMobile = isMobile.value
  isMobile.value = window.innerWidth < 1024

  // If switching from desktop to mobile, auto-close sidebar
  if (!wasMobile && isMobile.value) {
    isSidebarOpen.value = false
  }
  // If switching from mobile to desktop, auto-open sidebar (optional, usually UX friendly)
  if (wasMobile && !isMobile.value) {
    isSidebarOpen.value = true
  }
}

// Close sidebar on route change if on mobile
watch(route, () => {
  if (isMobile.value) {
    isSidebarOpen.value = false
  }
})

function logout() {
  auth.logout()
  router.push('/login')
}
</script>

<template>
  <div class="flex h-screen font-sans overflow-hidden" style="background: var(--t-main-bg);">

    <!-- Mobile Overlay -->
    <div v-if="isMobile && isSidebarOpen" @click="isSidebarOpen = false"
      class="fixed inset-0 bg-black/50 z-30 transition-opacity backdrop-blur-sm">
    </div>

    <!-- Sidebar -->
    <!-- 
      - Mobile: Fixed position, slide-in transformation.
      - Desktop: Relative position to push content, v-show/hidden toggling.
    -->
    <aside v-if="route.name !== 'Login'" :class="[
      'flex flex-col shadow-xl z-40 sidebar-shell transition-all duration-300 ease-in-out',
      isMobile ? 'fixed inset-y-0 left-0 w-72' : 'relative',
      isMobile && !isSidebarOpen ? '-translate-x-full' : 'translate-x-0',
      !isMobile && !isSidebarOpen ? 'w-0 overflow-hidden opacity-0' : 'w-64 opacity-100'
    ]">
      <!-- Header -->
      <div class="p-6 flex items-center justify-between sidebar-border-b">
        <div class="flex items-center space-x-3">
          <div
            class="w-10 h-10 rounded-lg flex items-center justify-center font-black text-xl text-white shadow-lg sidebar-logo flex-shrink-0">
            T
          </div>
          <h1 class="text-xl font-black tracking-tight uppercase italic sidebar-text whitespace-nowrap">TDC-POS</h1>
        </div>
        <!-- Mobile Close Button -->
        <button v-if="isMobile" @click="isSidebarOpen = false" class="text-gray-400 hover:text-white">
          <svg class="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <!-- User Panel -->
      <div class="p-4 sidebar-border-b sidebar-user-panel whitespace-nowrap overflow-hidden">
        <div class="flex items-center space-x-3">
          <div
            class="w-8 h-8 rounded-full flex items-center justify-center text-xs font-bold uppercase text-white sidebar-avatar flex-shrink-0">
            {{ auth.user?.username?.charAt(0) || 'U' }}
          </div>
          <div class="flex-1 min-w-0">
            <p class="text-sm font-bold truncate sidebar-text">{{ auth.user?.username || 'Guest' }}</p>
            <p class="text-[10px] uppercase font-black tracking-widest sidebar-muted">{{ auth.user?.role || 'User' }}
            </p>
          </div>
        </div>
      </div>

      <!-- Navigation -->
      <nav class="flex-1 p-4 space-y-1.5 overflow-y-auto overflow-x-hidden">
        <RouterLink to="/" class="nav-link" active-class="nav-link-active">
          <span class="nav-icon">/</span>
          <span class="font-medium">Dashboard</span>
        </RouterLink>

        <!-- Inventory -->
        <div v-if="auth.canManageProducts || auth.canViewStock" class="nav-section">Inventory</div>
        <RouterLink v-if="auth.canManageProducts" to="/products" class="nav-link" active-class="nav-link-active">
          <span class="nav-icon">P</span>
          <span class="font-medium">Products</span>
        </RouterLink>
        <RouterLink v-if="auth.canViewStock" to="/stocks" class="nav-link" active-class="nav-link-active">
          <span class="nav-icon">S</span>
          <span class="font-medium">Stock List</span>
        </RouterLink>

        <!-- Transaction -->
        <div v-if="auth.canBuy || auth.canSell" class="nav-section">Transaction</div>
        <RouterLink v-if="auth.canBuy" to="/buying" class="nav-link" active-class="nav-link-active">
          <span class="nav-icon">B</span>
          <span class="font-medium">Buying</span>
        </RouterLink>
        <RouterLink v-if="auth.canSell" to="/selling" class="nav-link" active-class="nav-link-active">
          <span class="nav-icon">S</span>
          <span class="font-medium">Selling</span>
        </RouterLink>

        <!-- Utilities -->
        <div v-if="auth.canViewReports || auth.canManageBackup || auth.canManageSettings" class="nav-section">Utilities
        </div>
        <RouterLink v-if="auth.canViewReports" to="/reports" class="nav-link" active-class="nav-link-active">
          <span class="nav-icon">R</span>
          <span class="font-medium">Reports</span>
        </RouterLink>
        <RouterLink v-if="auth.canManageBackup" to="/backup" class="nav-link" active-class="nav-link-active">
          <span class="nav-icon">B</span>
          <span class="font-medium">Backup</span>
        </RouterLink>
        <RouterLink v-if="auth.canManageSettings" to="/settings" class="nav-link" active-class="nav-link-active">
          <span class="nav-icon">S</span>
          <span class="font-medium">Settings</span>
        </RouterLink>

        <div class="nav-section">Assistant</div>
        <RouterLink to="/chat" class="nav-link" active-class="nav-link-active">
          <span class="nav-icon">🤖</span>
          <span class="font-medium">AI Chat</span>
        </RouterLink>

        <!-- Administration -->
        <div v-if="auth.canManageUsers || auth.canViewActivityLog" class="nav-section">System</div>
        <RouterLink v-if="auth.canManageUsers" to="/users" class="nav-link" active-class="nav-link-active">
          <span class="nav-icon">U</span>
          <span class="font-medium">Users</span>
        </RouterLink>
        <RouterLink v-if="auth.canViewActivityLog" to="/activity-log" class="nav-link" active-class="nav-link-active">
          <span class="nav-icon">A</span>
          <span class="font-medium">Activity Log</span>
        </RouterLink>
      </nav>

      <!-- Footer Actions -->
      <div class="p-4 sidebar-border-t space-y-2 whitespace-nowrap overflow-hidden">
        <button @click="theme.showPicker = !theme.showPicker"
          class="flex items-center w-full px-4 py-2.5 rounded-xl transition-all text-sm font-bold sidebar-theme-btn">
          <span class="mr-2">🎨</span>
          <span>Theme</span>
          <span class="ml-auto text-lg">{{ theme.currentTheme().emoji }}</span>
        </button>

        <div v-if="theme.showPicker"
          class="grid grid-cols-3 gap-2 p-2 rounded-xl sidebar-theme-panel animate-in fade-in duration-200">
          <button v-for="t in theme.allThemes" :key="t.id" @click="theme.setTheme(t.id)"
            class="flex flex-col items-center p-2 rounded-lg transition-all text-[9px] font-black uppercase tracking-widest"
            :class="theme.currentThemeId === t.id ? 'ring-2 ring-offset-1 sidebar-theme-active' : 'opacity-60 hover:opacity-100'"
            :style="`--ring-color: ${t.accent}; ring-color: ${t.accent};`">
            <span class="text-lg mb-0.5">{{ t.emoji }}</span>
            <span class="sidebar-muted">{{ t.name }}</span>
          </button>
        </div>

        <button @click="logout" class="sidebar-logout-btn">
          <span class="mr-2">Logout</span>
        </button>
      </div>

      <div class="sidebar-footer">
        TDC-POS {{ APP_VERSION }}
      </div>
    </aside>

    <!-- Main Content Area -->
    <main class="flex-1 overflow-auto relative w-full h-full flex flex-col" style="background: var(--t-main-bg);">

      <!-- Top Toggle Button (Visible on Mobile AND Desktop when closed) -->
      <div v-if="route.name !== 'Login'" class="sticky top-0 z-10 w-full flex items-center p-4">
        <button @click="isSidebarOpen = !isSidebarOpen"
          class="p-2 rounded-xl shadow-sm border border-gray-200 bg-white/80 backdrop-blur-md text-gray-700 hover:bg-white hover:shadow-md transition-all active:scale-95"
          style="background-color: var(--t-main-card-bg); border-color: var(--t-main-card-border); color: var(--t-main-text);">
          <svg v-if="!isSidebarOpen" class="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16" />
          </svg>
          <svg v-else class="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h7" />
          </svg>
        </button>
        <div class="ml-4 font-bold text-lg md:text-xl lg:hidden text-gray-800" style="color: var(--t-main-text);">
          {{ route.name }}
        </div>
      </div>

      <div class="flex-1 px-4 pb-4 md:px-8 md:pb-8 w-full max-w-[1920px] mx-auto">
        <RouterView />
      </div>
    </main>
    <FloatingCalculator />
  </div>
</template>

<style>
/* ... existing styles ... */
/* Include the previous styles here or just import `style.css` if it was external, but `App.vue` had inline styles. 
   I need to preserve the styles I viewed earlier. */

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

.sidebar-shell {
  background: var(--t-sidebar-bg);
  color: var(--t-sidebar-text);
}

.sidebar-border-b {
  border-bottom: 1px solid var(--t-sidebar-border);
}

.sidebar-border-t {
  border-top: 1px solid var(--t-sidebar-border);
}

.sidebar-text {
  color: var(--t-sidebar-text);
}

.sidebar-muted {
  color: var(--t-sidebar-muted);
}

.sidebar-user-panel {
  background: var(--t-sidebar-user-bg);
}

.sidebar-logo {
  background: var(--t-accent);
  box-shadow: 0 4px 14px var(--t-accent-shadow);
}

.sidebar-avatar {
  background: var(--t-accent);
}

.nav-link {
  display: flex;
  align-items: center;
  padding: 0.75rem 1rem;
  border-radius: 0.75rem;
  transition: all 0.15s;
  font-size: 0.875rem;
  color: var(--t-sidebar-text);
}

.nav-link:hover {
  background: var(--t-sidebar-hover);
}

.nav-icon {
  font-weight: 900;
  font-style: italic;
  margin-right: 0.5rem;
  opacity: 0.7;
}

.nav-link-active {
  background: var(--t-accent) !important;
  color: #ffffff !important;
  box-shadow: 0 4px 14px var(--t-accent-shadow);
}

.nav-link-active .nav-icon {
  opacity: 1;
}

.nav-section {
  padding: 1rem 1rem 0.25rem;
  font-size: 10px;
  font-weight: 900;
  text-transform: uppercase;
  letter-spacing: 0.2em;
  color: var(--t-sidebar-section-text);
}

.sidebar-theme-btn {
  background: var(--t-sidebar-hover);
  color: var(--t-sidebar-muted);
}

.sidebar-theme-btn:hover {
  color: var(--t-sidebar-text);
}

.sidebar-theme-panel {
  background: var(--t-sidebar-hover);
}

.sidebar-theme-active {
  outline: 2px solid var(--t-accent);
  outline-offset: 1px;
  background: rgba(255, 255, 255, 0.05);
}

.sidebar-logout-btn {
  display: flex;
  align-items: center;
  width: 100%;
  padding: 0.75rem 1rem;
  border-radius: 0.75rem;
  transition: all 0.15s;
  font-size: 0.875rem;
  font-weight: 700;
  background: var(--t-logout-bg);
  color: var(--t-logout-text);
}

.sidebar-logout-btn:hover {
  background: var(--t-logout-hover-bg);
  color: var(--t-logout-hover-text);
}

.sidebar-footer {
  padding: 0.75rem;
  text-align: center;
  font-size: 10px;
  font-weight: 900;
  background: var(--t-sidebar-footer-bg);
  color: var(--t-sidebar-footer-text);
}
</style>
