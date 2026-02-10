<script setup>
import { ref, onMounted, computed, reactive, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';

const viewMode = ref('pos'); // 'pos' or 'history'
const products = ref([]);
const orders = ref([]);
const cart = ref([]);
const searchQuery = ref("");
const selectedCategory = ref("All");

// Checkout Form
const checkoutModal = ref(false);
const showDetailsModal = ref(false);
const selectedOrder = ref(null);
const currencySymbol = ref('$'); // Default

const form = reactive({
  customer_name: "Guest",
  customer_phone: "",
  customer_address: "",
  payment_method: "cash",
  discount: 0,
  delivery_charge: 0,
  details: ""
});

// Categories
const categories = computed(() => {
  const cats = new Set(products.value.map(p => p.category).filter(c => c));
  return ['All', ...Array.from(cats)];
});

// Filtered Products
const filteredProducts = computed(() => {
  let result = products.value;
  if (selectedCategory.value !== 'All') {
    result = result.filter(p => p.category === selectedCategory.value);
  }
  if (searchQuery.value) {
    const query = searchQuery.value.toLowerCase();
    result = result.filter(p =>
      p.product_name.toLowerCase().includes(query) ||
      (p.product_code && p.product_code.toLowerCase().includes(query))
    );
  }
  return result;
});

// Cart Calculations
const subtotal = computed(() => cart.value.reduce((sum, item) => sum + item.subtotal, 0));
const grandTotal = computed(() => {
  return subtotal.value + form.delivery_charge - form.discount;
});

async function loadProducts() {
  try {
    products.value = await invoke('get_products');
  } catch (error) {
    console.error("Failed to load products:", error);
  }
}

async function loadOrders() {
  try {
    const [ordersData, settingsData] = await Promise.all([
      invoke('get_orders'),
      invoke('get_settings')
    ]);
    orders.value = ordersData;
    if (settingsData && settingsData.currency_symbol) {
      currencySymbol.value = settingsData.currency_symbol;
    }
  } catch (error) {
    console.error("Failed to load orders:", error);
  }
}

async function viewOrderDetails(order) {
  selectedOrder.value = order;
  try {
    const items = await invoke('get_order_items', { orderId: order.order_id });
    selectedOrder.value = { ...order, items: items };
    showDetailsModal.value = true;
  } catch (e) {
    console.error("Failed to load order items", e);
    alert("Failed to load details");
  }
}

function addToCart(product) {
  if (product.stock_quantity <= 0) {
    alert("Out of stock!");
    return;
  }

  const existing = cart.value.find(i => i.product_id === product.id);
  if (existing) {
    if (existing.quantity >= product.stock_quantity) {
      alert("Not enough stock!");
      return;
    }
    existing.quantity++;
    existing.subtotal = existing.quantity * existing.selling_price;
  } else {
    cart.value.push({
      product_id: product.id,
      product_name: product.product_name,
      quantity: 1,
      selling_price: product.default_selling_price,
      subtotal: product.default_selling_price,
      max_stock: product.stock_quantity
    });
  }
}

function updateQuantity(item, delta) {
  const newQty = item.quantity + delta;
  if (newQty > 0 && newQty <= item.max_stock) {
    item.quantity = newQty;
    item.subtotal = item.quantity * item.selling_price;
  }
}

function removeFromCart(index) {
  cart.value.splice(index, 1);
}

function openCheckout() {
  if (cart.value.length === 0) return;
  checkoutModal.value = true;
}

async function processOrder() {
  try {
    const orderData = {
      order_date: new Date().toISOString(), // format handled by sqlite if needed? Use ISO string
      order_type: "local",
      customer_name: form.customer_name,
      customer_phone: form.customer_phone,
      customer_address: form.customer_address,
      subtotal: subtotal.value,
      extra_charge: 0,
      delivery_charge: form.delivery_charge,
      discount: form.discount,
      grand_total: grandTotal.value,
      payment_method: form.payment_method,
      notes: form.details
    };

    const itemsData = cart.value.map(item => ({
      product_id: item.product_id,
      quantity: Number(item.quantity),
      selling_price: Number(item.selling_price),
      subtotal: Number(item.subtotal),
      buying_price_snapshot: null // backend handles
    }));

    await invoke('create_order', { order: orderData, items: itemsData });

    // Reset
    cart.value = [];
    checkoutModal.value = false;
    form.customer_name = "Guest";
    form.customer_phone = "";
    form.discount = 0;
    loadProducts(); // Refresh stock
    alert("Order completed successfully!");
  } catch (error) {
    console.error("Order failed:", error);
    alert("Order failed: " + error);
  }
}

watch(viewMode, (newMode) => {
  if (newMode === 'history') loadOrders();
  if (newMode === 'pos') loadProducts();
});

onMounted(() => {
  loadProducts();
});
</script>

<template>
  <div class="h-full flex flex-col">
    <!-- Header Toggle -->
    <div class="flex justify-between items-center mb-4 px-1">
      <h1 class="text-3xl font-bold text-gray-800">
        {{ viewMode === 'pos' ? 'Point of Sale' : 'Order History' }}
      </h1>
      <div class="bg-gray-200 p-1 rounded-lg flex text-sm font-medium">
        <button @click="viewMode = 'pos'"
          :class="{ 'bg-white shadow text-blue-600': viewMode === 'pos', 'text-gray-500 hover:text-gray-700': viewMode !== 'pos' }"
          class="px-4 py-2 rounded-md transition-all">
          New Sale
        </button>
        <button @click="viewMode = 'history'"
          :class="{ 'bg-white shadow text-blue-600': viewMode === 'history', 'text-gray-500 hover:text-gray-700': viewMode !== 'history' }"
          class="px-4 py-2 rounded-md transition-all">
          History
        </button>
      </div>
    </div>

    <!-- POS VIEW -->
    <div v-if="viewMode === 'pos'" class="flex flex-1 gap-6 overflow-hidden">

      <!-- Left: Products -->
      <div class="flex-1 flex flex-col bg-white rounded-lg shadow overflow-hidden">
        <!-- Filter Bar -->
        <div class="p-4 border-b border-gray-100 flex gap-4">
          <input v-model="searchQuery" type="text" placeholder="Search products..."
            class="flex-1 border border-gray-300 rounded-lg px-4 py-2 focus:ring-blue-500 focus:outline-none">
          <select v-model="selectedCategory"
            class="border border-gray-300 rounded-lg px-4 py-2 focus:ring-blue-500 focus:outline-none bg-white">
            <option v-for="cat in categories" :key="cat" :value="cat">{{ cat }}</option>
          </select>
        </div>

        <!-- Grid -->
        <div class="p-4 grid grid-cols-3 xl:grid-cols-4 gap-4 overflow-y-auto content-start">
          <div v-for="product in filteredProducts" :key="product.id" @click="addToCart(product)"
            class="border border-gray-200 rounded-xl p-4 cursor-pointer hover:shadow-md transition-shadow bg-gray-50 hover:bg-white active:scale-95 transform transition-transform"
            :class="{ 'opacity-50 pointer-events-none': product.stock_quantity <= 0 }">
            <div
              class="h-24 bg-gray-200 rounded-lg mb-3 flex items-center justify-center text-gray-400 text-3xl font-bold">
              {{ product.product_name.charAt(0) }}
            </div>
            <h3 class="font-bold text-gray-800 text-sm truncate">{{ product.product_name }}</h3>
            <div class="flex justify-between items-center mt-2">
              <span class="text-blue-600 font-bold">{{ currencySymbol }}{{ product.default_selling_price.toFixed(2)
                }}</span>
              <span class="text-xs px-2 py-1 rounded-full"
                :class="product.stock_quantity > 0 ? 'bg-green-100 text-green-700' : 'bg-red-100 text-red-700'">
                {{ product.stock_quantity }} left
              </span>
            </div>
          </div>
        </div>
      </div>

      <!-- Right: Cart -->
      <div class="w-96 flex flex-col bg-white rounded-lg shadow overflow-hidden border-l border-gray-100">
        <div class="p-4 border-b bg-gray-50 font-bold text-gray-700">Current Order</div>

        <div class="flex-1 overflow-y-auto p-4 space-y-4">
          <div v-for="(item, index) in cart" :key="item.product_id" class="flex justify-between items-center group">
            <div class="flex-1">
              <div class="font-medium text-gray-800">{{ item.product_name }}</div>
              <div class="text-xs text-gray-500">{{ currencySymbol }}{{ item.selling_price }} x {{ item.quantity }}
              </div>
            </div>
            <div class="flex items-center gap-3">
              <div class="flex items-center border rounded-lg">
                <button @click="updateQuantity(item, -1)" class="px-2 py-1 text-gray-600 hover:bg-gray-100">-</button>
                <span class="px-2 text-sm font-bold">{{ item.quantity }}</span>
                <button @click="updateQuantity(item, 1)" class="px-2 py-1 text-gray-600 hover:bg-gray-100">+</button>
              </div>
              <div class="font-bold w-16 text-right">{{ currencySymbol }}{{ item.subtotal.toFixed(2) }}</div>
              <button @click="removeFromCart(index)"
                class="text-red-400 hover:text-red-600 opacity-0 group-hover:opacity-100">×</button>
            </div>
          </div>
          <div v-if="cart.length === 0" class="text-center text-gray-400 mt-10">
            Cart is empty
          </div>
        </div>

        <div class="p-4 bg-gray-50 border-t space-y-2">
          <div class="flex justify-between text-gray-600">
            <span>Subtotal</span>
            <span>{{ currencySymbol }}{{ subtotal.toFixed(2) }}</span>
          </div>
          <div class="flex justify-between text-2xl font-bold text-gray-800 pt-2 border-t border-gray-200">
            <span>Total</span>
            <span>{{ currencySymbol }}{{ subtotal.toFixed(2) }}</span>
          </div>
          <button @click="openCheckout" :disabled="cart.length === 0"
            class="w-full mt-4 bg-blue-600 text-white py-3 rounded-lg font-bold shadow-lg hover:bg-blue-700 disabled:bg-gray-400 disabled:cursor-not-allowed transition">
            Proceed to Checkout
          </button>
        </div>
      </div>
    </div>

    <!-- HISTORY VIEW -->
    <div v-else class="bg-white rounded-lg shadow overflow-hidden flex-1 overflow-y-auto">
      <table class="w-full text-left border-collapse">
        <thead class="bg-gray-100 text-gray-600 uppercase text-sm font-semibold">
          <tr>
            <th class="p-4 border-b">Order ID</th>
            <th class="p-4 border-b">Date</th>
            <th class="p-4 border-b">Customer</th>
            <th class="p-4 border-b text-right">Total</th>
            <th class="p-4 border-b">Payment</th>
            <th class="p-4 border-b text-center">Actions</th>
          </tr>
        </thead>
        <tbody class="text-gray-700">
          <tr v-for="order in orders" :key="order.order_id" class="hover:bg-gray-50 border-b last:border-b-0">
            <td class="p-4 text-sm font-mono">#{{ order.order_id }}</td>
            <td class="p-4">{{ order.order_date }}</td>
            <td class="p-4 font-medium">{{ order.customer_name || '-' }}</td>
            <td class="p-4 text-right font-bold">{{ currencySymbol }}{{ order.grand_total.toFixed(2) }}</td>
            <td class="p-4">
              <span class="px-2 py-1 rounded text-xs uppercase font-bold bg-gray-100 text-gray-600">
                {{ order.payment_method }}
              </span>
            </td>
            <td class="p-4 text-center">
              <button @click="viewOrderDetails(order)"
                class="text-blue-600 hover:text-blue-800 text-sm font-medium">View</button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- Checkout Modal -->
    <div v-if="checkoutModal" class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div class="bg-white rounded-xl shadow-2xl w-full max-w-lg p-6 relative">
        <h2 class="text-2xl font-bold mb-6 text-gray-800">Checkout</h2>

        <div class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-gray-700">Customer Name</label>
            <input v-model="form.customer_name" type="text"
              class="mt-1 w-full border border-gray-300 rounded-lg px-3 py-2">
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-700">Payment Method</label>
            <select v-model="form.payment_method"
              class="mt-1 w-full border border-gray-300 rounded-lg px-3 py-2 bg-white">
              <option value="cash">Cash</option>
              <option value="card">Card</option>
              <option value="mobile">Mobile Banking</option>
            </select>
          </div>
          <div class="grid grid-cols-2 gap-4">
            <div>
              <label class="block text-sm font-medium text-gray-700">Discount</label>
              <input v-model.number="form.discount" type="number"
                class="mt-1 w-full border border-gray-300 rounded-lg px-3 py-2">
            </div>
            <div>
              <label class="block text-sm font-medium text-gray-700">Delivery Charge</label>
              <input v-model.number="form.delivery_charge" type="number"
                class="mt-1 w-full border border-gray-300 rounded-lg px-3 py-2">
            </div>
          </div>

          <div class="pt-4 border-t flex justify-between items-center text-xl font-bold text-gray-800">
            <span>Grand Total</span>
            <span>{{ currencySymbol }}{{ grandTotal.toFixed(2) }}</span>
          </div>
        </div>

        <div class="mt-8 flex justify-end space-x-3">
          <button @click="checkoutModal = false"
            class="px-4 py-2 text-gray-600 hover:bg-gray-100 rounded-lg">Cancel</button>
          <button @click="processOrder" class="px-6 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 font-bold">
            Confirm Payment
          </button>
        </div>
      </div>
    </div>

    <!-- Order Details Modal -->
    <div v-if="showDetailsModal && selectedOrder"
      class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div class="bg-white rounded-xl shadow-2xl w-full max-w-3xl p-6 relative max-h-[90vh] flex flex-col">
        <button @click="showDetailsModal = false"
          class="absolute top-4 right-4 text-gray-500 hover:text-gray-700 text-xl">✕</button>
        <h2 class="text-2xl font-bold mb-1 text-gray-800">Order Details</h2>
        <div class="text-sm text-gray-500 mb-6">Order ID: #{{ selectedOrder.order_id }} | Date: {{
          selectedOrder.order_date }}</div>

        <div class="grid grid-cols-2 gap-4 mb-6 bg-gray-50 p-4 rounded-lg">
          <div>
            <span class="block text-xs text-gray-500 uppercase">Customer</span>
            <span class="font-medium text-gray-800">{{ selectedOrder.customer_name || 'Guest' }}</span>
            <div class="text-xs text-gray-600">{{ selectedOrder.customer_phone }}</div>
            <div class="text-xs text-gray-600">{{ selectedOrder.customer_address }}</div>
          </div>
          <div class="text-right">
            <span class="block text-xs text-gray-500 uppercase">Payment Method</span>
            <span class="font-medium text-gray-800 uppercase">{{ selectedOrder.payment_method }}</span>
          </div>
        </div>

        <div class="flex-1 overflow-y-auto">
          <table class="w-full text-left text-sm border-collapse">
            <thead class="bg-gray-100 text-gray-600">
              <tr>
                <th class="p-3 border-b">Product</th>
                <th class="p-3 border-b text-right">Qty</th>
                <th class="p-3 border-b text-right">Price</th>
                <th class="p-3 border-b text-right">Subtotal</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="item in selectedOrder.items" :key="item.id" class="border-b last:border-0 hover:bg-gray-50">
                <td class="p-3 font-medium">{{ item.product_name }}</td>
                <td class="p-3 text-right">{{ item.quantity }}</td>
                <td class="p-3 text-right">{{ currencySymbol }}{{ item.selling_price.toFixed(2) }}</td>
                <td class="p-3 text-right font-medium">{{ currencySymbol }}{{ item.subtotal.toFixed(2) }}</td>
              </tr>
            </tbody>
          </table>
        </div>

        <div class="border-t mt-4 pt-4">
          <div class="flex justify-between text-sm py-1">
            <span class="text-gray-600">Subtotal</span>
            <span class="font-medium">{{ currencySymbol }}{{ selectedOrder.subtotal.toFixed(2) }}</span>
          </div>
          <div class="flex justify-between text-sm py-1" v-if="selectedOrder.discount > 0">
            <span class="text-gray-600">Discount</span>
            <span class="text-red-500">-{{ currencySymbol }}{{ selectedOrder.discount.toFixed(2) }}</span>
          </div>
          <div class="flex justify-between text-sm py-1" v-if="selectedOrder.delivery_charge > 0">
            <span class="text-gray-600">Delivery</span>
            <span class="font-medium">{{ currencySymbol }}{{ selectedOrder.delivery_charge.toFixed(2) }}</span>
          </div>

          <div class="flex justify-between items-end mt-2 pt-2 border-t border-dashed">
            <div class="text-xs text-gray-500 uppercase">Grand Total</div>
            <div class="text-2xl font-bold text-gray-800">{{ currencySymbol }}{{ selectedOrder.grand_total.toFixed(2) }}
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
