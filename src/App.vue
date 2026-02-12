<script setup>
import { RouterLink, RouterView, useRoute, useRouter } from 'vue-router'
import { useAuthStore } from './stores/auth'
import { useThemeStore } from './stores/theme'
import { computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { APP_VERSION } from './version'

const route = useRoute()
const router = useRouter()
const auth = useAuthStore()
const theme = useThemeStore()

const showSidebar = computed(() => route.name !== 'Login')

function logout() {
  auth.logout()
  router.push('/login')
}

onMounted(async () => {
  theme.initTheme();
  try {
    await invoke('check_and_auto_backup');
  } catch (err) {
    console.error("Auto-backup failed", err);
  }
});
</script>

<template>
  <div class="flex h-screen font-sans" style="background: var(--t-main-bg);">
    <!-- Sidebar -->
    <aside v-if="showSidebar" class="w-64 flex flex-col shadow-xl flex-shrink-0 z-20 sidebar-shell">
      <div class="p-6 flex items-center space-x-3 sidebar-border-b">
        <div
          class="w-10 h-10 rounded-lg flex items-center justify-center font-black text-xl text-white shadow-lg sidebar-logo">
          T
        </div>
        <h1 class="text-xl font-black tracking-tight uppercase italic sidebar-text">TDC-POS</h1>
      </div>

      <div class="p-4 sidebar-border-b sidebar-user-panel">
        <div class="flex items-center space-x-3">
          <div
            class="w-8 h-8 rounded-full flex items-center justify-center text-xs font-bold uppercase text-white sidebar-avatar">
            {{ auth.user?.username?.charAt(0) || 'U' }}
          </div>
          <div class="flex-1 min-w-0">
            <p class="text-sm font-bold truncate sidebar-text">{{ auth.user?.username || 'Guest' }}</p>
            <p class="text-[10px] uppercase font-black tracking-widest sidebar-muted">{{ auth.user?.role || 'User' }}
            </p>
          </div>
        </div>
      </div>

      <nav class="flex-1 p-4 space-y-1.5 overflow-y-auto">
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

        <!-- Administration -->
        <div v-if="auth.canManageUsers" class="nav-section">System</div>
        <RouterLink v-if="auth.canManageUsers" to="/users" class="nav-link" active-class="nav-link-active">
          <span class="nav-icon">U</span>
          <span class="font-medium">Users</span>
        </RouterLink>
      </nav>

      <!-- Theme Picker Toggle + Logout -->
      <div class="p-4 sidebar-border-t space-y-2">
        <!-- Theme Selector -->
        <button @click="theme.showPicker = !theme.showPicker"
          class="flex items-center w-full px-4 py-2.5 rounded-xl transition-all text-sm font-bold sidebar-theme-btn">
          <span class="mr-2">🎨</span>
          <span>Theme</span>
          <span class="ml-auto text-lg">{{ theme.currentTheme().emoji }}</span>
        </button>

        <!-- Theme Picker Panel -->
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

    <!-- Main Content -->
    <main class="flex-1 overflow-auto relative" style="background: var(--t-main-bg);">
      <div :class="{ 'p-4 md:p-8': showSidebar }">
        <RouterView />
      </div>
    </main>
  </div>
</template>

<style>
/* ===== GLOBAL SCROLLBAR ===== */
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

/* ===== SIDEBAR SHELL (uses CSS vars from theme store) ===== */
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

/* ===== NAV LINKS ===== */
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

/* ===== THEME BUTTON & PANEL ===== */
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

/* ===== LOGOUT ===== */
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

/* ===== FOOTER ===== */
.sidebar-footer {
  padding: 0.75rem;
  text-align: center;
  font-size: 10px;
  font-weight: 900;
  background: var(--t-sidebar-footer-bg);
  color: var(--t-sidebar-footer-text);
}
</style>
