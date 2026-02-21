<script setup>
import { ref, onMounted, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import jsPDF from 'jspdf';
import autoTable from 'jspdf-autotable';

const currentTab = ref('sales');
const startDate = ref(new Date().toISOString().split('T')[0]);
const endDate = ref(new Date().toISOString().split('T')[0]);
const searchQuery = ref('');

const salesData = ref([]);
const inventoryData = ref([]);
const expensesData = ref([]);
const currencySymbol = ref('৳');
const loading = ref(false);

// --- Sales Computed ---
const filteredSales = computed(() => {
  if (!searchQuery.value) return salesData.value;
  const q = searchQuery.value.toLowerCase();
  return salesData.value.filter(item =>
    (item.customer || '').toLowerCase().includes(q) ||
    String(item.order_id).includes(q) ||
    item.date.includes(q)
  );
});
const totalExpenses = computed(() => expensesData.value.reduce((sum, exp) => sum + exp.amount, 0));
const totalSales = computed(() => filteredSales.value.reduce((sum, item) => sum + item.total, 0));
const totalProfit = computed(() => filteredSales.value.reduce((sum, item) => sum + item.profit, 0) - totalExpenses.value);
const totalDiscount = computed(() => filteredSales.value.reduce((sum, item) => sum + item.discount, 0));
const totalOrderCount = computed(() => filteredSales.value.length);
const totalItemsSold = computed(() => filteredSales.value.reduce((sum, item) => sum + item.items_count, 0));
const avgOrderValue = computed(() => totalOrderCount.value > 0 ? totalSales.value / totalOrderCount.value : 0);
const profitMargin = computed(() => totalSales.value > 0 ? (totalProfit.value / totalSales.value) * 100 : 0);

// --- Inventory Computed ---
const filteredInventory = computed(() => {
  if (!searchQuery.value) return inventoryData.value;
  const q = searchQuery.value.toLowerCase();
  return inventoryData.value.filter(item =>
    item.name.toLowerCase().includes(q) ||
    (item.category || '').toLowerCase().includes(q)
  );
});
const totalStockValue = computed(() => filteredInventory.value.reduce((sum, item) => sum + item.stock_value, 0));
const totalRetailValue = computed(() => filteredInventory.value.reduce((sum, item) => sum + (item.stock * item.selling_price), 0));
const totalPotentialProfit = computed(() => totalRetailValue.value - totalStockValue.value);
const outOfStockCount = computed(() => filteredInventory.value.filter(item => item.stock <= 0).length);
const lowStockCount = computed(() => filteredInventory.value.filter(item => item.stock > 0 && item.stock <= 5).length);

// --- Date Presets ---
function setDatePreset(preset) {
  const today = new Date();
  endDate.value = today.toISOString().split('T')[0];
  if (preset === 'today') {
    startDate.value = endDate.value;
  } else if (preset === 'week') {
    const d = new Date(today);
    d.setDate(d.getDate() - 6);
    startDate.value = d.toISOString().split('T')[0];
  } else if (preset === 'month') {
    startDate.value = new Date(today.getFullYear(), today.getMonth(), 1).toISOString().split('T')[0];
  } else if (preset === 'year') {
    startDate.value = new Date(today.getFullYear(), 0, 1).toISOString().split('T')[0];
  } else if (preset === 'all') {
    startDate.value = '2020-01-01';
  }
  loadReport();
}

async function loadReport() {
  loading.value = true;
  try {
    if (currentTab.value === 'sales') {
      const [sales, expenses] = await Promise.all([
        invoke('get_sales_report', { startDate: startDate.value, endDate: endDate.value }),
        invoke('get_expenses', { startDate: startDate.value, endDate: endDate.value })
      ]);
      salesData.value = sales;
      expensesData.value = expenses;
    } else if (currentTab.value === 'inventory') {
      inventoryData.value = await invoke('get_inventory_report');
    }
    const settingsData = await invoke('get_settings');
    if (settingsData && settingsData.currency_symbol) {
      currencySymbol.value = settingsData.currency_symbol;
    }
  } catch (error) {
    console.error("Failed to load report:", error);
  } finally {
    loading.value = false;
  }
}

function exportPDF() {
  const doc = new jsPDF();
  const now = new Date().toLocaleDateString();

  if (currentTab.value === 'sales') {
    doc.setFontSize(16);
    doc.text(`Sales & Profit Report`, 14, 15);
    doc.setFontSize(9);
    doc.text(`Period: ${startDate.value} to ${endDate.value} | Generated: ${now}`, 14, 22);
    doc.text(`Total Sales: ${currencySymbol.value}${totalSales.value.toFixed(2)} | Net Profit: ${currencySymbol.value}${totalProfit.value.toFixed(2)} | Expenses: ${currencySymbol.value}${totalExpenses.value.toFixed(2)} | Orders: ${totalOrderCount.value}`, 14, 28);

    autoTable(doc, {
      startY: 34,
      head: [['Date', 'Order #', 'Customer', 'Items', 'Discount', 'Total', 'Profit']],
      body: filteredSales.value.map(row => [
        row.date, `#${row.order_id}`, row.customer || '-', row.items_count,
        row.discount.toFixed(2), row.total.toFixed(2), row.profit.toFixed(2)
      ]),
      foot: [['', '', 'TOTALS', totalItemsSold.value, totalDiscount.value.toFixed(2), totalSales.value.toFixed(2), totalProfit.value.toFixed(2)]],
      styles: { fontSize: 8 },
      headStyles: { fillColor: [59, 130, 246] },
      footStyles: { fillColor: [229, 231, 235], textColor: [31, 41, 55], fontStyle: 'bold' },
    });
    doc.save(`sales-report-${startDate.value}-to-${endDate.value}.pdf`);
  } else {
    doc.setFontSize(16);
    doc.text(`Inventory Valuation Report`, 14, 15);
    doc.setFontSize(9);
    doc.text(`Generated: ${now} | Stock Value: ${currencySymbol.value}${totalStockValue.value.toFixed(2)} | Retail Value: ${currencySymbol.value}${totalRetailValue.value.toFixed(2)}`, 14, 22);

    autoTable(doc, {
      startY: 28,
      head: [['Product', 'Category', 'Stock', 'Unit', 'Cost', 'Sell Price', 'Total Value']],
      body: filteredInventory.value.map(row => [
        row.name, row.category || '-', row.stock, row.unit || 'pcs',
        row.cost_price.toFixed(2), row.selling_price.toFixed(2), row.stock_value.toFixed(2)
      ]),
      styles: { fontSize: 8 },
      headStyles: { fillColor: [139, 92, 246] },
    });
    doc.save(`inventory-report-${now}.pdf`);
  }
}

onMounted(() => {
  const date = new Date();
  startDate.value = new Date(date.getFullYear(), date.getMonth(), 1).toISOString().split('T')[0];
  loadReport();
});
</script>

<template>
  <div class="h-full flex flex-col space-y-6">
    <!-- Header -->
    <div class="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-3 sm:gap-0">
      <div>
        <h1 class="text-3xl font-black text-gray-900 tracking-tight">Reports & Analytics</h1>
        <p class="text-gray-400 text-sm font-medium">Comprehensive business intelligence</p>
      </div>
      <button @click="exportPDF"
        :disabled="(currentTab === 'sales' && filteredSales.length === 0) || (currentTab === 'inventory' && filteredInventory.length === 0)"
        class="bg-gradient-to-r from-emerald-500 to-teal-600 hover:from-emerald-600 hover:to-teal-700 text-white px-6 py-2.5 rounded-2xl shadow-lg shadow-emerald-500/20 flex items-center gap-2 font-bold text-sm transition-all active:scale-95 disabled:opacity-40 disabled:cursor-not-allowed">
        <span>📄</span> Export PDF
      </button>
    </div>

    <!-- Controls Bar -->
    <div
      class="bg-white p-4 rounded-2xl shadow-sm border border-gray-100 flex flex-col md:flex-row gap-4 justify-between items-center">
      <!-- Tabs -->
      <div class="flex bg-gray-100 p-1 rounded-xl">
        <button @click="currentTab = 'sales'; searchQuery = ''; loadReport()"
          :class="{ 'bg-white shadow text-blue-600': currentTab === 'sales', 'text-gray-500 hover:text-gray-700': currentTab !== 'sales' }"
          class="px-5 py-2 rounded-lg transition-all font-black text-xs uppercase tracking-widest">
          Sales & Profit
        </button>
        <button @click="currentTab = 'inventory'; searchQuery = ''; loadReport()"
          :class="{ 'bg-white shadow text-purple-600': currentTab === 'inventory', 'text-gray-500 hover:text-gray-700': currentTab !== 'inventory' }"
          class="px-5 py-2 rounded-lg transition-all font-black text-xs uppercase tracking-widest">
          Inventory
        </button>
      </div>

      <!-- Search -->
      <div class="relative w-full md:w-64">
        <span class="absolute left-3 top-2.5 text-gray-400 text-sm">🔍</span>
        <input v-model="searchQuery" type="text" placeholder="Search..."
          class="w-full border border-gray-200 rounded-xl pl-9 pr-3 py-2 text-sm focus:ring-2 focus:ring-blue-500 focus:border-transparent outline-none bg-gray-50 transition-all">
      </div>

      <!-- Date Range (Sales only) -->
      <div v-if="currentTab === 'sales'" class="flex flex-wrap gap-2 items-center">
        <div class="flex bg-gray-50 border border-gray-200 rounded-xl overflow-hidden">
          <button
            v-for="p in [{ label: 'Today', key: 'today' }, { label: 'Week', key: 'week' }, { label: 'Month', key: 'month' }, { label: 'Year', key: 'year' }, { label: 'All', key: 'all' }]"
            :key="p.key" @click="setDatePreset(p.key)"
            class="px-3 py-1.5 text-[10px] font-black uppercase tracking-widest hover:bg-blue-50 hover:text-blue-600 text-gray-500 transition-colors border-r border-gray-200 last:border-r-0">
            {{ p.label }}
          </button>
        </div>
        <input v-model="startDate" type="date" class="border border-gray-200 rounded-lg px-2 py-1.5 text-xs bg-gray-50">
        <span class="text-gray-300 text-xs font-bold">→</span>
        <input v-model="endDate" type="date" class="border border-gray-200 rounded-lg px-2 py-1.5 text-xs bg-gray-50">
        <button @click="loadReport"
          class="bg-blue-600 text-white px-4 py-1.5 rounded-lg hover:bg-blue-700 text-xs font-bold transition-colors active:scale-95">Go</button>
      </div>
    </div>

    <!-- KPI Summary Cards -->
    <div v-if="currentTab === 'sales'" class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-6 gap-4">
      <div class="bg-blue-50 border border-blue-100 p-4 rounded-2xl text-left">
        <div class="text-[10px] font-black text-blue-500 uppercase tracking-widest">Total Revenue</div>
        <div class="text-xl font-black text-blue-800 mt-1">{{ currencySymbol }}{{ totalSales.toLocaleString(undefined,
          { minimumFractionDigits: 2 }) }}</div>
      </div>
      <div class="bg-green-50 border border-green-100 p-4 rounded-2xl text-left">
        <div class="text-[10px] font-black text-green-500 uppercase tracking-widest">Net Profit</div>
        <div class="text-xl font-black text-green-800 mt-1">{{ currencySymbol }}{{ totalProfit.toLocaleString(undefined,
          { minimumFractionDigits: 2 }) }}</div>
      </div>
      <div class="bg-amber-50 border border-amber-100 p-4 rounded-2xl text-left">
        <div class="text-[10px] font-black text-amber-500 uppercase tracking-widest">Discounts Given</div>
        <div class="text-xl font-black text-amber-800 mt-1">{{ currencySymbol }}{{
          totalDiscount.toLocaleString(undefined, { minimumFractionDigits: 2 }) }}</div>
      </div>
      <div class="bg-indigo-50 border border-indigo-100 p-4 rounded-2xl text-left">
        <div class="text-[10px] font-black text-indigo-500 uppercase tracking-widest">Total Orders</div>
        <div class="text-xl font-black text-indigo-800 mt-1">{{ totalOrderCount }}</div>
      </div>
      <div class="bg-purple-50 border border-purple-100 p-4 rounded-2xl text-left">
        <div class="text-[10px] font-black text-purple-500 uppercase tracking-widest">Avg Order Value</div>
        <div class="text-xl font-black text-purple-800 mt-1">{{ currencySymbol }}{{ avgOrderValue.toFixed(2) }}</div>
      </div>
      <!-- Expenses Card -->
      <div class="bg-rose-50 border border-rose-100 p-4 rounded-2xl text-left">
        <div class="text-[10px] font-black text-rose-500 uppercase tracking-widest">Total Expenses</div>
        <div class="text-xl font-black text-rose-800 mt-1">{{ currencySymbol }}{{
          totalExpenses.toLocaleString(undefined, { minimumFractionDigits: 2 }) }}</div>
      </div>
      <div class="bg-teal-50 border border-teal-100 p-4 rounded-2xl text-left">
        <div class="text-[10px] font-black text-teal-500 uppercase tracking-widest">Profit Margin</div>
        <div class="text-xl font-black text-teal-800 mt-1">{{ profitMargin.toFixed(1) }}%</div>
      </div>
    </div>

    <div v-if="currentTab === 'inventory'" class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-5 gap-4">
      <div class="bg-purple-50 border border-purple-100 p-4 rounded-2xl text-left">
        <div class="text-[10px] font-black text-purple-500 uppercase tracking-widest">Cost Value</div>
        <div class="text-xl font-black text-purple-800 mt-1">{{ currencySymbol }}{{
          totalStockValue.toLocaleString(undefined, { minimumFractionDigits: 2 }) }}</div>
      </div>
      <div class="bg-blue-50 border border-blue-100 p-4 rounded-2xl text-left">
        <div class="text-[10px] font-black text-blue-500 uppercase tracking-widest">Retail Value</div>
        <div class="text-xl font-black text-blue-800 mt-1">{{ currencySymbol }}{{
          totalRetailValue.toLocaleString(undefined, { minimumFractionDigits: 2 }) }}</div>
      </div>
      <div class="bg-green-50 border border-green-100 p-4 rounded-2xl text-left">
        <div class="text-[10px] font-black text-green-500 uppercase tracking-widest">Potential Profit</div>
        <div class="text-xl font-black text-green-800 mt-1">{{ currencySymbol }}{{
          totalPotentialProfit.toLocaleString(undefined, { minimumFractionDigits: 2 }) }}</div>
      </div>
      <div class="bg-red-50 border border-red-100 p-4 rounded-2xl text-left">
        <div class="text-[10px] font-black text-red-500 uppercase tracking-widest">Out of Stock</div>
        <div class="text-xl font-black text-red-800 mt-1">{{ outOfStockCount }}</div>
      </div>
      <div class="bg-amber-50 border border-amber-100 p-4 rounded-2xl text-left">
        <div class="text-[10px] font-black text-amber-500 uppercase tracking-widest">Low Stock</div>
        <div class="text-xl font-black text-amber-800 mt-1">{{ lowStockCount }}</div>
      </div>
    </div>

    <!-- Data Table -->
    <div class="bg-white rounded-2xl shadow-sm border border-gray-100 overflow-hidden flex-1 flex flex-col min-h-0">
      <div v-if="loading" class="flex-1 flex items-center justify-center">
        <div class="text-gray-400 font-bold text-sm animate-pulse">Loading report data...</div>
      </div>

      <!-- Sales Table -->
      <div v-else-if="currentTab === 'sales'" class="flex-1 overflow-auto">
        <table class="w-full text-left border-collapse min-w-[700px]">
          <thead class="bg-gray-50 text-[10px] font-black text-gray-400 uppercase tracking-widest sticky top-0 z-10">
            <tr>
              <th class="px-5 py-4 border-b border-gray-100">Date</th>
              <th class="px-5 py-4 border-b border-gray-100">Order #</th>
              <th class="px-5 py-4 border-b border-gray-100">Customer</th>
              <th class="px-5 py-4 border-b border-gray-100 text-center">Items</th>
              <th class="px-5 py-4 border-b border-gray-100 text-right">Discount</th>
              <th class="px-5 py-4 border-b border-gray-100 text-right">Total</th>
              <th class="px-5 py-4 border-b border-gray-100 text-right">Profit</th>
            </tr>
          </thead>
          <tbody class="text-gray-700 text-sm">
            <tr v-for="item in filteredSales" :key="item.order_id"
              class="hover:bg-blue-50/30 border-b border-gray-50 last:border-b-0 transition-colors">
              <td class="px-5 py-3.5 text-xs text-gray-500 font-mono">{{ item.date }}</td>
              <td class="px-5 py-3.5 font-bold text-blue-600">#{{ item.order_id }}</td>
              <td class="px-5 py-3.5">{{ item.customer || '—' }}</td>
              <td class="px-5 py-3.5 text-center">
                <span class="bg-gray-100 text-gray-600 px-2 py-0.5 rounded-full text-[10px] font-black">{{
                  item.items_count }}</span>
              </td>
              <td class="px-5 py-3.5 text-right text-amber-600 text-xs">{{ item.discount > 0 ?
                `-${currencySymbol}${item.discount.toFixed(2)}` : '—' }}</td>
              <td class="px-5 py-3.5 text-right font-bold">{{ currencySymbol }}{{ item.total.toFixed(2) }}</td>
              <td class="px-5 py-3.5 text-right font-black"
                :class="item.profit >= 0 ? 'text-green-600' : 'text-red-500'">
                {{ currencySymbol }}{{ item.profit.toFixed(2) }}
              </td>
            </tr>
            <tr v-if="filteredSales.length === 0">
              <td colspan="7" class="px-5 py-16 text-center">
                <div class="text-gray-300 text-4xl mb-2">📊</div>
                <div class="text-gray-400 font-bold text-sm">No sales data found for the selected period</div>
                <div class="text-gray-300 text-xs mt-1">Try adjusting the date range or search filters</div>
              </td>
            </tr>
          </tbody>
          <tfoot v-if="filteredSales.length > 0" class="bg-gray-50 font-black text-sm sticky bottom-0">
            <tr>
              <td class="px-5 py-4 text-gray-500" colspan="3">TOTALS ({{ totalOrderCount }} orders)</td>
              <td class="px-5 py-4 text-center text-gray-700">{{ totalItemsSold }}</td>
              <td class="px-5 py-4 text-right text-amber-600">{{ currencySymbol }}{{ totalDiscount.toFixed(2) }}</td>
              <td class="px-5 py-4 text-right text-blue-700">{{ currencySymbol }}{{ totalSales.toFixed(2) }}</td>
              <td class="px-5 py-4 text-right text-green-700">{{ currencySymbol }}{{ totalProfit.toFixed(2) }}</td>
            </tr>
          </tfoot>
        </table>
      </div>

      <!-- Inventory Table -->
      <div v-else-if="currentTab === 'inventory'" class="flex-1 overflow-auto">
        <table class="w-full text-left border-collapse min-w-[800px]">
          <thead class="bg-gray-50 text-[10px] font-black text-gray-400 uppercase tracking-widest sticky top-0 z-10">
            <tr>
              <th class="px-5 py-4 border-b border-gray-100">Product</th>
              <th class="px-5 py-4 border-b border-gray-100">Category</th>
              <th class="px-5 py-4 border-b border-gray-100 text-center">Stock</th>
              <th class="px-5 py-4 border-b border-gray-100 text-right">Avg Cost</th>
              <th class="px-5 py-4 border-b border-gray-100 text-right">Sell Price</th>
              <th class="px-5 py-4 border-b border-gray-100 text-right">Margin</th>
              <th class="px-5 py-4 border-b border-gray-100 text-right">Stock Value</th>
            </tr>
          </thead>
          <tbody class="text-gray-700 text-sm">
            <tr v-for="item in filteredInventory" :key="item.id"
              class="hover:bg-purple-50/30 border-b border-gray-50 last:border-b-0 transition-colors">
              <td class="px-5 py-3.5 font-bold text-gray-800">{{ item.name }}</td>
              <td class="px-5 py-3.5">
                <span v-if="item.category"
                  class="bg-indigo-50 text-indigo-600 px-2 py-0.5 rounded-full text-[10px] font-black uppercase">{{
                    item.category }}</span>
                <span v-else class="text-gray-300">—</span>
              </td>
              <td class="px-5 py-3.5 text-center">
                <span class="px-2 py-0.5 rounded-full text-[11px] font-black"
                  :class="item.stock <= 0 ? 'bg-red-100 text-red-600' : item.stock <= 5 ? 'bg-amber-100 text-amber-700' : 'bg-green-100 text-green-700'">
                  {{ item.stock }} {{ item.unit || 'pcs' }}
                </span>
              </td>
              <td class="px-5 py-3.5 text-right text-gray-600">{{ currencySymbol }}{{ item.cost_price.toFixed(2) }}</td>
              <td class="px-5 py-3.5 text-right font-bold">{{ currencySymbol }}{{ item.selling_price.toFixed(2) }}</td>
              <td class="px-5 py-3.5 text-right">
                <span class="font-black text-xs"
                  :class="item.selling_price > item.cost_price ? 'text-green-600' : 'text-red-500'">
                  {{ item.cost_price > 0 ? (((item.selling_price - item.cost_price) / item.cost_price) * 100).toFixed(1)
                    : '—' }}%
                </span>
              </td>
              <td class="px-5 py-3.5 text-right font-black text-purple-700">{{ currencySymbol }}{{
                item.stock_value.toFixed(2) }}</td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
  </div>
</template>
