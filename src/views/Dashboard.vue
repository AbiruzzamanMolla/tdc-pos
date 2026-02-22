<script setup>
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { useAuthStore } from '../stores/auth';
import { useI18nStore } from '../stores/i18n';

const auth = useAuthStore();
const i18n = useI18nStore();

const showFraudChecker = ref(false);
const fraudPhone = ref('');
const fraudResult = ref(null);
const isLoadingFraud = ref(false);
const fraudError = ref(null);

async function checkFraud() {
  if (!fraudPhone.value.trim()) return;
  
  isLoadingFraud.value = true;
  fraudError.value = null;
  fraudResult.value = null;
  
  try {
    const formData = new FormData();
    formData.append('phone', fraudPhone.value.trim());
    
    const response = await fetch('https://audiobookbangla.com/api/v2/check-fraud', {
      method: 'POST',
      body: formData
    });
    
    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }
    
    const data = await response.json();
    fraudResult.value = data;
  } catch (error) {
    fraudError.value = error.message || 'An error occurred while checking fraud';
    console.error("Fraud check failed:", error);
  } finally {
    isLoadingFraud.value = false;
  }
}

const stats = ref({
  sales_today: 0, sales_month: 0, sales_year: 0, total_sales: 0,
  purchases_today: 0, purchases_month: 0, purchases_year: 0, total_purchases: 0,
  profit_today: 0, profit_month: 0, profit_year: 0, total_profit: 0,
  inventory_value: 0, low_stock_count: 0, order_count: 0, product_count: 0
});

const currencySymbol = ref('৳');
const activeTab = ref('sales'); // 'sales', 'purchases', 'profit'

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
  <div class="h-full flex flex-col space-y-8 animate-in fade-in duration-500">
    <!-- Header -->
    <div class="flex flex-col sm:flex-row sm:justify-between sm:items-end gap-4">
      <div>
        <h1 class="text-4xl font-black text-gray-900 tracking-tight">{{ i18n.t('executive_dashboard') }}</h1>
        <p class="text-gray-500 font-medium">{{ i18n.t('precision_analytics') }}</p>
      </div>
      <div class="flex gap-3 self-end sm:self-auto">
        <!-- Fraud Checker Button -->
        <button @click="showFraudChecker = true"
          class="bg-red-50 text-red-600 px-4 py-2.5 rounded-xl font-black text-xs uppercase tracking-widest hover:bg-red-100 transition-all active:scale-95 border border-red-100 shadow-sm flex items-center gap-2">
          <span>🛡️</span> Fraud Checker
        </button>
        <!-- Language Switcher -->
        <button @click="i18n.toggleLocale"
          class="bg-blue-50 text-blue-600 px-4 py-2.5 rounded-xl font-black text-xs uppercase tracking-widest hover:bg-blue-100 transition-all active:scale-95 border border-blue-100 shadow-sm">
          {{ i18n.t('locale_name') }}
        </button>
        <button @click="loadStats"
          class="bg-white border border-gray-200 hover:border-blue-500 hover:text-blue-600 text-gray-600 px-6 py-2.5 rounded-xl font-bold transition-all shadow-sm active:scale-95 text-sm">
          {{ i18n.t('refresh_data') }}
        </button>
      </div>
    </div>

    <!-- Critical KPIs (Universal) -->
    <div v-if="auth.canViewReports" class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
      <div
        class="bg-gradient-to-br from-indigo-600 to-blue-700 p-6 rounded-3xl shadow-xl shadow-blue-100 text-white transform transition hover:scale-[1.02]">
        <div class="text-blue-100 text-xs font-black uppercase tracking-widest opacity-80">{{
          i18n.t('inventory_valuation') }}</div>
        <div class="text-3xl font-black mt-2">{{ currencySymbol }}{{ stats.inventory_value.toLocaleString(undefined,
          { minimumFractionDigits: 2 }) }}</div>
        <div class="mt-4 flex items-center gap-2">
          <span class="px-2 py-0.5 bg-white/20 rounded-lg text-[10px] font-bold uppercase tracking-tighter">{{
            i18n.t('current_assets') }}</span>
        </div>
      </div>

      <div
        class="bg-gradient-to-br from-emerald-600 to-teal-700 p-6 rounded-3xl shadow-xl shadow-teal-100 text-white transform transition hover:scale-[1.02]">
        <div class="text-teal-100 text-xs font-black uppercase tracking-widest opacity-80">{{ i18n.t('lifetime_profit')
        }}</div>
        <div class="text-3xl font-black mt-2">{{ currencySymbol }}{{ stats.total_profit.toLocaleString(undefined,
          { minimumFractionDigits: 2 }) }}</div>
        <div class="mt-4 flex items-center gap-2">
          <span class="px-2 py-0.5 bg-white/20 rounded-lg text-[10px] font-bold uppercase tracking-tighter">{{
            i18n.t('net_accrued') }}</span>
        </div>
      </div>

      <div
        class="bg-white p-6 rounded-3xl shadow-sm border border-gray-100 flex items-center justify-between group cursor-help">
        <div>
          <div class="text-gray-400 text-xs font-black uppercase tracking-widest">{{ i18n.t('active_orders') }}</div>
          <div class="text-3xl font-black text-gray-900 mt-1">{{ stats.order_count }}</div>
        </div>
        <div
          class="w-14 h-14 rounded-2xl bg-gray-50 flex items-center justify-center transition-colors group-hover:bg-blue-50">
          <span class="text-2xl">📦</span>
        </div>
      </div>

      <div
        class="bg-white p-6 rounded-3xl shadow-sm border border-gray-100 flex items-center justify-between group relative overflow-hidden">
        <div v-if="stats.low_stock_count > 0"
          class="absolute top-0 right-0 w-16 h-16 bg-red-500 rotate-45 translate-x-10 -translate-y-10"></div>
        <div>
          <div class="text-gray-400 text-xs font-black uppercase tracking-widest">{{ i18n.t('low_stock_items') }}</div>
          <div class="text-3xl font-black text-gray-900 mt-1" :class="{ 'text-red-500': stats.low_stock_count > 0 }">{{
            stats.low_stock_count }}</div>
        </div>
        <div
          class="w-14 h-14 rounded-2xl bg-gray-50 flex items-center justify-center transition-colors group-hover:bg-red-50">
          <span class="text-2xl" :class="{ 'animate-bounce': stats.low_stock_count > 0 }">⚠️</span>
        </div>
      </div>
    </div>

    <!-- Temporal Insights Tabs -->
    <div v-if="auth.canViewReports" class="bg-white rounded-[2rem] shadow-sm border border-gray-100 p-8">
      <div class="flex flex-col md:flex-row md:items-center justify-between gap-6 mb-10">
        <div class="flex bg-gray-100 p-1.5 rounded-2xl w-fit">
          <button v-for="tab in ['sales', 'purchases', 'profit']" :key="tab" @click="activeTab = tab"
            class="px-8 py-2.5 rounded-xl font-black text-xs uppercase tracking-widest transition-all"
            :class="activeTab === tab ? 'bg-white text-blue-600 shadow-sm' : 'text-gray-400 hover:text-gray-600'">
            {{ i18n.t(tab) }}
          </button>
        </div>
        <div class="text-right">
          <span class="text-gray-400 text-xs font-bold uppercase tracking-widest block">{{
            i18n.t('operational_performance') }}</span>
          <span class="text-gray-900 font-black text-sm uppercase">{{ i18n.t('temporal_visualization') }}</span>
        </div>
      </div>

      <!-- Sales Context -->
      <div v-if="activeTab === 'sales'"
        class="grid grid-cols-1 md:grid-cols-3 gap-8 animate-in slide-in-from-bottom-4 duration-500">
        <div class="p-8 rounded-3xl bg-blue-50/50 border border-blue-100 relative overflow-hidden group text-left">
          <div
            class="absolute -right-4 -bottom-4 text-8xl opacity-[0.03] font-black group-hover:scale-110 transition-transform">
            D</div>
          <h4 class="text-blue-600 text-xs font-black uppercase tracking-widest mb-2">{{ i18n.t('today') }}</h4>
          <div class="text-4xl font-black text-blue-900">{{ currencySymbol }}{{ stats.sales_today.toLocaleString() }}
          </div>
          <progress class="w-full h-1.5 mt-6 accent-blue-600 opacity-20" :value="stats.sales_today"
            :max="stats.sales_month / 10"></progress>
        </div>
        <div class="p-8 rounded-3xl bg-indigo-50/50 border border-indigo-100 relative overflow-hidden group text-left">
          <div
            class="absolute -right-4 -bottom-4 text-8xl opacity-[0.03] font-black group-hover:scale-110 transition-transform">
            M</div>
          <h4 class="text-indigo-600 text-xs font-black uppercase tracking-widest mb-2">{{ i18n.t('this_month') }}</h4>
          <div class="text-4xl font-black text-indigo-900">{{ currencySymbol }}{{ stats.sales_month.toLocaleString() }}
          </div>
          <progress class="w-full h-1.5 mt-6 accent-indigo-600 opacity-20" :value="stats.sales_month"
            :max="stats.sales_year / 12"></progress>
        </div>
        <div class="p-8 rounded-3xl bg-violet-50/50 border border-violet-100 relative overflow-hidden group text-left">
          <div
            class="absolute -right-4 -bottom-4 text-8xl opacity-[0.03] font-black group-hover:scale-110 transition-transform">
            Y</div>
          <h4 class="text-violet-600 text-xs font-black uppercase tracking-widest mb-2">{{ i18n.t('this_year') }}</h4>
          <div class="text-4xl font-black text-violet-900">{{ currencySymbol }}{{ stats.sales_year.toLocaleString() }}
          </div>
          <div class="text-[10px] uppercase font-bold text-violet-400 mt-6 tracking-widest">Projected annual growth
            optimization</div>
        </div>
      </div>

      <!-- Purchases Context -->
      <div v-if="activeTab === 'purchases'"
        class="grid grid-cols-1 md:grid-cols-3 gap-8 animate-in slide-in-from-bottom-4 duration-500 text-left">
        <div class="p-8 rounded-3xl bg-amber-50/50 border border-amber-100 relative overflow-hidden group">
          <div class="absolute -right-4 -bottom-4 text-6xl opacity-[0.03] font-black">STOCK</div>
          <h4 class="text-amber-600 text-xs font-black uppercase tracking-widest mb-2">{{ i18n.t('procurement_today') }}
          </h4>
          <div class="text-4xl font-black text-amber-900">{{ currencySymbol }}{{ stats.purchases_today.toLocaleString()
          }}</div>
        </div>
        <div class="p-8 rounded-3xl bg-orange-50/50 border border-orange-100 relative overflow-hidden group">
          <div class="absolute -right-4 -bottom-4 text-6xl opacity-[0.03] font-black">FLOW</div>
          <h4 class="text-orange-600 text-xs font-black uppercase tracking-widest mb-2">{{ i18n.t('this_month') }}</h4>
          <div class="text-4xl font-black text-orange-900">{{ currencySymbol }}{{ stats.purchases_month.toLocaleString()
          }}</div>
        </div>
        <div class="p-8 rounded-3xl bg-rose-50/50 border border-rose-100 relative overflow-hidden group">
          <div class="absolute -right-4 -bottom-4 text-6xl opacity-[0.03] font-black">CAPEX</div>
          <h4 class="text-rose-600 text-xs font-black uppercase tracking-widest mb-2">{{ i18n.t('this_year') }}</h4>
          <div class="text-4xl font-black text-rose-900">{{ currencySymbol }}{{ stats.purchases_year.toLocaleString() }}
          </div>
        </div>
      </div>

      <!-- Profit Context -->
      <div v-if="activeTab === 'profit'"
        class="grid grid-cols-1 md:grid-cols-3 gap-8 animate-in slide-in-from-bottom-4 duration-500 text-left">
        <div
          class="p-8 rounded-3xl bg-emerald-50/50 border border-emerald-100 relative overflow-hidden group text-left">
          <div class="absolute -right-4 -bottom-4 text-6xl opacity-[0.05] font-black">NET</div>
          <h4 class="text-emerald-600 text-xs font-black uppercase tracking-widest mb-2">{{ i18n.t('today') }} ({{
            i18n.t('profit') }})</h4>
          <div class="text-4xl font-black text-emerald-900">{{ currencySymbol }}{{ stats.profit_today.toLocaleString()
          }}</div>
          <div class="text-[10px] uppercase font-bold text-emerald-500 mt-6 tracking-widest">{{
            i18n.t('real_time_analysis') }}</div>
        </div>
        <div class="p-8 rounded-3xl bg-teal-50/50 border border-teal-100 relative overflow-hidden group text-left">
          <div class="absolute -right-4 -bottom-4 text-6xl opacity-[0.05] font-black">GROSS</div>
          <h4 class="text-teal-600 text-xs font-black uppercase tracking-widest mb-2">{{ i18n.t('monthly_margin') }}
          </h4>
          <div class="text-4xl font-black text-teal-900">{{ currencySymbol }}{{ stats.profit_month.toLocaleString() }}
          </div>
          <div class="text-[10px] uppercase font-bold text-teal-500 mt-6 tracking-widest">{{ i18n.t('snapshot_accuracy')
          }}</div>
        </div>
        <div class="p-8 rounded-3xl bg-cyan-50/50 border border-cyan-100 relative overflow-hidden group text-left">
          <div class="absolute -right-4 -bottom-4 text-6xl opacity-[0.05] font-black">YEAR</div>
          <h4 class="text-cyan-600 text-xs font-black uppercase tracking-widest mb-2">{{ i18n.t('annual_profit') }}</h4>
          <div class="text-4xl font-black text-cyan-900">{{ currencySymbol }}{{ stats.profit_year.toLocaleString() }}
          </div>
          <div class="text-[10px] uppercase font-bold text-cyan-500 mt-6 tracking-widest">{{
            i18n.t('fiscal_performance') }}</div>
        </div>
      </div>
    </div>

    <!-- System Actions & Stats -->
    <div class="grid grid-cols-1 lg:grid-cols-3 gap-8">
      <div class="lg:col-span-2 bg-white rounded-3xl border border-gray-100 p-8 shadow-sm text-left">
        <h3 class="text-lg font-black text-gray-900 uppercase tracking-widest mb-6 border-b border-gray-50 pb-4">{{
          i18n.t('operational_shortcuts') }}</h3>
        <div class="grid grid-cols-2 sm:grid-cols-4 gap-4">
          <router-link v-if="auth.canSell" to="/selling"
            class="flex flex-col items-center p-6 bg-blue-50 rounded-2xl hover:bg-blue-100 transition-colors group">
            <span class="text-2xl mb-2 group-hover:scale-110 transition-transform">🛒</span>
            <span class="text-[10px] font-black uppercase tracking-widest text-blue-700">{{ i18n.t('checkout') }}</span>
          </router-link>
          <router-link v-if="auth.canManageProducts" to="/products"
            class="flex flex-col items-center p-6 bg-indigo-50 rounded-2xl hover:bg-indigo-100 transition-colors group">
            <span class="text-2xl mb-2 group-hover:scale-110 transition-transform">📦</span>
            <span class="text-[10px] font-black uppercase tracking-widest text-indigo-700">{{ i18n.t('stock') }}</span>
          </router-link>
          <router-link v-if="auth.canBuy" to="/buying"
            class="flex flex-col items-center p-6 bg-purple-50 rounded-2xl hover:bg-purple-100 transition-colors group">
            <span class="text-2xl mb-2 group-hover:scale-110 transition-transform">🚛</span>
            <span class="text-[10px] font-black uppercase tracking-widest text-purple-700">{{ i18n.t('receive')
            }}</span>
          </router-link>
          <router-link v-if="auth.canViewReports" to="/reports"
            class="flex flex-col items-center p-6 bg-emerald-50 rounded-2xl hover:bg-emerald-100 transition-colors group">
            <span class="text-2xl mb-2 group-hover:scale-110 transition-transform">📊</span>
            <span class="text-[10px] font-black uppercase tracking-widest text-emerald-700">{{ i18n.t('stats') }}</span>
          </router-link>
        </div>
      </div>

      <div class="bg-gray-900 rounded-3xl p-8 flex flex-col justify-between text-white shadow-xl text-left">
        <div>
          <h3 class="text-xs font-black uppercase tracking-[0.2em] text-gray-400 mb-8">{{ i18n.t('system_integrity') }}
          </h3>
          <div class="space-y-6">
            <div class="flex justify-between items-center border-b border-white/10 pb-4">
              <span class="text-sm font-medium text-gray-300">{{ i18n.t('catalog_size') }}</span>
              <span class="font-black text-xl">{{ stats.product_count }} SKU</span>
            </div>
            <div class="flex justify-between items-center border-b border-white/10 pb-4">
              <span class="text-sm font-medium text-gray-300">{{ i18n.t('auth_status') }}</span>
              <span class="bg-blue-500 text-[10px] px-3 py-1 rounded-full font-black uppercase tracking-widest">{{
                auth.user?.role }}</span>
            </div>
          </div>
        </div>
        <div class="mt-8 text-left">
          <div class="text-[10px] text-gray-500 uppercase font-black tracking-widest mb-1">{{ i18n.t('last_updated') }}
          </div>
          <div class="text-xs text-gray-400 font-mono">{{ new Date().toLocaleTimeString() }}</div>
        </div>
      </div>
    </div>

    <!-- Fraud Checker Modal -->
    <div v-if="showFraudChecker" class="fixed inset-0 bg-black/60 backdrop-blur-sm z-50 flex items-center justify-center p-4">
      <div class="bg-white rounded-3xl shadow-2xl w-full max-w-3xl overflow-hidden flex flex-col max-h-[90vh]">
        <!-- Header -->
        <div class="p-6 border-b border-gray-100 flex justify-between items-center bg-gray-50/50">
          <h2 class="text-xl font-black text-gray-900 flex items-center gap-2">
            <span>🛡️</span> {{ i18n.t('fraud_checker') || 'Fraud Checker' }}
          </h2>
          <button @click="showFraudChecker = false" class="text-gray-400 hover:text-gray-600 transition-colors p-2 rounded-full hover:bg-gray-100">
            <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path></svg>
          </button>
        </div>
        
        <!-- Content -->
        <div class="p-6 overflow-y-auto flex-1 flex flex-col gap-6">
          <div class="flex gap-4">
            <input 
              v-model="fraudPhone" 
              @keyup.enter="checkFraud"
              type="text" 
              placeholder="Enter Phone Number..." 
              class="flex-1 bg-white border-2 border-gray-200 text-gray-900 text-base rounded-2xl focus:ring-0 focus:border-blue-500 block w-full p-4 font-bold transition-all shadow-sm"
            >
            <button 
              @click="checkFraud" 
              :disabled="isLoadingFraud || !fraudPhone.trim()"
              class="bg-blue-600 hover:bg-blue-700 text-white px-8 py-4 rounded-2xl font-black text-sm uppercase tracking-widest transition-all shadow-sm active:scale-95 disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2 border border-blue-500"
            >
              <span v-if="isLoadingFraud" class="animate-spin text-base">⏳</span>
              <span v-else class="text-base">🔍</span>
              Check
            </button>
          </div>

          <div v-if="fraudError" class="p-4 bg-red-50 text-red-600 rounded-2xl font-medium border border-red-100">
            {{ fraudError }}
          </div>

          <div v-if="fraudResult" class="bg-gray-900 rounded-3xl p-6 overflow-x-auto shadow-inner w-full">
            <pre class="text-sm font-mono text-green-400 leading-relaxed whitespace-pre-wrap">{{ JSON.stringify(fraudResult, null, 2) }}</pre>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
progress::-webkit-progress-bar {
  background: transparent;
}

progress::-webkit-progress-value {
  border-radius: 99px;
}
</style>
