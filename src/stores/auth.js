import { defineStore } from 'pinia';
import { ref, computed } from 'vue';

export const useAuthStore = defineStore('auth', () => {
  const user = ref(JSON.parse(localStorage.getItem('user')) || null);

  const isAuthenticated = computed(() => !!user.value);
  const role = computed(() => user.value?.role || '');
  
  const isSuperAdmin = computed(() => role.value === 'super_admin');
  const isAdmin = computed(() => ['super_admin', 'admin'].includes(role.value));
  const isManager = computed(() => ['super_admin', 'admin', 'manager'].includes(role.value));
  
  // Specific permissions
  const canManageUsers = computed(() => ['super_admin', 'admin'].includes(role.value));
  const canManageSettings = computed(() => ['super_admin', 'admin'].includes(role.value));
  const canManageBackup = computed(() => ['super_admin', 'admin'].includes(role.value));
  
  const canBuy = computed(() => ['super_admin', 'admin', 'manager', 'buy_manager'].includes(role.value));
  const canSell = computed(() => ['super_admin', 'admin', 'manager', 'sell_manager'].includes(role.value));
  const canViewReports = computed(() => ['super_admin', 'admin', 'manager', 'report_checker', 'inspector'].includes(role.value));
  const canManageProducts = computed(() => ['super_admin', 'admin', 'manager', 'buy_manager'].includes(role.value));
  const canViewStock = computed(() => !!role.value); // Everyone logged in
  const canViewActivityLog = computed(() => ['super_admin', 'admin', 'manager', 'inspector'].includes(role.value));

  function setUser(userData) {
    user.value = userData;
    if (userData) {
      localStorage.setItem('user', JSON.stringify(userData));
    } else {
      localStorage.removeItem('user');
    }
  }

  function logout() {
    setUser(null);
  }

  return { 
    user, 
    isAuthenticated, 
    role,
    isSuperAdmin,
    isAdmin,
    isManager,
    canManageUsers,
    canManageSettings,
    canManageBackup,
    canBuy,
    canSell,
    canViewReports,
    canManageProducts,
    canViewStock,
    canViewActivityLog,
    setUser, 
    logout 
  };
});
