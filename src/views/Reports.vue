<script setup>
import { ref, onMounted, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import jsPDF from 'jspdf';
import autoTable from 'jspdf-autotable';

const currentTab = ref('sales'); // 'sales', 'inventory'
const startDate = ref(new Date().toISOString().split('T')[0]);
const endDate = ref(new Date().toISOString().split('T')[0]);

const salesData = ref([]);
const inventoryData = ref([]);
const currencySymbol = ref('$'); // Default

const totalSales = computed(() => salesData.value.reduce((sum, item) => sum + item.total, 0));
const totalProfit = computed(() => salesData.value.reduce((sum, item) => sum + item.profit, 0));
const totalStockValue = computed(() => inventoryData.value.reduce((sum, item) => sum + item.stock_value, 0));

async function loadReport() {
  try {
    if (currentTab.value === 'sales') {
      salesData.value = await invoke('get_sales_report', {
        startDate: startDate.value,
        endDate: endDate.value
      });
    } else if (currentTab.value === 'inventory') {
      inventoryData.value = await invoke('get_inventory_report');
    }
    const settingsData = await invoke('get_settings');
    if (settingsData && settingsData.currency_symbol) {
      currencySymbol.value = settingsData.currency_symbol;
    }
  } catch (error) {
    console.error("Failed to load report:", error);
    alert("Error loading report: " + error);
  }
}

function exportPDF() {
  const doc = new jsPDF();

  if (currentTab.value === 'sales') {
    doc.text(`Sales Report (${startDate.value} to ${endDate.value})`, 14, 15);
    doc.setFontSize(10);
    // Note: PDF currency symbol support depends on font. Using "Total" instead of symbol might be safer or just string concat manually.
    // For now assuming $ or ৳ might need font, but let's stick to simple text for PDF.
    doc.text(`Total Sales: ${currencySymbol.value}${totalSales.value.toFixed(2)} | Total Profit: ${currencySymbol.value}${totalProfit.value.toFixed(2)}`, 14, 22);

    autoTable(doc, {
      startY: 28,
      head: [['Date', 'Order #', 'Customer', 'Total', 'Profit']],
      body: salesData.value.map(row => [
        row.date,
        row.order_id,
        row.customer || '-',
        row.total.toFixed(2),
        row.profit.toFixed(2)
      ]),
    });

    doc.save(`sales-report-${startDate.value}.pdf`);
  } else {
    doc.text(`Inventory Valuation Report (${new Date().toLocaleDateString()})`, 14, 15);
    doc.text(`Total Stock Value: ${currencySymbol.value}${totalStockValue.value.toFixed(2)}`, 14, 22);

    autoTable(doc, {
      startY: 28,
      head: [['Product', 'Stock', 'Unit', 'Cost Price', 'Value']],
      body: inventoryData.value.map(row => [
        row.name,
        row.stock,
        row.unit,
        row.cost_price.toFixed(2),
        row.stock_value.toFixed(2)
      ]),
    });

    doc.save(`inventory-report.pdf`);
  }
}

onMounted(() => {
  // Set default range to current month
  const date = new Date();
  const firstDay = new Date(date.getFullYear(), date.getMonth(), 1);
  startDate.value = firstDay.toISOString().split('T')[0];

  loadReport();
});
</script>

<template>
  <div class="h-full flex flex-col space-y-6">
    <div class="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-3">
      <h1 class="text-2xl md:text-3xl font-bold text-gray-800">Reports</h1>
      <div class="space-x-2">
        <button @click="exportPDF"
          class="bg-green-600 hover:bg-green-700 text-white px-4 py-2 rounded-lg shadow flex items-center gap-2"
          :disabled="(currentTab === 'sales' && salesData.length === 0) || (currentTab === 'inventory' && inventoryData.length === 0)">
          <span>Download PDF</span>
        </button>
      </div>
    </div>

    <!-- Controls -->
    <div class="bg-white p-4 rounded-lg shadow flex flex-col md:flex-row gap-4 justify-between items-center">
      <div class="flex bg-gray-100 p-1 rounded-lg">
        <button @click="currentTab = 'sales'; loadReport()"
          :class="{ 'bg-white shadow text-blue-600': currentTab === 'sales', 'text-gray-600': currentTab !== 'sales' }"
          class="px-4 py-2 rounded-md transition-all font-medium">
          Sales & Profit
        </button>
        <button @click="currentTab = 'inventory'; loadReport()"
          :class="{ 'bg-white shadow text-blue-600': currentTab === 'inventory', 'text-gray-600': currentTab !== 'inventory' }"
          class="px-4 py-2 rounded-md transition-all font-medium">
          Inventory Value
        </button>
      </div>

      <div v-if="currentTab === 'sales'" class="flex gap-2 items-center">
        <input v-model="startDate" type="date" class="border border-gray-300 rounded-lg px-3 py-2">
        <span class="text-gray-500">to</span>
        <input v-model="endDate" type="date" class="border border-gray-300 rounded-lg px-3 py-2">
        <button @click="loadReport"
          class="bg-blue-600 text-white px-4 py-2 rounded-lg hover:bg-blue-700">Filter</button>
      </div>
    </div>

    <!-- Content -->
    <div class="bg-white rounded-lg shadow overflow-hidden flex-1 overflow-x-auto overflow-y-auto">

      <!-- Summaries -->
      <div v-if="currentTab === 'sales'" class="bg-blue-50 p-4 border-b border-blue-100 flex gap-8">
        <div>
          <div class="text-xs text-blue-500 uppercase font-bold">Total Sales</div>
          <div class="text-2xl font-bold text-blue-700">{{ currencySymbol }}{{ totalSales.toFixed(2) }}</div>
        </div>
        <div>
          <div class="text-xs text-green-500 uppercase font-bold">Net Profit</div>
          <div class="text-2xl font-bold text-green-700">{{ currencySymbol }}{{ totalProfit.toFixed(2) }}</div>
        </div>
      </div>

      <div v-if="currentTab === 'inventory'" class="bg-purple-50 p-4 border-b border-purple-100">
        <div>
          <div class="text-xs text-purple-500 uppercase font-bold">Total Inventory Value</div>
          <div class="text-2xl font-bold text-purple-700">{{ currencySymbol }}{{ totalStockValue.toFixed(2) }}</div>
        </div>
      </div>

      <!-- Table: Sales -->
      <table v-if="currentTab === 'sales'" class="w-full text-left border-collapse min-w-[500px]">
        <thead class="bg-gray-100 text-gray-600 uppercase text-xs font-semibold sticky top-0 z-10">
          <tr>
            <th class="p-4 border-b">Date</th>
            <th class="p-4 border-b">Order #</th>
            <th class="p-4 border-b">Customer</th>
            <th class="p-4 border-b text-right">Total</th>
            <th class="p-4 border-b text-right">Profit</th>
          </tr>
        </thead>
        <tbody class="text-gray-700">
          <tr v-for="item in salesData" :key="item.order_id" class="hover:bg-gray-50 border-b last:border-b-0">
            <td class="p-4">{{ item.date }}</td>
            <td class="p-4">#{{ item.order_id }}</td>
            <td class="p-4">{{ item.customer || '-' }}</td>
            <td class="p-4 text-right font-medium">{{ currencySymbol }}{{ item.total.toFixed(2) }}</td>
            <td class="p-4 text-right text-green-600 font-bold">{{ currencySymbol }}{{ item.profit.toFixed(2) }}</td>
          </tr>
          <tr v-if="salesData.length === 0">
            <td colspan="5" class="p-8 text-center text-gray-500">No sales found for selected period.</td>
          </tr>
        </tbody>
      </table>

      <!-- Table: Inventory -->
      <table v-if="currentTab === 'inventory'" class="w-full text-left border-collapse min-w-[500px]">
        <thead class="bg-gray-100 text-gray-600 uppercase text-xs font-semibold sticky top-0 z-10">
          <tr>
            <th class="p-4 border-b">Product</th>
            <th class="p-4 border-b">Stock Qty</th>
            <th class="p-4 border-b">Cost Price</th>
            <th class="p-4 border-b text-right">Total Value</th>
          </tr>
        </thead>
        <tbody class="text-gray-700">
          <tr v-for="item in inventoryData" :key="item.id" class="hover:bg-gray-50 border-b last:border-b-0">
            <td class="p-4 font-medium">{{ item.name }}</td>
            <td class="p-4">{{ item.stock }} {{ item.unit }}</td>
            <td class="p-4">{{ currencySymbol }}{{ item.cost_price.toFixed(2) }}</td>
            <td class="p-4 text-right font-bold">{{ currencySymbol }}{{ item.stock_value.toFixed(2) }}</td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>
