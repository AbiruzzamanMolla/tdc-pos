<script setup>
import { ref, onMounted, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { convertFileSrc } from '@tauri-apps/api/core';

const products = ref([]);
const searchQuery = ref("");
const showModal = ref(false);
const isEditing = ref(false);

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
  images: [] // Array of file paths
});

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
    products.value = await invoke('get_products');
  } catch (error) {
    console.error("Failed to load products:", error);
  }
}

async function openModal(product = null) {
  if (product) {
    isEditing.value = true;
    form.value = { ...product, images: [] }; // Reset images initially

    // Fetch images for this product
    try {
      const images = await invoke('get_product_images', { productId: product.id });
      form.value.images = images || [];
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
      images: []
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
      if (Array.isArray(selected)) {
        form.value.images = [...form.value.images, ...selected];
      } else {
        form.value.images.push(selected);
      }
    }
  } catch (err) {
    console.error("Failed to select images:", err);
  }
}

function removeImage(index) {
  form.value.images.splice(index, 1);
}

function getAssetUrl(path) {
  return convertFileSrc(path);
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
      created_at: form.value.created_at,
      updated_at: form.value.updated_at,
      is_deleted: 0,
      images: null // Handled separately
    };

    if (isEditing.value) {
      await invoke('update_product', { product: productData, images: form.value.images });
    } else {
      await invoke('create_product', { product: productData, images: form.value.images });
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
    loadProducts();
  } catch (error) {
    console.error("Failed to delete product:", error);
  }
}

onMounted(() => {
  loadProducts();
});
</script>

<template>
  <div class="h-full flex flex-col space-y-6">
    <div class="flex justify-between items-center">
      <h1 class="text-3xl font-bold text-gray-800">Products</h1>
      <button @click="openModal()"
        class="bg-blue-600 hover:bg-blue-700 text-white px-4 py-2 rounded-lg shadow transition">
        + Add Product
      </button>
    </div>

    <!-- Search -->
    <div class="bg-white p-4 rounded-lg shadow">
      <input v-model="searchQuery" type="text" placeholder="Search by name or code..."
        class="w-full border border-gray-300 rounded-lg px-4 py-2 focus:ring-2 focus:ring-blue-500 focus:outline-none">
    </div>

    <!-- Table -->
    <div class="bg-white rounded-lg shadow overflow-hidden flex-1 overflow-y-auto">
      <table class="w-full text-left border-collapse">
        <thead class="bg-gray-100 text-gray-600 uppercase text-sm font-semibold sticky top-0 z-10">
          <tr>
            <th class="p-4 border-b">Code</th>
            <th class="p-4 border-b">Name</th>
            <th class="p-4 border-b">Category</th>
            <th class="p-4 border-b">Stock</th>
            <th class="p-4 border-b">Buy Price</th>
            <th class="p-4 border-b">Sell Price</th>
            <th class="p-4 border-b text-right">Actions</th>
          </tr>
        </thead>
        <tbody class="text-gray-700">
          <tr v-for="product in filteredProducts" :key="product.id" class="hover:bg-gray-50 border-b last:border-b-0">
            <td class="p-4">{{ product.product_code || '-' }}</td>
            <td class="p-4 font-medium">{{ product.product_name }}</td>
            <td class="p-4">{{ product.category || '-' }}</td>
            <td class="p-4" :class="{ 'text-red-500 font-bold': product.stock_quantity <= 5 }">
              {{ product.stock_quantity }} {{ product.unit }}
            </td>
            <td class="p-4">{{ product.buying_price.toFixed(2) }}</td>
            <td class="p-4">{{ product.default_selling_price.toFixed(2) }}</td>
            <td class="p-4 text-right space-x-2">
              <button @click="openModal(product)" class="text-blue-600 hover:text-blue-800 font-medium">Edit</button>
              <button @click="deleteProduct(product.id)"
                class="text-red-600 hover:text-red-800 font-medium">Delete</button>
            </td>
          </tr>
          <tr v-if="filteredProducts.length === 0">
            <td colspan="7" class="p-8 text-center text-gray-500">No products found.</td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- Modal -->
    <div v-if="showModal" class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div class="bg-white rounded-xl shadow-2xl w-full max-w-4xl p-6 relative h-[90vh] flex flex-col">
        <button @click="closeModal" class="absolute top-4 right-4 text-gray-500 hover:text-gray-700 text-2xl">✕</button>
        <h2 class="text-2xl font-bold mb-6 text-gray-800 shrink-0">{{ isEditing ? 'Edit Product' : 'Add New Product' }}
        </h2>

        <div class="overflow-y-auto flex-1 pr-2">
          <div class="grid grid-cols-2 gap-6">
            <div class="col-span-2">
              <label class="block text-sm font-medium text-gray-700 mb-1">Product Name</label>
              <input v-model="form.product_name" type="text"
                class="w-full border border-gray-300 rounded-lg px-3 py-2 focus:ring-blue-500 focus:outline-none">
            </div>

            <div>
              <label class="block text-sm font-medium text-gray-700 mb-1">Product Code (SKU)</label>
              <input v-model="form.product_code" type="text"
                class="w-full border border-gray-300 rounded-lg px-3 py-2 focus:ring-blue-500 focus:outline-none">
            </div>

            <div>
              <label class="block text-sm font-medium text-gray-700 mb-1">Category</label>
              <input v-model="form.category" type="text"
                class="w-full border border-gray-300 rounded-lg px-3 py-2 focus:ring-blue-500 focus:outline-none">
            </div>

            <div>
              <label class="block text-sm font-medium text-gray-700 mb-1">Brand</label>
              <input v-model="form.brand" type="text"
                class="w-full border border-gray-300 rounded-lg px-3 py-2 focus:ring-blue-500 focus:outline-none">
            </div>

            <div>
              <label class="block text-sm font-medium text-gray-700 mb-1">Unit (pcs, kg)</label>
              <input v-model="form.unit" type="text"
                class="w-full border border-gray-300 rounded-lg px-3 py-2 focus:ring-blue-500 focus:outline-none">
            </div>

            <div>
              <label class="block text-sm font-medium text-gray-700 mb-1">Buying Price (Cost)</label>
              <div class="relative">
                <span class="absolute left-3 top-2 text-gray-500">৳</span>
                <input v-model.number="form.buying_price" type="number" step="0.01"
                  class="w-full border border-gray-300 rounded-lg pl-8 pr-3 py-2 focus:ring-blue-500 focus:outline-none">
              </div>
            </div>

            <div>
              <label class="block text-sm font-medium text-gray-700 mb-1">Selling Price</label>
              <div class="relative">
                <span class="absolute left-3 top-2 text-gray-500">৳</span>
                <input v-model.number="form.default_selling_price" type="number" step="0.01"
                  class="w-full border border-gray-300 rounded-lg pl-8 pr-3 py-2 focus:ring-blue-500 focus:outline-none">
              </div>
            </div>

            <div>
              <label class="block text-sm font-medium text-gray-700 mb-1">Initial Stock</label>
              <input v-model.number="form.stock_quantity" type="number"
                class="w-full border border-gray-300 rounded-lg px-3 py-2 focus:ring-blue-500 focus:outline-none"
                :disabled="isEditing">
              <span v-if="isEditing" class="text-xs text-gray-500">Stock managed via Stock Entries & Sales</span>
            </div>

            <div>
              <label class="block text-sm font-medium text-gray-700 mb-1">Tax %</label>
              <input v-model.number="form.tax_percentage" type="number" step="0.1"
                class="w-full border border-gray-300 rounded-lg px-3 py-2 focus:ring-blue-500 focus:outline-none">
            </div>

            <!-- Image Upload Section -->
            <div class="col-span-2 border-t pt-4 mt-2">
              <div class="flex justify-between items-center mb-2">
                <label class="block text-sm font-medium text-gray-700">Product Images</label>
                <button @click="selectImages"
                  class="text-sm bg-gray-100 hover:bg-gray-200 text-gray-700 px-3 py-1 rounded border">
                  Select Images
                </button>
              </div>

              <div class="grid grid-cols-4 gap-4 mt-2">
                <div v-for="(img, index) in form.images" :key="index"
                  class="relative group aspect-square bg-gray-100 rounded-lg overflow-hidden border">
                  <img :src="getAssetUrl(img)" class="w-full h-full object-cover">
                  <button @click="removeImage(index)"
                    class="absolute top-1 right-1 bg-red-600 text-white rounded-full p-1 opacity-0 group-hover:opacity-100 transition shadow">
                    <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
                      <path fill-rule="evenodd"
                        d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z"
                        clip-rule="evenodd" />
                    </svg>
                  </button>
                </div>
                <!-- Placeholder if empty -->
                <div v-if="form.images.length === 0"
                  class="col-span-4 text-center text-gray-400 py-4 italic border-2 border-dashed rounded-lg">
                  No images selected. Click "Select Images" to add.
                </div>
              </div>
            </div>

          </div>
        </div>

        <div class="mt-6 flex justify-end space-x-3 shrink-0 pt-4 border-t">
          <button @click="closeModal"
            class="px-6 py-2 border border-gray-300 rounded-lg text-gray-700 hover:bg-gray-50 font-medium">Cancel</button>
          <button @click="saveProduct"
            class="px-6 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 font-bold shadow">
            {{ isEditing ? 'Update Product' : 'Save Product' }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
