<script setup>
import { ref, onMounted, computed, reactive, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import ProductDetailsModal from '../components/ProductDetailsModal.vue';
import { logActivity } from '../utils/activityLogger';

const viewMode = ref('new');
const purchases = ref([]);
const products = ref([]);
const cart = ref([]);
const showDetailsModal = ref(false);
const selectedPurchase = ref(null);
const searchQuery = ref("");
const currencySymbol = ref('৳');
const showProductDetails = ref(false);
const selectedProductDetails = ref(null);

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
  return cart.value.reduce((sum, item) => sum + (item.subtotal || 0), 0);
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
    const prods = await invoke('get_products');
    // Load first image preview for each product
    for (const p of prods) {
      if (p.images && p.images.length > 0) {
        try {
          p._thumb = await invoke('read_image_base64', { path: p.images[0] });
        } catch { p._thumb = null; }
      } else {
        p._thumb = null;
      }
    }
    products.value = prods;
  } catch (error) {
    console.error("Failed to load products:", error);
  }
}

function openDetails(product) {
  selectedProductDetails.value = product;
  showProductDetails.value = true;
}

function addToCart(product) {
  const existing = cart.value.find(i => i.product_id === product.id);
  if (existing) {
    existing.quantity++;
    recalculateItem(existing);
    return;
  }

  const newItem = {
    product_id: product.id,
    product_name: product.product_name,
    _thumb: product._thumb,
    quantity: 1,
    buying_price: product.buying_price, // Unit Price input
    extra_charge: 0,
    subtotal: product.buying_price,
    purchase_unit_cost: product.buying_price
  };
  cart.value.push(newItem);
}

function recalculateItem(item) {
  const qty = Number(item.quantity) || 0;
  const price = Number(item.buying_price) || 0;
  const extra = Number(item.extra_charge) || 0;

  if (qty > 0) {
    item.subtotal = (qty * price) + extra;
    item.purchase_unit_cost = item.subtotal / qty;
  } else {
    item.subtotal = extra;
    item.purchase_unit_cost = 0;
  }
}

function updateQuantity(item, delta) {
  const newQty = item.quantity + delta;
  if (newQty > 0) {
    item.quantity = newQty;
    recalculateItem(item);
  }
}

function updatePrice(item) {
  if (item.buying_price < 0) item.buying_price = 0;
  recalculateItem(item);
}

function updateExtraCharge(item) {
  if (item.extra_charge < 0) item.extra_charge = 0;
  recalculateItem(item);
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
      extra_charge: Number(item.extra_charge),
      subtotal: Number(item.subtotal),
      purchase_unit_cost: Number(item.purchase_unit_cost)
    }));

    await invoke('create_purchase', { purchase: purchaseData, items: itemsData });
    await logActivity('CREATE', 'Purchase', null, `New purchase from ${form.supplier_name || 'Unknown'} — ${cart.value.length} items, Total: ${totalAmount.value}`);

    // Reset
    cart.value = [];
    form.supplier_name = "";
    form.supplier_phone = "";
    form.invoice_number = "";
    form.purchase_date = new Date().toISOString().split('T')[0];
    form.notes = "";
    loadProducts();
    alert("Buying entry saved successfully! Stock and Weighted Average Price updated.");
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
    await logActivity('DELETE', 'Purchase', purchase.purchase_id, `Deleted purchase #${purchase.purchase_id} from ${purchase.supplier_name || 'Unknown'}`);
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
  <div class="h-full flex flex-col font-sans">
    <!-- Header Toggle -->
    <div class="flex flex-col sm:flex-row justify-between items-start sm:items-center mb-6 gap-3 px-1">
      <h1 class="text-2xl md:text-3xl font-extrabold text-gray-900 tracking-tight">
        {{ viewMode === 'new' ? 'Product Procurement' : 'Procurement History' }}
      </h1>
      <div class="bg-gray-100 p-1 rounded-xl flex text-sm font-bold shadow-inner">
        <button @click="viewMode = 'new'"
          :class="{ 'bg-white shadow-sm text-blue-600': viewMode === 'new', 'text-gray-500 hover:text-gray-700': viewMode !== 'new' }"
          class="px-6 py-2 rounded-lg transition-all active:scale-95">
          New Purchase
        </button>
        <button @click="viewMode = 'history'"
          :class="{ 'bg-white shadow-sm text-blue-600': viewMode === 'history', 'text-gray-500 hover:text-gray-700': viewMode !== 'history' }"
          class="px-6 py-2 rounded-lg transition-all active:scale-95">
          History
        </button>
      </div>
    </div>

    <!-- NEW BUYING VIEW -->
    <div v-if="viewMode === 'new'" class="flex flex-col xl:flex-row flex-1 gap-6 overflow-hidden">

      <!-- Left: Products Catalog -->
      <div class="flex-1 flex flex-col bg-white rounded-2xl shadow-sm border border-gray-100 overflow-hidden min-h-0">
        <div class="p-4 border-b border-gray-50 bg-gray-50/30">
          <div class="relative">
            <input v-model="searchQuery" type="text" placeholder="Search by name or code..."
              class="w-full border border-gray-200 rounded-xl px-4 py-3 pl-10 focus:ring-2 focus:ring-blue-500 focus:border-transparent focus:outline-none text-sm transition-all shadow-sm">
            <span class="absolute left-3 top-3.5 text-gray-400">
              <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24"
                stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                  d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
              </svg>
            </span>
          </div>
        </div>

        <div class="p-4 grid grid-cols-2 sm:grid-cols-3 xl:grid-cols-4 gap-4 overflow-y-auto content-start flex-1">
          <div v-for="product in filteredProducts" :key="product.id" class="relative group/card">
            <div @click="addToCart(product)"
              class="border border-gray-100 rounded-2xl p-4 cursor-pointer hover:shadow-xl transition-all bg-white hover:-translate-y-1 active:scale-95 transform h-full flex flex-col">
              <div
                class="h-28 bg-gray-50 rounded-xl mb-3 flex items-center justify-center text-gray-300 text-3xl font-black overflow-hidden border border-gray-50 shadow-inner">
                <img v-if="product._thumb" :src="product._thumb" class="w-full h-full object-cover">
                <span v-else>{{ product.product_name.charAt(0) }}</span>
              </div>
              <h3 class="font-extrabold text-gray-900 text-sm truncate mb-1 text-left">{{ product.product_name }}</h3>
              <div class="flex justify-between items-center mt-auto pt-2 border-t border-gray-50">
                <div class="flex flex-col">
                  <span class="text-[10px] uppercase text-gray-400 font-black tracking-widest">Avg Cost</span>
                  <span class="text-blue-600 font-black text-sm">{{ currencySymbol }}{{ (product.buying_price ||
                    0).toFixed(2) }}</span>
                </div>
                <div class="text-right">
                  <span
                    class="text-[10px] uppercase text-gray-400 font-black tracking-widest block text-left">Stock</span>
                  <span
                    class="text-xs px-2 py-0.5 rounded-lg bg-gray-100 text-gray-700 font-bold border border-gray-200">
                    {{ product.stock_quantity }}
                  </span>
                </div>
              </div>
            </div>
            <!-- Details Button -->
            <button @click.stop="openDetails(product)"
              class="absolute top-3 right-3 w-8 h-8 bg-white/90 backdrop-blur rounded-full shadow-lg border border-gray-100 flex items-center justify-center text-blue-600 opacity-0 group-hover/card:opacity-100 hover:bg-blue-600 hover:text-white transition-all z-10"
              title="View Product Details">
              <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24"
                stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                  d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
            </button>
          </div>
          <div v-if="filteredProducts.length === 0" class="col-span-full text-center text-gray-400 py-16">
            <p class="text-lg italic font-medium">No products found for "{{ searchQuery }}"</p>
          </div>
        </div>
      </div>

      <!-- Right: Buying Cart & Details -->
      <div
        class="w-full lg:w-[450px] flex flex-col bg-white rounded-2xl shadow-lg border border-gray-100 overflow-hidden flex-shrink-0 max-h-[60vh] lg:max-h-full">
        <div
          class="p-4 border-b bg-gray-900 text-white font-black text-xs uppercase tracking-widest flex justify-between items-center">
          <span>Purchase Cart</span>
          <span class="bg-white/20 px-2 py-1 rounded-lg text-[10px]">{{ cart.length }} Items</span>
        </div>

        <!-- Supplier & Invoice Info -->
        <div class="p-4 border-b bg-gray-50/50 space-y-4 text-left">
          <div class="grid grid-cols-2 gap-4">
            <div>
              <label class="block text-[10px] font-black text-gray-400 uppercase tracking-widest mb-1.5">Supplier
                Name</label>
              <input v-model="form.supplier_name" type="text" placeholder="John Doe / Acme Corp"
                class="w-full border border-gray-200 rounded-xl px-3 py-2 text-sm focus:ring-2 focus:ring-blue-500 focus:outline-none transition-all shadow-sm">
            </div>
            <div>
              <label class="block text-[10px] font-black text-gray-400 uppercase tracking-widest mb-1.5">Invoice
                Number</label>
              <input v-model="form.invoice_number" type="text" placeholder="PUR-2024-001"
                class="w-full border border-gray-200 rounded-xl px-3 py-2 text-sm focus:ring-2 focus:ring-blue-500 focus:outline-none transition-all shadow-sm">
            </div>
          </div>
          <div class="grid grid-cols-2 gap-4">
            <div>
              <label class="block text-[10px] font-black text-gray-400 uppercase tracking-widest mb-1.5">Phone
                Number</label>
              <input v-model="form.supplier_phone" type="text" placeholder="+880..."
                class="w-full border border-gray-200 rounded-xl px-3 py-2 text-sm focus:ring-2 focus:ring-blue-500 focus:outline-none transition-all shadow-sm">
            </div>
            <div>
              <label class="block text-[10px] font-black text-gray-400 uppercase tracking-widest mb-1.5">Purchase
                Date</label>
              <input v-model="form.purchase_date" type="date"
                class="w-full border border-gray-200 rounded-xl px-3 py-2 text-sm focus:ring-2 focus:ring-blue-500 focus:outline-none transition-all shadow-sm">
            </div>
          </div>
        </div>

        <!-- Cart Items -->
        <div class="flex-1 overflow-y-auto p-4 space-y-4 bg-white min-h-0">
          <div v-for="(item, index) in cart" :key="item.product_id"
            class="border border-gray-100 rounded-2xl p-4 group bg-gray-50/30 hover:bg-white hover:border-blue-100 hover:shadow-md transition-all">
            <div class="flex justify-between items-start mb-3">
              <div class="flex items-center gap-3 flex-1 min-w-0">
                <div
                  class="w-10 h-10 rounded-xl bg-white border border-gray-100 flex-shrink-0 flex items-center justify-center text-xs text-gray-400 font-black overflow-hidden shadow-sm">
                  <img v-if="item._thumb" :src="item._thumb" class="w-full h-full object-cover">
                  <span v-else>{{ item.product_name.charAt(0) }}</span>
                </div>
                <div class="font-extrabold text-gray-900 text-sm truncate text-left">{{ item.product_name }}</div>
              </div>
              <button @click="removeFromCart(index)"
                class="text-gray-300 hover:text-red-500 transition-colors ml-2 text-xl leading-none font-bold">✕</button>
            </div>

            <div class="grid grid-cols-3 gap-3">
              <div class="flex flex-col gap-1 text-left">
                <span class="text-[9px] uppercase text-gray-400 font-black">Quantity</span>
                <div class="flex items-center border border-gray-200 rounded-lg bg-white overflow-hidden shadow-sm">
                  <button @click="updateQuantity(item, -1)"
                    class="px-2 py-1 text-gray-400 hover:bg-gray-50 hover:text-gray-900 transition-all">-</button>
                  <span class="flex-1 font-black text-xs min-w-[32px] text-center text-gray-700 font-mono">{{
                    item.quantity }}</span>
                  <button @click="updateQuantity(item, 1)"
                    class="px-2 py-1 text-gray-400 hover:bg-gray-50 hover:text-gray-900 transition-all">+</button>
                </div>
              </div>

              <div class="flex flex-col gap-1 text-left">
                <span class="text-[9px] uppercase text-gray-400 font-black">Unit Price</span>
                <div class="relative">
                  <input type="number" v-model.number="item.buying_price" @input="updatePrice(item)"
                    class="w-full border border-gray-200 rounded-lg pl-5 pr-2 py-1 font-mono text-xs focus:ring-2 focus:ring-blue-500 focus:outline-none shadow-sm"
                    step="0.01" min="0">
                  <span class="absolute left-1.5 top-1.5 text-[10px] text-gray-400">{{ currencySymbol }}</span>
                </div>
              </div>

              <div class="flex flex-col gap-1 text-left">
                <span class="text-[9px] uppercase text-gray-400 font-black">Extra Charge</span>
                <div class="relative">
                  <input type="number" v-model.number="item.extra_charge" @input="updateExtraCharge(item)"
                    class="w-full border border-gray-200 rounded-lg pl-5 pr-2 py-1 font-mono text-xs focus:ring-2 focus:ring-blue-500 focus:outline-none shadow-sm border-amber-100"
                    step="0.01" min="0">
                  <span class="absolute left-1.5 top-1.5 text-[10px] text-gray-400">{{ currencySymbol }}</span>
                </div>
              </div>
            </div>

            <div
              class="mt-4 pt-3 border-t border-dashed border-gray-200 flex justify-between items-center bg-white/50 px-2 py-2 rounded-xl border border-gray-50 shadow-inner">
              <div class="flex flex-col text-left">
                <span class="text-[9px] uppercase text-amber-500 font-black tracking-widest">Landed Cost/Unit</span>
                <span class="text-xs font-black text-amber-600 font-mono">{{ currencySymbol }}{{
                  (item.purchase_unit_cost || 0).toFixed(2) }}</span>
              </div>
              <div class="text-right">
                <span class="text-[9px] uppercase text-gray-400 font-black tracking-widest block">Line Total</span>
                <span class="font-black text-sm text-gray-900 font-mono">{{ currencySymbol }}{{ (item.subtotal ||
                  0).toFixed(2) }}</span>
              </div>
            </div>
          </div>

          <div v-if="cart.length === 0" class="text-center py-16 px-4">
            <div class="w-16 h-16 bg-blue-50 rounded-full flex items-center justify-center mx-auto mb-4 text-blue-200">
              <svg xmlns="http://www.w3.org/2000/svg" class="h-8 w-8" fill="none" viewBox="0 0 24 24"
                stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                  d="M3 3h2l.4 2M7 13h10l4-8H5.4M7 13L5.4 5M7 13l-2.293 2.293c-.63.63-.184 1.707.707 1.707H17m0 0a2 2 0 100 4 2 2 0 000-4zm-8 2a2 2 0 11-4 0 2 2 0 014 0z" />
              </svg>
            </div>
            <p class="text-gray-400 text-sm font-medium">Select products from the left to start building your purchase
              order.</p>
          </div>
        </div>

        <!-- Notes -->
        <div class="p-4 border-t bg-white text-left">
          <label class="block text-[10px] font-black text-gray-400 uppercase tracking-widest mb-1.5">Purchase
            Notes</label>
          <textarea v-model="form.notes" rows="2" placeholder="Any internal notes or supplier remarks..."
            class="w-full border border-gray-200 rounded-xl px-3 py-2 text-xs resize-none focus:ring-2 focus:ring-blue-500 focus:outline-none transition-all shadow-sm"></textarea>
        </div>

        <!-- Footer -->
        <div class="p-5 bg-gradient-to-t from-gray-50 to-white border-t space-y-4">
          <div class="flex justify-between items-center text-left">
            <span class="text-xs font-black text-gray-400 uppercase tracking-widest">Grand Total</span>
            <span class="text-3xl font-black text-gray-900 font-mono">{{ currencySymbol }}{{ totalAmount.toFixed(2)
            }}</span>
          </div>
          <button @click="savePurchase" :disabled="cart.length === 0"
            class="w-full bg-blue-600 text-white py-4 rounded-2xl font-black text-sm shadow-xl hover:bg-blue-700 active:scale-[0.98] disabled:bg-gray-200 disabled:text-gray-400 disabled:shadow-none disabled:cursor-not-allowed transition-all uppercase tracking-widest">
            Confirm & Update Stock
          </button>
        </div>
      </div>
    </div>

    <!-- HISTORY VIEW -->
    <div v-else
      class="bg-white rounded-2xl shadow-sm border border-gray-100 overflow-hidden flex-1 overflow-x-auto overflow-y-auto text-left">
      <table class="w-full text-left border-collapse min-w-[800px]">
        <thead
          class="bg-gray-50 text-gray-400 uppercase text-[10px] font-black tracking-widest sticky top-0 z-10 border-b border-gray-100">
          <tr>
            <th class="p-4">Date</th>
            <th class="p-4">Supplier</th>
            <th class="p-4">Invoice Number</th>
            <th class="p-4 text-right">Grand Total</th>
            <th class="p-4">Notes</th>
            <th class="p-4 text-center">Actions</th>
          </tr>
        </thead>
        <tbody class="text-gray-700 text-sm font-medium">
          <tr v-for="purchase in purchases" :key="purchase.purchase_id"
            class="hover:bg-blue-50/30 border-b border-gray-50 transition-colors">
            <td class="p-4 text-xs font-mono text-gray-500">{{ purchase.purchase_date }}</td>
            <td class="p-4 font-extrabold text-gray-900">{{ purchase.supplier_name || 'Walk-in Vendor' }}</td>
            <td
              class="p-4 text-gray-500 text-xs font-bold bg-gray-50/50 my-2 inline-block rounded-lg px-2 border border-gray-100">
              {{ purchase.invoice_number || 'No INV' }}</td>
            <td class="p-4 text-right font-black text-gray-900 font-mono">{{ currencySymbol }}{{ (purchase.total_amount
              || 0).toFixed(2) }}</td>
            <td class="p-4 text-gray-400 text-xs italic truncate max-w-[150px]">{{ purchase.notes || '—' }}</td>
            <td class="p-4 text-center">
              <div class="flex justify-center gap-2">
                <button @click="viewPurchaseDetails(purchase)"
                  class="bg-white text-blue-600 hover:bg-blue-600 hover:text-white border border-blue-100 px-3 py-1.5 rounded-xl font-bold text-[10px] uppercase transition-all shadow-sm">Details</button>
                <button @click="deletePurchase(purchase)"
                  class="bg-white text-red-400 hover:bg-red-500 hover:text-white border border-red-50 px-3 py-1.5 rounded-xl font-bold text-[10px] uppercase transition-all shadow-sm">Delete</button>
              </div>
            </td>
          </tr>
          <tr v-if="purchases.length === 0">
            <td colspan="6" class="p-16 text-center text-gray-300 italic font-medium">No procurement records discovered.
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- Purchase Details Modal -->
    <div v-if="showDetailsModal && selectedPurchase"
      class="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-[110] p-4 font-sans animate-in fade-in duration-300">
      <div
        class="bg-white rounded-2xl shadow-2xl w-full max-w-2xl p-6 relative max-h-[90vh] flex flex-col text-left transform scale-100 transition-all border border-white/20">
        <button @click="showDetailsModal = false"
          class="absolute top-4 right-4 text-gray-400 hover:text-gray-900 transition-colors text-2xl z-10 p-1">✕</button>

        <div class="mb-6 border-b border-gray-50 pb-4">
          <h2 class="text-2xl font-black text-gray-900 mb-1">Purchase Details</h2>
          <div class="flex gap-4 items-center">
            <span class="text-xs px-2 py-0.5 rounded bg-blue-50 text-blue-600 font-bold border border-blue-100">INV: {{
              selectedPurchase.invoice_number || 'N/A' }}</span>
            <span class="text-xs text-gray-400 font-mono">{{ selectedPurchase.purchase_date }}</span>
          </div>
        </div>

        <div class="grid grid-cols-2 gap-6 mb-8 bg-gray-50/50 p-4 rounded-2xl border border-gray-100 text-xs">
          <div>
            <span
              class="block text-[9px] font-black text-gray-400 uppercase tracking-widest mb-1.5 text-left">Supplier</span>
            <span class="text-sm font-extrabold text-gray-900">{{ selectedPurchase.supplier_name || 'N/A' }}</span>
          </div>
          <div class="text-right text-left">
            <span class="block text-[9px] font-black text-gray-400 uppercase tracking-widest mb-1.5 text-left">Contact
              Info</span>
            <span class="text-sm font-extrabold text-gray-900">{{ selectedPurchase.supplier_phone || 'N/A' }}</span>
          </div>
        </div>

        <div
          class="flex-1 overflow-y-auto overflow-x-auto min-h-0 border border-gray-100 rounded-2xl shadow-inner bg-gray-50/30">
          <table class="w-full text-left text-xs border-collapse min-w-[500px]">
            <thead
              class="bg-white text-gray-400 uppercase text-[9px] font-black tracking-tighter border-b border-gray-100 sticky top-0">
              <tr>
                <th class="p-3">Product Name</th>
                <th class="p-3 text-right">Qty</th>
                <th class="p-3 text-right">Unit Price</th>
                <th class="p-3 text-right text-amber-500">Extra</th>
                <th class="p-3 text-right text-blue-600">Landed</th>
                <th class="p-3 text-right">Subtotal</th>
              </tr>
            </thead>
            <tbody class="divide-y divide-gray-50">
              <tr v-for="item in selectedPurchase.items" :key="item.id" class="hover:bg-white transition-colors">
                <td class="p-3 font-extrabold text-gray-900 text-sm">{{ item.product_name }}</td>
                <td class="p-3 text-right font-mono text-xs">{{ item.quantity }}</td>
                <td class="p-3 text-right font-mono text-xs">{{ currencySymbol }}{{ (item.buying_price || 0).toFixed(2)
                }}</td>
                <td class="p-3 text-right text-amber-600 font-mono text-xs">{{ currencySymbol }}{{ (item.extra_charge ||
                  0).toFixed(2) }}</td>
                <td class="p-3 text-right font-black text-blue-600 font-mono text-xs">{{ currencySymbol }}{{
                  (item.purchase_unit_cost || item.buying_price).toFixed(2) }}</td>
                <td class="p-3 text-right font-black text-gray-900 font-mono text-sm">{{ currencySymbol }}{{
                  (item.subtotal || 0).toFixed(2) }}</td>
              </tr>
            </tbody>
          </table>
        </div>

        <div class="mt-8 pt-4 border-t border-gray-100">
          <div v-if="selectedPurchase.notes" class="mb-4 text-left">
            <p class="text-[9px] font-black text-gray-400 uppercase tracking-widest mb-2">Purchase Notes</p>
            <p class="text-sm text-gray-600 bg-gray-50 p-4 rounded-xl italic border border-gray-100 shadow-inner">"{{
              selectedPurchase.notes }}"</p>
          </div>
          <div class="flex justify-between items-center text-left">
            <div class="flex flex-col text-left">
              <span class="text-[9px] font-black text-gray-400 uppercase tracking-widest">Total Procurement Cost</span>
              <span class="text-xs text-gray-500 italic">Incl. all extra charges</span>
            </div>
            <div class="text-3xl font-black text-gray-900 font-mono">{{ currencySymbol }}{{
              (selectedPurchase.total_amount || 0).toFixed(2) }}</div>
          </div>
        </div>

        <div class="mt-8 flex justify-end">
          <button @click="showDetailsModal = false"
            class="px-10 py-3 bg-gray-900 text-white rounded-2xl font-black text-xs uppercase tracking-widest shadow-xl hover:bg-gray-800 transition-all active:scale-95">
            Close
          </button>
        </div>
      </div>
    </div>

    <!-- Product Details Modal Component -->
    <ProductDetailsModal :show="showProductDetails" :product="selectedProductDetails" :currency-symbol="currencySymbol"
      @close="showProductDetails = false" />
  </div>
</template>
