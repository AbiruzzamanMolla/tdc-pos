<script setup>
import { ref, onMounted, computed } from 'vue';
import { onBeforeRouteLeave } from 'vue-router';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import ProductDetailsModal from '../components/ProductDetailsModal.vue';
import { logActivity } from '../utils/activityLogger';


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
  return buy + (buy * (percent / 100));
});

const showViewModal = ref(false);
const selectedProduct = ref(null);




const filteredProducts = computed(() => {
  if (!searchQuery.value) return products.value;
  const query = searchQuery.value.toLowerCase();
  return products.value.filter(p =>
    p.product_name.toLowerCase().includes(query) ||
    (p.product_code && p.product_code.toLowerCase().includes(query))
  );
});

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

async function deleteProduct(id) {
  if (!confirm("Are you sure you want to delete this product?")) return;
  try {
    await invoke('delete_product', { id });
    await logActivity('DELETE', 'Product', id, `Deleted product #${id}`);
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
  <div class="h-full flex flex-col space-y-4">
    <div class="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-3">
      <h1 class="text-2xl md:text-3xl font-bold text-gray-800">Products</h1>
      <button @click="openModal()"
        class="bg-blue-600 hover:bg-blue-700 text-white px-4 py-2 rounded-lg shadow transition text-sm">
        + Add Product
      </button>
    </div>

    <!-- Search -->
    <div class="bg-white p-3 rounded-lg shadow">
      <input v-model="searchQuery" type="text" placeholder="Search by name or code..."
        class="w-full border border-gray-300 rounded-lg px-4 py-2 focus:ring-2 focus:ring-blue-500 focus:outline-none text-sm">
    </div>

    <!-- Table -->
    <div class="bg-white rounded-lg shadow overflow-hidden flex-1 overflow-x-auto overflow-y-auto">
      <table class="w-full text-left border-collapse min-w-[700px]">
        <thead class="bg-gray-100 text-gray-600 uppercase text-xs font-semibold sticky top-0 z-10">
          <tr>
            <th class="p-3 border-b">Image</th>
            <th class="p-3 border-b">Code</th>
            <th class="p-3 border-b">Name</th>
            <th class="p-3 border-b">Category</th>
            <th class="p-3 border-b">Stock</th>
            <th class="p-3 border-b">Buy Price</th>
            <th class="p-3 border-b">Sell Price</th>
            <th class="p-3 border-b">Buying Cost</th>
            <th class="p-3 border-b">Profit</th>
            <th class="p-3 border-b text-right">Actions</th>
          </tr>

        </thead>
        <tbody class="text-gray-700 text-sm">
          <tr v-for="product in filteredProducts" :key="product.id" class="hover:bg-gray-50 border-b last:border-b-0">
            <td class="p-3">
              <div class="w-10 h-10 rounded bg-gray-100 overflow-hidden flex-shrink-0">
                <img v-if="product._thumb" :src="product._thumb" class="w-full h-full object-cover" />
                <div v-else class="w-full h-full flex items-center justify-center text-gray-400 text-xs">N/A</div>
              </div>
            </td>
            <td class="p-3 text-xs font-mono">{{ product.product_code || '-' }}</td>
            <td class="p-3 font-medium">{{ product.product_name }}</td>
            <td class="p-3">{{ product.category || '-' }}</td>
            <td class="p-3" :class="{ 'text-red-500 font-bold': product.stock_quantity <= 5 }">
              {{ product.stock_quantity }} {{ product.unit }}
            </td>
            <td class="p-3">{{ currencySymbol }}{{ product.buying_price.toFixed(2) }}</td>
            <td class="p-3">{{ currencySymbol }}{{ product.default_selling_price.toFixed(2) }}</td>
            <td class="p-3">{{ currencySymbol }}{{ (product.buying_price - product.original_price).toFixed(2) }}</td>
            <td class="p-3 font-bold text-green-600">{{ currencySymbol }}{{ (product.default_selling_price -
              product.buying_price).toFixed(2) }}</td>
            <td class="p-3 text-right space-x-1 whitespace-nowrap">
              <button @click="openViewModal(product)"
                class="text-green-600 hover:text-green-800 text-xs font-medium border border-green-200 px-2 py-1 rounded hover:bg-green-50">View</button>
              <button @click="openModal(product)"
                class="text-blue-600 hover:text-blue-800 text-xs font-medium border border-blue-200 px-2 py-1 rounded hover:bg-blue-50">Edit</button>
              <button @click="deleteProduct(product.id)"
                class="text-red-600 hover:text-red-800 text-xs font-medium border border-red-200 px-2 py-1 rounded hover:bg-red-50">Delete</button>
            </td>

          </tr>
          <tr v-if="filteredProducts.length === 0">
            <td colspan="8" class="p-8 text-center text-gray-500">No products found.</td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- Modal -->
    <div v-if="showModal" class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4">
      <div class="bg-white rounded-xl shadow-2xl w-full max-w-3xl p-5 relative max-h-[90vh] flex flex-col">
        <button @click="closeModal" class="absolute top-3 right-3 text-gray-500 hover:text-gray-700 text-xl">✕</button>
        <h2 class="text-xl font-bold mb-4 text-gray-800 shrink-0">{{ isEditing ? 'Edit Product' : 'Add New Product' }}
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
