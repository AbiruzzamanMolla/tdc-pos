<script setup>
import { ref, onMounted, computed, reactive } from 'vue';
import { invoke } from '@tauri-apps/api/core';

const purchases = ref([]);
const products = ref([]);
const showModal = ref(false);
const searchQuery = ref("");

// Form Data
const form = reactive({
  supplier_name: "",
  supplier_phone: "",
  invoice_number: "",
  purchase_date: new Date().toISOString().split('T')[0],
  notes: "",
  items: [] // { product_id, product_name, quantity, buying_price, subtotal }
});

// Product Selection in Modal
const productSearch = ref("");
const filteredProducts = computed(() => {
  if (!productSearch.value) return [];
  const query = productSearch.value.toLowerCase();
  return products.value.filter(p =>
    p.product_name.toLowerCase().includes(query) ||
    (p.product_code && p.product_code.toLowerCase().includes(query))
  );
});

const totalAmount = computed(() => {
  return form.items.reduce((sum, item) => sum + item.subtotal, 0);
});

async function loadPurchases() {
  try {
    purchases.value = await invoke('get_purchases');
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

function openModal() {
  form.supplier_name = "";
  form.supplier_phone = "";
  form.invoice_number = "";
  form.purchase_date = new Date().toISOString().split('T')[0];
  form.notes = "";
  form.items = [];
  productSearch.value = "";
  showModal.value = true;
  loadProducts();
}

function closeModal() {
  showModal.value = false;
}

function addProductToPurchase(product) {
  // Check if already added
  const existing = form.items.find(i => i.product_id === product.id);
  if (existing) {
    alert("Product already added!");
    return;
  }

  form.items.push({
    product_id: product.id,
    product_name: product.product_name,
    quantity: 1,
    buying_price: product.buying_price,
    subtotal: product.buying_price // 1 * price
  });
  productSearch.value = ""; // Clear search
}

function updateItemTotal(item) {
  item.subtotal = item.quantity * item.buying_price;
}

function removeItem(index) {
  form.items.splice(index, 1);
}

async function savePurchase() {
  if (form.items.length === 0) {
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

    const itemsData = form.items.map(item => ({
      product_id: item.product_id,
      quantity: Number(item.quantity),
      buying_price: Number(item.buying_price),
      subtotal: Number(item.subtotal)
    }));

    await invoke('create_purchase', { purchase: purchaseData, items: itemsData });
    closeModal();
    loadPurchases();
  } catch (error) {
    console.error("Failed to save purchase:", error);
    alert("Error saving purchase: " + error);
  }
}

onMounted(() => {
  loadPurchases();
});
</script>

<template>
  <div class="h-full flex flex-col space-y-6">
    <div class="flex justify-between items-center">
      <h1 class="text-3xl font-bold text-gray-800">Purchases</h1>
      <button @click="openModal"
        class="bg-blue-600 hover:bg-blue-700 text-white px-4 py-2 rounded-lg shadow transition">
        + New Purchase
      </button>
    </div>

    <!-- List -->
    <div class="bg-white rounded-lg shadow overflow-hidden flex-1 overflow-y-auto">
      <table class="w-full text-left border-collapse">
        <thead class="bg-gray-100 text-gray-600 uppercase text-sm font-semibold">
          <tr>
            <th class="p-4 border-b">Date</th>
            <th class="p-4 border-b">Supplier</th>
            <th class="p-4 border-b">Invoice #</th>
            <th class="p-4 border-b text-right">Total Amount</th>
            <th class="p-4 border-b">Notes</th>
          </tr>
        </thead>
        <tbody class="text-gray-700">
          <tr v-for="purchase in purchases" :key="purchase.purchase_id"
            class="hover:bg-gray-50 border-b last:border-b-0">
            <td class="p-4">{{ purchase.purchase_date }}</td>
            <td class="p-4 font-medium">{{ purchase.supplier_name || '-' }}</td>
            <td class="p-4 text-gray-500 text-sm">{{ purchase.invoice_number || '-' }}</td>
            <td class="p-4 text-right font-bold">{{ purchase.total_amount.toFixed(2) }}</td>
            <td class="p-4 text-gray-500 text-sm truncate max-w-xs">{{ purchase.notes }}</td>
          </tr>
          <tr v-if="purchases.length === 0">
            <td colspan="5" class="p-8 text-center text-gray-500">No purchase history found.</td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- Modal -->
    <div v-if="showModal" class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div class="bg-white rounded-xl shadow-2xl w-full max-w-4xl p-6 relative flex flex-col h-[90vh]">
        <button @click="closeModal" class="absolute top-4 right-4 text-gray-500 hover:text-gray-700 text-xl">✕</button>
        <h2 class="text-2xl font-bold mb-4 text-gray-800">New Purchase (Inward Stock)</h2>

        <div class="grid grid-cols-3 gap-4 mb-6">
          <div>
            <label class="block text-sm font-medium text-gray-700">Supplier Name</label>
            <input v-model="form.supplier_name" type="text"
              class="mt-1 w-full border border-gray-300 rounded-lg px-3 py-2 focus:ring-blue-500 focus:outline-none">
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-700">Invoice Number</label>
            <input v-model="form.invoice_number" type="text"
              class="mt-1 w-full border border-gray-300 rounded-lg px-3 py-2 focus:ring-blue-500 focus:outline-none">
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-700">Date</label>
            <input v-model="form.purchase_date" type="date"
              class="mt-1 w-full border border-gray-300 rounded-lg px-3 py-2 focus:ring-blue-500 focus:outline-none">
          </div>
        </div>

        <!-- Product Search & Add -->
        <div class="mb-4 relative z-10">
          <label class="block text-sm font-medium text-gray-700 mb-1">Search & Add Product</label>
          <input v-model="productSearch" type="text" placeholder="Type name or code..."
            class="w-full border border-gray-300 rounded-lg px-4 py-2 focus:ring-blue-500 focus:outline-none">

          <!-- Dropdown Results -->
          <div v-if="filteredProducts.length > 0"
            class="absolute top-full left-0 w-full bg-white border border-gray-200 shadow-lg rounded-b-lg max-h-48 overflow-y-auto mt-1">
            <div v-for="product in filteredProducts" :key="product.id" @click="addProductToPurchase(product)"
              class="px-4 py-2 hover:bg-gray-100 cursor-pointer flex justify-between">
              <span>{{ product.product_name }}</span>
              <span class="text-gray-500 text-sm">{{ product.product_code }}</span>
            </div>
          </div>
        </div>

        <!-- Items Table -->
        <div class="flex-1 overflow-y-auto border border-gray-200 rounded-lg mb-4">
          <table class="w-full text-left text-sm">
            <thead class="bg-gray-50 text-gray-600 sticky top-0">
              <tr>
                <th class="p-3">Product</th>
                <th class="p-3 w-24">Qty</th>
                <th class="p-3 w-32">Buy Price</th>
                <th class="p-3 w-32 text-right">Subtotal</th>
                <th class="p-3 w-10"></th>
              </tr>
            </thead>
            <tbody class="divide-y divide-gray-100">
              <tr v-for="(item, index) in form.items" :key="item.product_id">
                <td class="p-3 font-medium">{{ item.product_name }}</td>
                <td class="p-3">
                  <input v-model.number="item.quantity" @input="updateItemTotal(item)" type="number" min="1"
                    class="w-full border rounded px-2 py-1 text-center">
                </td>
                <td class="p-3">
                  <input v-model.number="item.buying_price" @input="updateItemTotal(item)" type="number" step="0.01"
                    class="w-full border rounded px-2 py-1 text-right">
                </td>
                <td class="p-3 text-right font-medium">
                  {{ item.subtotal.toFixed(2) }}
                </td>
                <td class="p-3 text-center">
                  <button @click="removeItem(index)" class="text-red-500 hover:text-red-700 font-bold">×</button>
                </td>
              </tr>
              <tr v-if="form.items.length === 0">
                <td colspan="5" class="p-6 text-center text-gray-400">No items added. Search products above.</td>
              </tr>
            </tbody>
          </table>
        </div>

        <!-- Footer Totals -->
        <div class="flex justify-between items-end border-t pt-4">
          <div class="w-1/2">
            <label class="block text-sm font-medium text-gray-700">Notes / Remarks</label>
            <textarea v-model="form.notes" rows="2"
              class="w-full mt-1 border border-gray-300 rounded-lg px-3 py-2 text-sm max-h-20"></textarea>
          </div>
          <div class="text-right">
            <div class="text-sm text-gray-500 mb-1">Total Amount</div>
            <div class="text-3xl font-bold text-gray-800">{{ totalAmount.toFixed(2) }}</div>
            <div class="mt-4 space-x-3">
              <button @click="closeModal"
                class="px-6 py-2 border border-gray-300 rounded-lg text-gray-700 hover:bg-gray-50">Cancel</button>
              <button @click="savePurchase"
                class="px-6 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 font-medium">Save
                Purchase</button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
