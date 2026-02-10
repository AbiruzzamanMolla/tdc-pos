<script setup>
import { ref, onMounted, computed, reactive, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';

const viewMode = ref('new');
const purchases = ref([]);
const products = ref([]);
const cart = ref([]);
const showDetailsModal = ref(false);
const selectedPurchase = ref(null);
const searchQuery = ref("");
const currencySymbol = ref('৳');

const form = reactive({
  supplier_name: "",
  supplier_phone: "",
  invoice_number: "",
  purchase_date: new Date().toISOString().split('T')[0],
  notes: ""
});

const filteredProducts = computed(() => {
  if (!searchQuery.value) return products.value;
  const query = searchQuery.value.toLowerCase();
  return products.value.filter(p =>
    p.product_name.toLowerCase().includes(query) ||
    (p.product_code && p.product_code.toLowerCase().includes(query))
  );
});

const totalAmount = computed(() => {
  return cart.value.reduce((sum, item) => sum + item.subtotal, 0);
});

async function loadPurchases() {
  try {
    const [purchasesData, settingsData] = await Promise.all([
      invoke('get_purchases'),
      invoke('get_settings')
    ]);
    purchases.value = purchasesData;
    if (settingsData && settingsData.currency_symbol) {
      currencySymbol.value = settingsData.currency_symbol;
    }
  } catch (error) {
    console.error("Failed to load purchases:", error);
  }
}

async function loadProducts() {
  try {
    products.value = await invoke('get_products');
  } catch (error) {
    console.error("Failed to load products:", error);
  }
}

function addToCart(product) {
  const existing = cart.value.find(i => i.product_id === product.id);
  if (existing) {
    existing.quantity++;
    existing.subtotal = existing.quantity * existing.buying_price;
    return;
  }

  cart.value.push({
    product_id: product.id,
    product_name: product.product_name,
    quantity: 1,
    buying_price: product.buying_price,
    subtotal: product.buying_price
  });
}

function updateQuantity(item, delta) {
  const newQty = item.quantity + delta;
  if (newQty > 0) {
    item.quantity = newQty;
    item.subtotal = item.quantity * item.buying_price;
  }
}

function updatePrice(item) {
  if (item.buying_price < 0) item.buying_price = 0;
  item.subtotal = item.quantity * item.buying_price;
}

function removeFromCart(index) {
  cart.value.splice(index, 1);
}

async function savePurchase() {
  if (cart.value.length === 0) {
    alert("Please add at least one product.");
    return;
  }

  try {
    const purchaseData = {
      supplier_name: form.supplier_name,
      supplier_phone: form.supplier_phone,
      invoice_number: form.invoice_number,
      purchase_date: form.purchase_date,
      total_amount: totalAmount.value,
      notes: form.notes
    };

    const itemsData = cart.value.map(item => ({
      product_id: item.product_id,
      quantity: Number(item.quantity),
      buying_price: Number(item.buying_price),
      subtotal: Number(item.subtotal)
    }));

    await invoke('create_purchase', { purchase: purchaseData, items: itemsData });

    // Reset
    cart.value = [];
    form.supplier_name = "";
    form.supplier_phone = "";
    form.invoice_number = "";
    form.purchase_date = new Date().toISOString().split('T')[0];
    form.notes = "";
    loadProducts();
    alert("Buying entry saved successfully! Stock updated.");
  } catch (error) {
    console.error("Failed to save buying entry:", error);
    alert("Error saving buying entry: " + error);
  }
}

async function viewPurchaseDetails(purchase) {
  selectedPurchase.value = purchase;
  try {
    const items = await invoke('get_purchase_items', { purchaseId: purchase.purchase_id });
    selectedPurchase.value = { ...purchase, items: items };
    showDetailsModal.value = true;
  } catch (e) {
    console.error("Failed to load purchase items", e);
    alert("Failed to load details");
  }
}

async function deletePurchase(purchase) {
  if (!confirm(`Are you sure you want to delete this buying entry? It will reverse stock quantities.`)) return;
  try {
    await invoke('delete_purchase', { purchaseId: purchase.purchase_id });
    loadPurchases();
  } catch (error) {
    console.error("Failed to delete buying entry:", error);
    alert("Failed to delete: " + error);
  }
}

watch(viewMode, (newMode) => {
  if (newMode === 'history') loadPurchases();
  if (newMode === 'new') loadProducts();
});

onMounted(() => {
  loadProducts();
  invoke('get_settings').then(s => {
    if (s && s.currency_symbol) currencySymbol.value = s.currency_symbol;
  });
});
</script>

<template>
  <div class="h-full flex flex-col">
    <!-- Header Toggle -->
    <div class="flex flex-col sm:flex-row justify-between items-start sm:items-center mb-4 gap-3 px-1">
      <h1 class="text-2xl md:text-3xl font-bold text-gray-800">
        {{ viewMode === 'new' ? 'Buying' : 'Buying History' }}
      </h1>
      <div class="bg-gray-200 p-1 rounded-lg flex text-sm font-medium">
        <button @click="viewMode = 'new'"
          :class="{ 'bg-white shadow text-blue-600': viewMode === 'new', 'text-gray-500 hover:text-gray-700': viewMode !== 'new' }"
          class="px-4 py-2 rounded-md transition-all">
          New Entry
        </button>
        <button @click="viewMode = 'history'"
          :class="{ 'bg-white shadow text-blue-600': viewMode === 'history', 'text-gray-500 hover:text-gray-700': viewMode !== 'history' }"
          class="px-4 py-2 rounded-md transition-all">
          History
        </button>
      </div>
    </div>

    <!-- NEW BUYING VIEW (POS Style) -->
    <div v-if="viewMode === 'new'" class="flex flex-col lg:flex-row flex-1 gap-4 overflow-hidden">

      <!-- Left: Products to Buy -->
      <div class="flex-1 flex flex-col bg-white rounded-lg shadow overflow-hidden min-h-0">
        <div class="p-3 border-b border-gray-100">
          <input v-model="searchQuery" type="text" placeholder="Search products to buy..."
            class="w-full border border-gray-300 rounded-lg px-3 py-2 focus:ring-blue-500 focus:outline-none text-sm">
        </div>

        <div class="p-3 grid grid-cols-2 sm:grid-cols-3 xl:grid-cols-4 gap-3 overflow-y-auto content-start flex-1">
          <div v-for="product in filteredProducts" :key="product.id" @click="addToCart(product)"
            class="border border-gray-200 rounded-xl p-3 cursor-pointer hover:shadow-md transition-shadow bg-gray-50 hover:bg-white active:scale-95 transform transition-transform">
            <div
              class="h-16 bg-gray-200 rounded-lg mb-2 flex items-center justify-center text-gray-400 text-2xl font-bold">
              {{ product.product_name.charAt(0) }}
            </div>
            <h3 class="font-bold text-gray-800 text-xs truncate">{{ product.product_name }}</h3>
            <div class="flex justify-between items-center mt-1">
              <span class="text-green-600 font-bold text-sm">{{ currencySymbol }}{{ product.buying_price.toFixed(2)
                }}</span>
              <span class="text-xs px-1.5 py-0.5 rounded-full bg-blue-100 text-blue-700">
                Stock: {{ product.stock_quantity }}
              </span>
            </div>
          </div>
          <div v-if="filteredProducts.length === 0" class="col-span-full text-center text-gray-400 py-8">
            No products found.
          </div>
        </div>
      </div>

      <!-- Right: Buying Cart & Supplier Info -->
      <div
        class="w-full lg:w-[420px] flex flex-col bg-white rounded-lg shadow overflow-hidden flex-shrink-0 max-h-[50vh] lg:max-h-full">
        <div class="p-3 border-b bg-gray-50 font-bold text-gray-700 text-sm">Buying Cart ({{ cart.length }} items)</div>

        <!-- Supplier Info -->
        <div class="p-3 border-b bg-white space-y-2">
          <div class="grid grid-cols-2 gap-2">
            <div>
              <label class="block text-xs font-medium text-gray-500">Supplier</label>
              <input v-model="form.supplier_name" type="text" placeholder="Supplier name"
                class="w-full border border-gray-200 rounded px-2 py-1 text-sm focus:ring-blue-500 focus:outline-none">
            </div>
            <div>
              <label class="block text-xs font-medium text-gray-500">Invoice #</label>
              <input v-model="form.invoice_number" type="text" placeholder="INV-001"
                class="w-full border border-gray-200 rounded px-2 py-1 text-sm focus:ring-blue-500 focus:outline-none">
            </div>
          </div>
          <div class="grid grid-cols-2 gap-2">
            <div>
              <label class="block text-xs font-medium text-gray-500">Phone</label>
              <input v-model="form.supplier_phone" type="text" placeholder="Phone"
                class="w-full border border-gray-200 rounded px-2 py-1 text-sm focus:ring-blue-500 focus:outline-none">
            </div>
            <div>
              <label class="block text-xs font-medium text-gray-500">Date</label>
              <input v-model="form.purchase_date" type="date"
                class="w-full border border-gray-200 rounded px-2 py-1 text-sm focus:ring-blue-500 focus:outline-none">
            </div>
          </div>
        </div>

        <!-- Items -->
        <div class="flex-1 overflow-y-auto p-3 space-y-3">
          <div v-for="(item, index) in cart" :key="item.product_id" class="border border-gray-100 rounded-lg p-3 group">
            <div class="flex justify-between items-start">
              <div class="flex-1 min-w-0">
                <div class="font-medium text-gray-800 text-sm truncate">{{ item.product_name }}</div>
              </div>
              <button @click="removeFromCart(index)"
                class="text-red-400 hover:text-red-600 ml-2 text-lg leading-none">×</button>
            </div>
            <div class="flex items-center gap-2 mt-2 flex-wrap">
              <div class="flex items-center border rounded text-sm">
                <button @click="updateQuantity(item, -1)" class="px-2 py-0.5 text-gray-600 hover:bg-gray-100">-</button>
                <span class="px-2 font-bold text-xs">{{ item.quantity }}</span>
                <button @click="updateQuantity(item, 1)" class="px-2 py-0.5 text-gray-600 hover:bg-gray-100">+</button>
              </div>
              <span class="text-xs text-gray-400">×</span>
              <div class="flex items-center gap-1">
                <span class="text-xs text-gray-500">{{ currencySymbol }}</span>
                <input type="number" v-model.number="item.buying_price" @input="updatePrice(item)"
                  class="w-20 border border-gray-200 rounded px-2 py-0.5 text-xs focus:ring-blue-500 focus:outline-none"
                  step="0.01" min="0">
              </div>
              <span class="text-xs text-gray-400">=</span>
              <span class="font-bold text-sm text-gray-800">{{ currencySymbol }}{{ item.subtotal.toFixed(2) }}</span>
            </div>
          </div>
          <div v-if="cart.length === 0" class="text-center text-gray-400 mt-8 text-sm">
            Click products to add to buying cart
          </div>
        </div>

        <!-- Notes -->
        <div class="px-3 py-2 border-t bg-white">
          <textarea v-model="form.notes" rows="1" placeholder="Notes / Remarks..."
            class="w-full border border-gray-200 rounded px-2 py-1 text-xs resize-none focus:ring-blue-500 focus:outline-none"></textarea>
        </div>

        <!-- Footer -->
        <div class="p-3 bg-gray-50 border-t space-y-1 text-sm">
          <div class="flex justify-between text-xl font-bold text-gray-800 pt-1">
            <span>Total</span>
            <span>{{ currencySymbol }}{{ totalAmount.toFixed(2) }}</span>
          </div>
          <button @click="savePurchase" :disabled="cart.length === 0"
            class="w-full mt-3 bg-green-600 text-white py-2.5 rounded-lg font-bold shadow-lg hover:bg-green-700 disabled:bg-gray-400 disabled:cursor-not-allowed transition text-sm">
            Save Buying Entry
          </button>
        </div>
      </div>
    </div>

    <!-- HISTORY VIEW -->
    <div v-else class="bg-white rounded-lg shadow overflow-hidden flex-1 overflow-x-auto overflow-y-auto">
      <table class="w-full text-left border-collapse min-w-[600px]">
        <thead class="bg-gray-100 text-gray-600 uppercase text-xs font-semibold sticky top-0 z-10">
          <tr>
            <th class="p-3 border-b">Date</th>
            <th class="p-3 border-b">Supplier</th>
            <th class="p-3 border-b">Invoice #</th>
            <th class="p-3 border-b text-right">Total</th>
            <th class="p-3 border-b">Notes</th>
            <th class="p-3 border-b text-center">Actions</th>
          </tr>
        </thead>
        <tbody class="text-gray-700 text-sm">
          <tr v-for="purchase in purchases" :key="purchase.purchase_id"
            class="hover:bg-gray-50 border-b last:border-b-0">
            <td class="p-3 text-xs">{{ purchase.purchase_date }}</td>
            <td class="p-3 font-medium text-sm">{{ purchase.supplier_name || '-' }}</td>
            <td class="p-3 text-gray-500 text-xs">{{ purchase.invoice_number || '-' }}</td>
            <td class="p-3 text-right font-bold">{{ currencySymbol }}{{ purchase.total_amount.toFixed(2) }}</td>
            <td class="p-3 text-gray-500 text-xs truncate max-w-[150px]">{{ purchase.notes || '-' }}</td>
            <td class="p-3 text-center">
              <div class="flex justify-center gap-1">
                <button @click="viewPurchaseDetails(purchase)"
                  class="text-blue-600 hover:text-blue-800 text-xs font-medium border border-blue-200 px-2 py-1 rounded hover:bg-blue-50">View</button>
                <button @click="deletePurchase(purchase)"
                  class="text-red-600 hover:text-red-800 text-xs font-medium border border-red-200 px-2 py-1 rounded hover:bg-red-50">Delete</button>
              </div>
            </td>
          </tr>
          <tr v-if="purchases.length === 0">
            <td colspan="6" class="p-8 text-center text-gray-500">No buying history found.</td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- Details Modal -->
    <div v-if="showDetailsModal && selectedPurchase"
      class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4">
      <div class="bg-white rounded-xl shadow-2xl w-full max-w-2xl p-5 relative max-h-[90vh] flex flex-col">
        <button @click="showDetailsModal = false"
          class="absolute top-3 right-3 text-gray-500 hover:text-gray-700 text-xl">✕</button>
        <h2 class="text-xl font-bold mb-1 text-gray-800">Buying Entry Details</h2>
        <div class="text-xs text-gray-500 mb-4">Invoice: {{ selectedPurchase.invoice_number || 'N/A' }} | {{
          selectedPurchase.purchase_date }}</div>

        <div class="grid grid-cols-2 gap-3 mb-4 bg-gray-50 p-3 rounded-lg text-sm">
          <div>
            <span class="block text-xs text-gray-500 uppercase">Supplier</span>
            <span class="font-medium text-gray-800">{{ selectedPurchase.supplier_name || 'N/A' }}</span>
          </div>
          <div class="text-right">
            <span class="block text-xs text-gray-500 uppercase">Phone</span>
            <span class="font-medium text-gray-800">{{ selectedPurchase.supplier_phone || 'N/A' }}</span>
          </div>
        </div>

        <div class="flex-1 overflow-y-auto overflow-x-auto">
          <table class="w-full text-left text-sm border-collapse min-w-[400px]">
            <thead class="bg-gray-100 text-gray-600">
              <tr>
                <th class="p-2 border-b text-xs">Product</th>
                <th class="p-2 border-b text-right text-xs">Qty</th>
                <th class="p-2 border-b text-right text-xs">Buy Price</th>
                <th class="p-2 border-b text-right text-xs">Subtotal</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="item in selectedPurchase.items" :key="item.id" class="border-b last:border-0 hover:bg-gray-50">
                <td class="p-2 font-medium text-sm">{{ item.product_name }}</td>
                <td class="p-2 text-right text-sm">{{ item.quantity }}</td>
                <td class="p-2 text-right text-sm">{{ currencySymbol }}{{ item.buying_price.toFixed(2) }}</td>
                <td class="p-2 text-right font-medium text-sm">{{ currencySymbol }}{{ item.subtotal.toFixed(2) }}</td>
              </tr>
            </tbody>
          </table>
        </div>

        <div class="border-t mt-3 pt-3">
          <div v-if="selectedPurchase.notes" class="mb-2">
            <p class="text-xs text-gray-500 uppercase mb-1">Notes</p>
            <p class="text-sm text-gray-700 bg-gray-50 p-2 rounded">{{ selectedPurchase.notes }}</p>
          </div>
          <div class="flex justify-between items-end">
            <div class="text-xs text-gray-500 uppercase">Total Amount</div>
            <div class="text-2xl font-bold text-gray-800">{{ currencySymbol }}{{
              selectedPurchase.total_amount.toFixed(2) }}</div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
