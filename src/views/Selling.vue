<script setup>
import { ref, onMounted, computed, reactive, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import ProductDetailsModal from '../components/ProductDetailsModal.vue';
import { logActivity } from '../utils/activityLogger';

const viewMode = ref('pos');
const products = ref([]);
const orders = ref([]);
const cart = ref([]);
const searchQuery = ref("");
const selectedCategory = ref("All");

const checkoutModal = ref(false);
const showDetailsModal = ref(false);
const selectedOrder = ref(null);
const currencySymbol = ref('৳');
const showProductDetails = ref(false);
const selectedProductDetails = ref(null);

const form = reactive({
  customer_name: "Guest",
  customer_phone: "",
  customer_address: "",
  payment_method: "cash",
  delivery_charge: 0,
  details: ""
});

const categories = computed(() => {
  const cats = new Set(products.value.map(p => p.category).filter(c => c));
  return ['All', ...Array.from(cats)];
});

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

// Cart calculations
const subtotal = computed(() => cart.value.reduce((sum, item) => sum + item.subtotal, 0));
// Auto-calculated discount: difference between default selling price total and actual selling price total
const autoDiscount = computed(() => {
  return cart.value.reduce((sum, item) => {
    const defaultTotal = item.default_selling_price * item.quantity;
    const actualTotal = item.selling_price * item.quantity;
    return sum + Math.max(0, defaultTotal - actualTotal);
  }, 0);
});
const grandTotal = computed(() => {
  return subtotal.value + form.delivery_charge;
});

async function loadProducts() {
  try {
    const [prods, settingsData] = await Promise.all([
      invoke('get_products'),
      invoke('get_settings')
    ]);
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
    if (settingsData && settingsData.currency_symbol) {
      currencySymbol.value = settingsData.currency_symbol;
    }
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

async function deleteOrder(order) {
  if (!confirm(`Are you sure you want to delete Sale #${order.order_id}? This will restore stock quantities.`)) return;
  try {
    await invoke('delete_order', { orderId: order.order_id });
    await logActivity('DELETE', 'Order', order.order_id, `Deleted sale #${order.order_id} — ${order.customer_name || 'Guest'}`);
    loadOrders();
  } catch (e) {
    console.error("Failed to delete order", e);
    alert("Failed to delete order: " + e);
  }
}

function openDetails(product) {
  selectedProductDetails.value = product;
  showProductDetails.value = true;
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
      _thumb: product._thumb,
      quantity: 1,
      selling_price: product.default_selling_price,
      default_selling_price: product.default_selling_price,
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

function updatePrice(item) {
  if (item.selling_price < 0) item.selling_price = 0;
  item.subtotal = item.quantity * item.selling_price;
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
      order_date: new Date().toISOString(),
      order_type: "local",
      customer_name: form.customer_name,
      customer_phone: form.customer_phone,
      customer_address: form.customer_address,
      subtotal: subtotal.value,
      extra_charge: 0,
      delivery_charge: form.delivery_charge,
      discount: autoDiscount.value,
      grand_total: grandTotal.value,
      payment_method: form.payment_method,
      notes: form.details
    };

    const itemsData = cart.value.map(item => ({
      product_id: item.product_id,
      quantity: Number(item.quantity),
      selling_price: Number(item.selling_price),
      subtotal: Number(item.subtotal),
      buying_price_snapshot: null
    }));

    await invoke('create_order', { order: orderData, items: itemsData });
    await logActivity('CREATE', 'Order', null, `New sale to ${form.customer_name || 'Guest'} — ${cart.value.length} items, Total: ${grandTotal.value}`);

    cart.value = [];
    checkoutModal.value = false;
    form.customer_name = "Guest";
    form.customer_phone = "";
    form.delivery_charge = 0;
    loadProducts();
    alert("Sale completed successfully!");
  } catch (error) {
    console.error("Order failed:", error);
    alert("Sale failed: " + error);
  }
}

watch(viewMode, (newMode) => {
  if (newMode === 'history') loadOrders();
  if (newMode === 'pos') loadProducts();
});

onMounted(() => {
  loadProducts();
  // Also load settings for currency
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
        {{ viewMode === 'pos' ? 'Selling' : 'Sales History' }}
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
    <div v-if="viewMode === 'pos'" class="flex flex-col xl:flex-row flex-1 gap-4 overflow-hidden">

      <!-- Left: Products -->
      <div class="flex-1 flex flex-col bg-white rounded-lg shadow overflow-hidden min-h-0">
        <div class="p-3 border-b border-gray-100 flex flex-col sm:flex-row gap-3">
          <input v-model="searchQuery" type="text" placeholder="Search products..."
            class="flex-1 border border-gray-300 rounded-lg px-3 py-2 focus:ring-blue-500 focus:outline-none text-sm">
          <select v-model="selectedCategory"
            class="border border-gray-300 rounded-lg px-3 py-2 focus:ring-blue-500 focus:outline-none bg-white text-sm">
            <option v-for="cat in categories" :key="cat" :value="cat">{{ cat }}</option>
          </select>
        </div>

        <div class="p-3 grid grid-cols-2 sm:grid-cols-3 xl:grid-cols-4 gap-3 overflow-y-auto content-start flex-1">
          <div v-for="product in filteredProducts" :key="product.id" class="relative group/card">
            <div @click="addToCart(product)"
              class="border border-gray-200 rounded-xl p-3 cursor-pointer hover:shadow-md transition-shadow bg-gray-50 hover:bg-white active:scale-95 transform transition-transform h-full"
              :class="{ 'opacity-50 pointer-events-none': product.stock_quantity <= 0 }">
              <div
                class="h-16 bg-gray-200 rounded-lg mb-2 flex items-center justify-center text-gray-400 text-2xl font-bold overflow-hidden border border-gray-100">
                <img v-if="product._thumb" :src="product._thumb" class="w-full h-full object-cover">
                <span v-else>{{ product.product_name.charAt(0) }}</span>
              </div>
              <h3 class="font-bold text-gray-800 text-xs truncate">{{ product.product_name }}</h3>
              <div class="flex justify-between items-center mt-1">
                <span class="text-blue-600 font-bold text-sm">{{ currencySymbol }}{{
                  product.default_selling_price.toFixed(2) }}</span>
                <span class="text-xs px-1.5 py-0.5 rounded-full"
                  :class="product.stock_quantity > 0 ? 'bg-green-100 text-green-700' : 'bg-red-100 text-red-700'">
                  {{ product.stock_quantity }}
                </span>
              </div>
            </div>
            <!-- Details Button -->
            <button @click.stop="openDetails(product)"
              class="absolute top-2 right-2 w-7 h-7 bg-white/90 backdrop-blur rounded-full shadow border border-gray-100 flex items-center justify-center text-blue-600 opacity-0 group-hover/card:opacity-100 hover:bg-blue-600 hover:text-white transition-all z-10"
              title="View Product Details">
              <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24"
                stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                  d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
            </button>
          </div>
          <div v-if="filteredProducts.length === 0" class="col-span-full text-center text-gray-400 py-8">
            No products found.
          </div>
        </div>
      </div>

      <!-- Right: Cart -->
      <div
        class="w-full lg:w-96 flex flex-col bg-white rounded-lg shadow overflow-hidden flex-shrink-0 max-h-[50vh] lg:max-h-full">
        <div class="p-3 border-b bg-gray-50 font-bold text-gray-700 text-sm">Cart ({{ cart.length }} items)</div>

        <div class="flex-1 overflow-y-auto p-3 space-y-3">
          <div v-for="(item, index) in cart" :key="item.product_id" class="border border-gray-100 rounded-lg p-3 group">
            <div class="flex justify-between items-start">
              <div class="flex items-center gap-2 flex-1 min-w-0">
                <div
                  class="w-8 h-8 rounded bg-gray-50 border border-gray-100 flex-shrink-0 flex items-center justify-center text-[10px] text-gray-400 font-bold overflow-hidden">
                  <img v-if="item._thumb" :src="item._thumb" class="w-full h-full object-cover">
                  <span v-else>{{ item.product_name.charAt(0) }}</span>
                </div>
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
                <input type="number" v-model.number="item.selling_price" @input="updatePrice(item)"
                  class="w-20 border border-gray-200 rounded px-2 py-0.5 text-xs focus:ring-blue-500 focus:outline-none"
                  step="0.01" min="0">
              </div>
              <span class="text-xs text-gray-400">=</span>
              <span class="font-bold text-sm text-gray-800">{{ currencySymbol }}{{ item.subtotal.toFixed(2) }}</span>
            </div>
            <div v-if="item.selling_price < item.default_selling_price" class="text-xs text-orange-500 mt-1">
              Discount: {{ currencySymbol }}{{ ((item.default_selling_price - item.selling_price) *
                item.quantity).toFixed(2) }}
            </div>
          </div>
          <div v-if="cart.length === 0" class="text-center text-gray-400 mt-8 text-sm">
            Click products to add to cart
          </div>
        </div>

        <div class="p-3 bg-gray-50 border-t space-y-1 text-sm">
          <div class="flex justify-between text-gray-600">
            <span>Subtotal</span>
            <span>{{ currencySymbol }}{{ subtotal.toFixed(2) }}</span>
          </div>
          <div v-if="autoDiscount > 0" class="flex justify-between text-orange-600">
            <span>Price Discount</span>
            <span>-{{ currencySymbol }}{{ autoDiscount.toFixed(2) }}</span>
          </div>
          <div class="flex justify-between text-xl font-bold text-gray-800 pt-2 border-t border-gray-200">
            <span>Total</span>
            <span>{{ currencySymbol }}{{ grandTotal.toFixed(2) }}</span>
          </div>
          <button @click="openCheckout" :disabled="cart.length === 0"
            class="w-full mt-3 bg-blue-600 text-white py-2.5 rounded-lg font-bold shadow-lg hover:bg-blue-700 disabled:bg-gray-400 disabled:cursor-not-allowed transition text-sm">
            Proceed to Checkout
          </button>
        </div>
      </div>
    </div>

    <!-- HISTORY VIEW -->
    <div v-else class="bg-white rounded-lg shadow overflow-hidden flex-1 overflow-x-auto overflow-y-auto">
      <table class="w-full text-left border-collapse min-w-[600px]">
        <thead class="bg-gray-100 text-gray-600 uppercase text-xs font-semibold sticky top-0 z-10">
          <tr>
            <th class="p-3 border-b">ID</th>
            <th class="p-3 border-b">Date</th>
            <th class="p-3 border-b">Customer</th>
            <th class="p-3 border-b text-right">Total</th>
            <th class="p-3 border-b">Payment</th>
            <th class="p-3 border-b text-center">Actions</th>
          </tr>
        </thead>
        <tbody class="text-gray-700 text-sm">
          <tr v-for="order in orders" :key="order.order_id" class="hover:bg-gray-50 border-b last:border-b-0">
            <td class="p-3 text-xs font-mono">#{{ order.order_id }}</td>
            <td class="p-3 text-xs">{{ order.order_date }}</td>
            <td class="p-3 font-medium text-sm">{{ order.customer_name || '-' }}</td>
            <td class="p-3 text-right font-bold">{{ currencySymbol }}{{ order.grand_total.toFixed(2) }}</td>
            <td class="p-3">
              <span class="px-2 py-0.5 rounded text-xs uppercase font-bold bg-gray-100 text-gray-600">
                {{ order.payment_method }}
              </span>
            </td>
            <td class="p-3 text-center">
              <div class="flex justify-center gap-1">
                <button @click="viewOrderDetails(order)"
                  class="text-blue-600 hover:text-blue-800 text-xs font-medium border border-blue-200 px-2 py-1 rounded hover:bg-blue-50">View</button>
                <button @click="deleteOrder(order)"
                  class="text-red-600 hover:text-red-800 text-xs font-medium border border-red-200 px-2 py-1 rounded hover:bg-red-50">Delete</button>
              </div>
            </td>
          </tr>
          <tr v-if="orders.length === 0">
            <td colspan="6" class="p-8 text-center text-gray-500">No sales history found.</td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- Checkout Modal -->
    <div v-if="checkoutModal" class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4">
      <div class="bg-white rounded-xl shadow-2xl w-full max-w-lg p-5 relative">
        <button @click="checkoutModal = false"
          class="absolute top-3 right-3 text-gray-500 hover:text-gray-700 text-xl">✕</button>
        <h2 class="text-xl font-bold mb-4 text-gray-800">Checkout</h2>

        <div class="space-y-3">
          <div>
            <label class="block text-sm font-medium text-gray-700">Customer Name</label>
            <input v-model="form.customer_name" type="text"
              class="mt-1 w-full border border-gray-300 rounded-lg px-3 py-2 text-sm">
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-700">Phone</label>
            <input v-model="form.customer_phone" type="text"
              class="mt-1 w-full border border-gray-300 rounded-lg px-3 py-2 text-sm">
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-700">Payment Method</label>
            <select v-model="form.payment_method"
              class="mt-1 w-full border border-gray-300 rounded-lg px-3 py-2 bg-white text-sm">
              <option value="cash">Cash</option>
              <option value="card">Card</option>
              <option value="mobile">Mobile Banking</option>
            </select>
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-700">Delivery Charge</label>
            <input v-model.number="form.delivery_charge" type="number"
              class="mt-1 w-full border border-gray-300 rounded-lg px-3 py-2 text-sm">
          </div>

          <div class="pt-3 border-t space-y-1 text-sm">
            <div class="flex justify-between text-gray-600">
              <span>Subtotal</span>
              <span>{{ currencySymbol }}{{ subtotal.toFixed(2) }}</span>
            </div>
            <div v-if="autoDiscount > 0" class="flex justify-between text-orange-600">
              <span>Price Discount</span>
              <span>-{{ currencySymbol }}{{ autoDiscount.toFixed(2) }}</span>
            </div>
            <div v-if="form.delivery_charge > 0" class="flex justify-between text-gray-600">
              <span>Delivery</span>
              <span>+{{ currencySymbol }}{{ form.delivery_charge.toFixed(2) }}</span>
            </div>
            <div class="flex justify-between items-center text-xl font-bold text-gray-800 pt-2 border-t">
              <span>Grand Total</span>
              <span>{{ currencySymbol }}{{ grandTotal.toFixed(2) }}</span>
            </div>
          </div>
        </div>

        <div class="mt-5 flex justify-end space-x-3">
          <button @click="checkoutModal = false"
            class="px-4 py-2 text-gray-600 hover:bg-gray-100 rounded-lg text-sm">Cancel</button>
          <button @click="processOrder"
            class="px-5 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 font-bold text-sm">
            Confirm Payment
          </button>
        </div>
      </div>
    </div>

    <!-- Order Details Modal -->
    <div v-if="showDetailsModal && selectedOrder"
      class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4">
      <div class="bg-white rounded-xl shadow-2xl w-full max-w-2xl p-5 relative max-h-[90vh] flex flex-col">
        <button @click="showDetailsModal = false"
          class="absolute top-3 right-3 text-gray-500 hover:text-gray-700 text-xl">✕</button>
        <h2 class="text-xl font-bold mb-1 text-gray-800">Sale Details</h2>
        <div class="text-xs text-gray-500 mb-4">Sale #{{ selectedOrder.order_id }} | {{ selectedOrder.order_date }}
        </div>

        <div class="grid grid-cols-2 gap-3 mb-4 bg-gray-50 p-3 rounded-lg text-sm">
          <div>
            <span class="block text-xs text-gray-500 uppercase">Customer</span>
            <span class="font-medium text-gray-800">{{ selectedOrder.customer_name || 'Guest' }}</span>
            <div class="text-xs text-gray-600">{{ selectedOrder.customer_phone }}</div>
          </div>
          <div class="text-right">
            <span class="block text-xs text-gray-500 uppercase">Payment</span>
            <span class="font-medium text-gray-800 uppercase">{{ selectedOrder.payment_method }}</span>
          </div>
        </div>

        <div class="flex-1 overflow-y-auto overflow-x-auto">
          <table class="w-full text-left text-sm border-collapse min-w-[400px]">
            <thead class="bg-gray-100 text-gray-600">
              <tr>
                <th class="p-2 border-b text-xs">Product</th>
                <th class="p-2 border-b text-right text-xs">Qty</th>
                <th class="p-2 border-b text-right text-xs">Price</th>
                <th class="p-2 border-b text-right text-xs">Subtotal</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="item in selectedOrder.items" :key="item.id" class="border-b last:border-0 hover:bg-gray-50">
                <td class="p-2 font-medium text-sm">{{ item.product_name }}</td>
                <td class="p-2 text-right text-sm">{{ item.quantity }}</td>
                <td class="p-2 text-right text-sm">{{ currencySymbol }}{{ item.selling_price.toFixed(2) }}</td>
                <td class="p-2 text-right font-medium text-sm">{{ currencySymbol }}{{ item.subtotal.toFixed(2) }}</td>
              </tr>
            </tbody>
          </table>
        </div>

        <div class="border-t mt-3 pt-3 space-y-1 text-sm">
          <div class="flex justify-between py-0.5">
            <span class="text-gray-600">Subtotal</span>
            <span class="font-medium">{{ currencySymbol }}{{ selectedOrder.subtotal.toFixed(2) }}</span>
          </div>
          <div class="flex justify-between py-0.5" v-if="selectedOrder.discount > 0">
            <span class="text-gray-600">Discount</span>
            <span class="text-red-500">-{{ currencySymbol }}{{ selectedOrder.discount.toFixed(2) }}</span>
          </div>
          <div class="flex justify-between py-0.5" v-if="selectedOrder.delivery_charge > 0">
            <span class="text-gray-600">Delivery</span>
            <span class="font-medium">{{ currencySymbol }}{{ selectedOrder.delivery_charge.toFixed(2) }}</span>
          </div>
          <div class="flex justify-between items-end mt-1 pt-2 border-t border-dashed">
            <div class="text-xs text-gray-500 uppercase">Grand Total</div>
            <div class="text-2xl font-bold text-gray-800">{{ currencySymbol }}{{ selectedOrder.grand_total.toFixed(2) }}
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Product Details Modal -->
    <ProductDetailsModal :show="showProductDetails" :product="selectedProductDetails" :currency-symbol="currencySymbol"
      @close="showProductDetails = false" />
  </div>
</template>
