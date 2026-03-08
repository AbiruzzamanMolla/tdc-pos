<script setup>
import { ref, onMounted, computed } from 'vue';
import { onBeforeRouteLeave } from 'vue-router';
import { invoke } from '@tauri-apps/api/core';
import { open, confirm } from '@tauri-apps/plugin-dialog';
import ProductDetailsModal from '../components/ProductDetailsModal.vue';
import { logActivity } from '../utils/activityLogger';
import { useAuthStore } from '../stores/auth';

const auth = useAuthStore();


const products = ref([]);
const searchQuery = ref("");
const showModal = ref(false);
const isEditing = ref(false);
const currencySymbol = ref('৳');

const form = ref({
  id: null,
  product_name: "",
  product_code: "",
  category: "",
  brand: "",
  buying_price: 0,
  default_selling_price: 0,
  stock_quantity: 0,
  unit: "pcs",
  tax_percentage: 0,
  original_price: 0,
  profit_percentage: 0,
  facebook_link: "",
  product_link: "",
  images: [],        // raw file paths for saving
  imagesPreviews: [] // base64 data URIs for display
});

const expectedSellingPrice = computed(() => {
  const buy = Number(form.value.buying_price) || 0;
  const percent = Number(form.value.profit_percentage) || 0;
  return Number((buy + (buy * (percent / 100))).toFixed(2));
});

const showViewModal = ref(false);
const selectedProduct = ref(null);




const currentPage = ref(1);
const itemsPerPage = 12;

const filteredProducts = computed(() => {
  if (!searchQuery.value) return products.value;
  const query = searchQuery.value.toLowerCase();
  return products.value.filter(p =>
    p.product_name.toLowerCase().includes(query) ||
    (p.product_code && p.product_code.toLowerCase().includes(query))
  );
});

const totalPages = computed(() => Math.ceil(filteredProducts.value.length / itemsPerPage) || 1);
const paginatedProducts = computed(() => {
  const start = (currentPage.value - 1) * itemsPerPage;
  return filteredProducts.value.slice(start, start + itemsPerPage);
});

function goToPage(p) {
  if (p >= 1 && p <= totalPages.value) currentPage.value = p;
}

async function loadProducts() {
  try {
    const [prods, settingsData] = await Promise.all([
      invoke('get_products'),
      invoke('get_settings')
    ]);
    if (settingsData && settingsData.currency_symbol) {
      currencySymbol.value = settingsData.currency_symbol;
    }
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

async function openModal(product = null) {
  if (product) {
    isEditing.value = true;
    form.value = { ...product, images: [], imagesPreviews: [] };

    try {
      const paths = await invoke('get_product_images', { productId: product.id });
      form.value.images = paths || [];
      // Convert to base64 for preview
      const previews = [];
      for (const p of form.value.images) {
        try {
          const b64 = await invoke('read_image_base64', { path: p });
          previews.push(b64);
        } catch { previews.push(null); }
      }
      form.value.imagesPreviews = previews;
    } catch (e) {
      console.error("Failed to load images", e);
    }

  } else {
    isEditing.value = false;
    form.value = {
      product_name: "",
      product_code: "",
      category: "",
      brand: "",
      buying_price: 0,
      default_selling_price: 0,
      stock_quantity: 0,
      unit: "pcs",
      tax_percentage: 0,
      original_price: 0,
      profit_percentage: 0,
      facebook_link: "",
      product_link: "",
      images: [],
      imagesPreviews: []
    };

  }
  showModal.value = true;
}

function closeModal() {
  showModal.value = false;
}

async function selectImages() {
  try {
    const selected = await open({
      multiple: true,
      filters: [{
        name: 'Images',
        extensions: ['png', 'jpg', 'jpeg', 'webp']
      }]
    });

    if (selected) {
      const paths = Array.isArray(selected) ? selected : [selected];
      form.value.images = [...form.value.images, ...paths];
      // Generate previews for new images
      for (const p of paths) {
        try {
          const b64 = await invoke('read_image_base64', { path: p });
          form.value.imagesPreviews.push(b64);
        } catch { form.value.imagesPreviews.push(null); }
      }
    }
  } catch (err) {
    console.error("Failed to select images:", err);
  }
}

function removeImage(index) {
  form.value.images.splice(index, 1);
  form.value.imagesPreviews.splice(index, 1);
}

async function saveProduct() {
  if (auth.isDemo) {
    alert("View-only account: Cannot save products.");
    return;
  }

  try {
    const productData = {
      id: form.value.id,
      product_name: form.value.product_name,
      product_code: form.value.product_code,
      category: form.value.category,
      brand: form.value.brand,
      buying_price: Number(form.value.buying_price),
      default_selling_price: Number(form.value.default_selling_price),
      stock_quantity: Number(form.value.stock_quantity),
      unit: form.value.unit,
      tax_percentage: Number(form.value.tax_percentage),
      original_price: Number(form.value.original_price),
      profit_percentage: Number(form.value.profit_percentage),
      facebook_link: form.value.facebook_link,
      product_link: form.value.product_link,
      created_at: form.value.created_at,
      updated_at: form.value.updated_at,
      is_deleted: 0,
      images: null
    };


    if (isEditing.value) {
      await invoke('update_product', { product: productData, images: form.value.images });
      await logActivity('UPDATE', 'Product', productData.id, `Updated product: ${productData.product_name}`);
    } else {
      await invoke('create_product', { product: productData, images: form.value.images });
      await logActivity('CREATE', 'Product', null, `Created product: ${productData.product_name}`);
    }

    closeModal();
    loadProducts();
  } catch (error) {
    console.error("Failed to save product:", error);
    alert("Error saving product: " + error);
  }
}

async function deleteProduct(product) {
  if (auth.isDemo) {
    alert("View-only account: Cannot delete products.");
    return;
  }

  if (product.stock_quantity > 0) {
    alert("Cannot delete product: stock is currently available. Please empty stock first.");
    return;
  }
  const isConfirmed = await confirm(`Are you sure you want to delete ${product.product_name}?`, { kind: 'warning' });
  if (!isConfirmed) return;
  try {
    await invoke('delete_product', { id: product.id });
    await logActivity('DELETE', 'Product', product.id, `Deleted product #${product.id}`);
    loadProducts();
  } catch (error) {
    console.error("Failed to delete product:", error);
  }
}

function openViewModal(product) {
  selectedProduct.value = product;
  showViewModal.value = true;
}




function formatDate(dateStr) {
  if (!dateStr) return 'N/A';
  return new Date(dateStr).toLocaleString();
}

onBeforeRouteLeave((to, from) => {
  if (showModal.value) {
    const isFilled = form.value.product_name || form.value.product_code || form.value.brand || form.value.category || form.value.buying_price > 0 || form.value.default_selling_price > 0;
    if (isFilled) {
      const answer = window.confirm('You have unsaved changes. Are you sure you want to leave?');
      if (!answer) return false;
    }
  }
});

onMounted(() => {

  loadProducts();
});
</script>

<template>
  <div class="flex flex-col space-y-4">
    <div class="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-3">
      <h1 class="text-2xl md:text-3xl font-bold text-gray-800">Products</h1>
      <button v-if="!auth.isDemo" @click="openModal()"
        class="w-full sm:w-auto justify-center bg-blue-600 hover:bg-blue-700 text-white px-4 py-2 rounded-lg shadow transition text-sm flex items-center gap-2">
        + Add Product
      </button>
    </div>

    <!-- Search -->
    <div class="bg-white p-3 rounded-lg shadow">
      <input v-model="searchQuery" @input="currentPage = 1" type="text" placeholder="Search by name or code..."
        class="w-full border border-gray-300 rounded-lg px-4 py-2 focus:ring-2 focus:ring-blue-500 focus:outline-none text-sm">
    </div>

    <!-- Card Grid View -->
    <div class="flex-1 overflow-y-auto pr-2 pb-4">
      <div v-if="filteredProducts.length === 0" class="bg-white rounded-lg shadow p-8 text-center text-gray-500">
        No products found.
      </div>
      <div v-else class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 xl:grid-cols-4 gap-4">
        <div v-for="product in paginatedProducts" :key="product.id"
          class="bg-white rounded-xl shadow-sm border border-gray-100 overflow-hidden hover:shadow-md transition-shadow flex flex-col relative">

          <!-- Image Section -->
          <div class="aspect-video w-full bg-gray-50 flex items-center justify-center relative overflow-hidden group">
            <img v-if="product._thumb" :src="product._thumb"
              class="w-full h-full object-cover group-hover:scale-105 transition-transform duration-300" />
            <div v-else class="w-full h-full flex flex-col items-center justify-center text-gray-400">
              <svg xmlns="http://www.w3.org/2000/svg" class="h-10 w-10 mb-2 opacity-50" fill="none" viewBox="0 0 24 24"
                stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                  d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z" />
              </svg>
              <span class="text-xs font-medium uppercase tracking-wider">No Image</span>
            </div>

            <!-- Category Badge overlay -->
            <div v-if="product.category"
              class="absolute top-2 left-2 bg-white/90 backdrop-blur-sm px-2 py-1 rounded text-[10px] font-bold text-gray-600 shadow-sm border border-white/20 uppercase tracking-widest">
              {{ product.category }}
            </div>

            <!-- Code Badge overlay -->
            <div v-if="product.product_code"
              class="absolute top-2 right-2 bg-black/60 backdrop-blur-sm px-2 py-1 rounded text-xs font-mono text-white/90 shadow-sm">
              {{ product.product_code }}
            </div>
          </div>

          <!-- Content Section -->
          <div class="p-4 flex flex-col flex-1">
            <h3 class="font-bold text-gray-800 text-lg leading-tight mb-1 line-clamp-2" :title="product.product_name">
              {{ product.product_name }}
            </h3>

            <!-- Stock & Price Highlight -->
            <div class="mt-3 flex items-center justify-between border-b pb-3 border-gray-100">
              <div>
                <div class="text-[10px] text-gray-500 uppercase font-semibold mb-0.5">Stock</div>
                <div class="flex items-baseline gap-1">
                  <span class="text-xl font-black leading-none"
                    :class="{ 'text-red-500': product.stock_quantity <= 5, 'text-gray-800': product.stock_quantity > 5 }">
                    {{ product.stock_quantity }}
                  </span>
                  <span class="text-xs text-gray-500 font-medium">{{ product.unit }}</span>
                </div>
              </div>

              <div class="text-right">
                <div class="text-[10px] text-gray-500 uppercase font-semibold mb-0.5">Selling Price</div>
                <div class="text-lg font-bold text-blue-600 leading-none">
                  {{ currencySymbol }}{{ product.default_selling_price.toFixed(2) }}
                </div>
              </div>
            </div>

            <!-- Additional Details -->
            <div class="mt-3 grid grid-cols-2 gap-x-2 gap-y-2 text-xs">
              <div class="flex flex-col">
                <span class="text-gray-400">Buy Price</span>
                <span class="font-medium text-gray-700">{{ currencySymbol }}{{ product.buying_price.toFixed(2) }}</span>
              </div>
              <div class="flex flex-col text-right">
                <span class="text-gray-400">Buying Cost</span>
                <span class="font-medium text-gray-700">{{ currencySymbol }}{{ (product.buying_price -
                  product.original_price).toFixed(2) }}</span>
              </div>
              <div class="grid col-span-2 bg-green-50 rounded-lg p-2 mt-1 border border-green-100">
                <div class="flex justify-between items-center">
                  <span class="text-green-700 font-medium">Est. Profit</span>
                  <span class="font-bold text-green-700 text-sm">{{ currencySymbol }}{{ (product.default_selling_price -
                    product.buying_price).toFixed(2) }}</span>
                </div>
              </div>
            </div>

            <div class="mt-auto pt-4">
              <!-- Actions -->
              <div class="flex gap-2">
                <button @click="openViewModal(product)"
                  class="flex-1 flex justify-center items-center gap-1.5 py-2 px-3 bg-gray-50 hover:bg-gray-100 text-gray-600 rounded-lg text-xs font-bold transition-colors border border-gray-200"
                  title="View">
                  <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24"
                    stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                      d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                      d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
                  </svg>
                  <span class="sr-only sm:not-sr-only">View</span>
                </button>
                <button v-if="!auth.isDemo" @click="openModal(product)"
                  class="flex-1 flex justify-center items-center gap-1.5 py-2 px-3 bg-blue-50 hover:bg-blue-100 text-blue-600 rounded-lg text-xs font-bold transition-colors border border-blue-200"
                  title="Edit">
                  <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24"
                    stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                      d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                  </svg>
                  <span class="sr-only sm:not-sr-only">Edit</span>
                </button>
                <button v-if="!auth.isDemo" @click="deleteProduct(product)"
                  class="bg-red-50 hover:bg-red-100 text-red-600 p-2 rounded-lg transition-colors border border-red-200"
                  title="Delete">
                  <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24"
                    stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                      d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                  </svg>
                </button>
              </div>
            </div>

          </div>
        </div>
      </div>

      <!-- Pagination -->
      <div v-if="totalPages > 1"
        class="flex flex-col sm:flex-row items-center justify-between gap-3 mt-6 bg-white rounded-xl p-4 shadow-sm border border-gray-100">
        <span class="text-xs font-bold text-gray-400 uppercase tracking-widest">Showing {{ (currentPage - 1) *
          itemsPerPage + 1 }}–{{ Math.min(currentPage * itemsPerPage, filteredProducts.length) }} of {{
            filteredProducts.length }}</span>
        <div class="flex items-center gap-2">
          <button @click="goToPage(currentPage - 1)" :disabled="currentPage === 1"
            class="px-4 py-2 rounded-lg text-xs font-black uppercase tracking-widest border transition-all disabled:opacity-30 disabled:cursor-not-allowed hover:bg-gray-50 active:scale-95">←
            Prev</button>
          <span class="text-xs font-bold text-gray-600 px-3">Page {{ currentPage }} of {{ totalPages }}</span>
          <button @click="goToPage(currentPage + 1)" :disabled="currentPage === totalPages"
            class="px-4 py-2 rounded-lg text-xs font-black uppercase tracking-widest border transition-all disabled:opacity-30 disabled:cursor-not-allowed hover:bg-gray-50 active:scale-95">Next
            →</button>
        </div>
      </div>
    </div>

    <!-- Modal -->
    <div v-if="showModal" class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4">
      <div class="bg-white rounded-xl shadow-2xl w-full max-w-3xl p-4 sm:p-5 relative max-h-[90vh] flex flex-col my-4">
        <button @click="closeModal"
          class="absolute top-3 right-3 text-gray-500 hover:text-gray-700 text-xl w-8 h-8 flex items-center justify-center rounded-full bg-gray-100 hover:bg-gray-200 transition-colors">✕</button>
        <h2 class="text-lg sm:text-xl font-bold mb-4 text-gray-800 shrink-0 pr-8">
          {{ isEditing ? 'Edit Product' : 'Add ' + 'New Product' }}
        </h2>

        <div class="overflow-y-auto flex-1 pr-1">
          <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
            <div class="sm:col-span-2">
              <label class="block text-sm font-medium text-gray-700 mb-1">Product Name *</label>
              <input v-model="form.product_name" type="text"
                class="w-full border border-gray-300 rounded-lg px-3 py-2 focus:ring-blue-500 focus:outline-none text-sm">
            </div>

            <div>
              <label class="block text-sm font-medium text-gray-700 mb-1">Product Code (SKU)</label>
              <input v-model="form.product_code" type="text"
                class="w-full border border-gray-300 rounded-lg px-3 py-2 focus:ring-blue-500 focus:outline-none text-sm">
            </div>

            <div>
              <label class="block text-sm font-medium text-gray-700 mb-1">Category</label>
              <input v-model="form.category" type="text"
                class="w-full border border-gray-300 rounded-lg px-3 py-2 focus:ring-blue-500 focus:outline-none text-sm">
            </div>

            <div>
              <label class="block text-sm font-medium text-gray-700 mb-1">Brand</label>
              <input v-model="form.brand" type="text"
                class="w-full border border-gray-300 rounded-lg px-3 py-2 focus:ring-blue-500 focus:outline-none text-sm">
            </div>

            <div>
              <label class="block text-sm font-medium text-gray-700 mb-1">Unit (pcs, kg)</label>
              <input v-model="form.unit" type="text"
                class="w-full border border-gray-300 rounded-lg px-3 py-2 focus:ring-blue-500 focus:outline-none text-sm">
            </div>

            <div>
              <label class="block text-sm font-medium text-gray-700 mb-1">Original Product Price</label>
              <div class="relative">
                <span class="absolute left-3 top-2 text-gray-500 text-sm">{{ currencySymbol }}</span>
                <input v-model.number="form.original_price" type="number" step="0.01"
                  class="w-full border border-gray-300 rounded-lg pl-8 pr-3 py-2 focus:ring-blue-500 focus:outline-none text-sm">
              </div>
            </div>

            <div>
              <label class="block text-sm font-medium text-gray-700 mb-1">Buying Price (Average Cost)</label>
              <div class="relative">
                <span class="absolute left-3 top-2 text-gray-500 text-sm">{{ currencySymbol }}</span>
                <input v-model.number="form.buying_price" type="number" step="0.01"
                  class="w-full border border-gray-100 bg-gray-50 rounded-lg pl-8 pr-3 py-2 text-gray-500 text-sm cursor-not-allowed"
                  disabled>
              </div>
              <span class="text-[10px] text-blue-600 font-medium whitespace-nowrap">Auto-updated via weighted average
                procurement</span>
            </div>

            <div>
              <label class="block text-sm font-medium text-gray-700 mb-1">Profit Percentage (%)</label>
              <div class="relative">
                <input v-model.number="form.profit_percentage" type="number" step="0.1"
                  class="w-full border border-gray-300 rounded-lg px-3 py-2 focus:ring-blue-500 focus:outline-none text-sm transition-all shadow-sm">
                <span class="absolute right-3 top-2 text-gray-400 text-xs font-bold">%</span>
              </div>
            </div>

            <div>
              <label class="block text-sm font-medium text-gray-700 mb-1">Expected Selling Price</label>
              <div class="relative">
                <span class="absolute left-3 top-2 text-gray-500 text-sm">{{ currencySymbol }}</span>
                <input :value="expectedSellingPrice.toFixed(2)" type="text"
                  class="w-full border border-gray-100 bg-blue-50/50 text-blue-700 font-bold rounded-lg pl-8 pr-3 py-2 text-sm cursor-not-allowed"
                  disabled>
              </div>
              <span class="text-[10px] text-blue-600 font-medium italic">Buy Price + {{ form.profit_percentage
              }}%</span>
            </div>

            <div>
              <label class="block text-sm font-medium text-gray-700 mb-1">Final Selling Price</label>
              <div class="relative">
                <span class="absolute left-3 top-2 text-gray-500 text-sm">{{ currencySymbol }}</span>
                <input v-model.number="form.default_selling_price" type="number" step="0.01"
                  class="w-full border border-gray-300 rounded-lg pl-8 pr-3 py-2 focus:ring-blue-500 focus:outline-none text-sm shadow-sm">
              </div>
              <span class="text-[10px] text-gray-400">Target price for POS</span>
            </div>

            <div>
              <label class="block text-sm font-medium text-gray-700 mb-1">Tax %</label>
              <input v-model.number="form.tax_percentage" type="number" step="0.1"
                class="w-full border border-gray-300 rounded-lg px-3 py-2 focus:ring-blue-500 focus:outline-none text-sm">
            </div>

            <div>
              <label class="block text-sm font-medium text-gray-700 mb-1">Current Stock</label>
              <input v-model.number="form.stock_quantity" type="number"
                class="w-full border border-gray-100 bg-gray-50 rounded-lg px-3 py-2 text-gray-500 text-sm cursor-not-allowed"
                disabled>
              <span class="text-[10px] text-amber-600 font-medium whitespace-nowrap">Auto-managed via entries</span>
            </div>

            <div class="sm:col-span-2 grid grid-cols-1 sm:grid-cols-2 gap-4">
              <div>
                <label class="block text-sm font-medium text-gray-700 mb-1">Facebook Link (Optional)</label>
                <input v-model="form.facebook_link" type="url" placeholder="https://facebook.com/..."
                  class="w-full border border-gray-300 rounded-lg px-3 py-2 focus:ring-blue-500 focus:outline-none text-sm">
              </div>

              <div>
                <label class="block text-sm font-medium text-gray-700 mb-1">Product Link (Optional)</label>
                <input v-model="form.product_link" type="url" placeholder="https://example.com/product/..."
                  class="w-full border border-gray-300 rounded-lg px-3 py-2 focus:ring-blue-500 focus:outline-none text-sm">
              </div>
            </div>


            <!-- Image Upload Section -->
            <div class="sm:col-span-2 border-t pt-3 mt-1">
              <div class="flex justify-between items-center mb-2">
                <label class="block text-sm font-medium text-gray-700">Product Images</label>
                <button @click="selectImages"
                  class="text-xs bg-gray-100 hover:bg-gray-200 text-gray-700 px-3 py-1 rounded border">
                  Select Images
                </button>
              </div>

              <div class="grid grid-cols-3 sm:grid-cols-4 gap-3 mt-2">
                <div v-for="(preview, index) in form.imagesPreviews" :key="index"
                  class="relative group aspect-square bg-gray-100 rounded-lg overflow-hidden border">
                  <img v-if="preview" :src="preview" class="w-full h-full object-cover">
                  <div v-else class="w-full h-full flex items-center justify-center text-gray-400 text-xs">Error</div>
                  <button @click="removeImage(index)"
                    class="absolute top-1 right-1 bg-red-600 text-white rounded-full w-5 h-5 flex items-center justify-center text-xs opacity-0 group-hover:opacity-100 transition shadow">
                    ×
                  </button>
                </div>
                <div v-if="form.imagesPreviews.length === 0"
                  class="col-span-3 sm:col-span-4 text-center text-gray-400 py-4 text-sm italic border-2 border-dashed rounded-lg">
                  No images selected.
                </div>
              </div>
            </div>

          </div>
        </div>

        <div class="mt-4 flex justify-end space-x-3 shrink-0 pt-3 border-t">
          <button @click="closeModal"
            class="px-5 py-2 border border-gray-300 rounded-lg text-gray-700 hover:bg-gray-50 font-medium text-sm">Cancel</button>
          <button @click="saveProduct"
            class="px-5 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 font-bold shadow text-sm">
            {{ isEditing ? 'Update Product' : 'Save Product' }}
          </button>
        </div>
      </div>
    </div>

    <!-- View Details Modal Component -->
    <ProductDetailsModal :show="showViewModal" :product="selectedProduct" :currency-symbol="currencySymbol"
      @close="showViewModal = false" />
  </div>
</template>
