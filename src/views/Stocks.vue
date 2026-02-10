<script setup>
import { ref, onMounted, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';

const products = ref([]);
const searchQuery = ref("");
const loading = ref(true);

async function loadStock() {
    try {
        loading.value = true;
        products.value = await invoke('get_products');
    } catch (err) {
        console.error("Failed to load stock", err);
    } finally {
        loading.value = false;
    }
}

const filteredProducts = computed(() => {
    if (!searchQuery.value) return products.value;
    const query = searchQuery.value.toLowerCase();
    return products.value.filter(p =>
        p.product_name.toLowerCase().includes(query) ||
        (p.product_code && p.product_code.toLowerCase().includes(query))
    );
});

onMounted(loadStock);
</script>

<template>
    <div class="h-full flex flex-col space-y-6">
        <div class="flex justify-between items-center">
            <h1 class="text-3xl font-bold text-gray-800">Available Stock</h1>
            <button @click="loadStock" class="text-blue-600 hover:text-blue-800 text-sm font-medium">Refresh</button>
        </div>

        <div class="bg-white p-4 rounded-xl shadow-sm border border-gray-100 mb-4">
            <input v-model="searchQuery" type="text" placeholder="Search product by name or SKU..."
                class="w-full border border-gray-200 rounded-lg px-4 py-2 focus:ring-2 focus:ring-blue-500 focus:outline-none transition-all">
        </div>

        <div class="bg-white rounded-xl shadow-sm border border-gray-100 overflow-hidden flex-1 flex flex-col">
            <div class="overflow-x-auto">
                <table class="w-full text-left border-collapse">
                    <thead class="bg-gray-50 text-gray-500 uppercase text-xs font-bold">
                        <tr>
                            <th class="p-4 border-b">Product Name</th>
                            <th class="p-4 border-b">SKU / Code</th>
                            <th class="p-4 border-b">Category</th>
                            <th class="p-4 border-b text-center">In Stock</th>
                            <th class="p-4 border-b text-center">Unit</th>
                            <th class="p-4 border-b text-center">Status</th>
                        </tr>
                    </thead>
                    <tbody class="text-gray-700 divide-y divide-gray-50">
                        <tr v-for="product in filteredProducts" :key="product.id"
                            class="hover:bg-blue-50/30 transition-colors">
                            <td class="p-4 font-semibold">{{ product.product_name }}</td>
                            <td class="p-4 text-sm font-mono text-gray-500">{{ product.product_code || '-' }}</td>
                            <td class="p-4 text-sm">{{ product.category || 'General' }}</td>
                            <td class="p-4 text-center font-black text-lg"
                                :class="product.stock_quantity <= 5 ? 'text-red-600' : 'text-blue-700'">
                                {{ product.stock_quantity }}
                            </td>
                            <td class="p-4 text-center text-sm text-gray-500">{{ product.unit || 'pcs' }}</td>
                            <td class="p-4 text-center">
                                <span v-if="product.stock_quantity <= 0"
                                    class="bg-red-100 text-red-600 px-2 py-1 rounded-full text-xs font-bold">Out of
                                    Stock</span>
                                <span v-else-if="product.stock_quantity <= 5"
                                    class="bg-orange-100 text-orange-600 px-2 py-1 rounded-full text-xs font-bold">Low
                                    Stock</span>
                                <span v-else
                                    class="bg-green-100 text-green-600 px-2 py-1 rounded-full text-xs font-bold">Available</span>
                            </td>
                        </tr>
                        <tr v-if="filteredProducts.length === 0 && !loading">
                            <td colspan="6" class="p-10 text-center text-gray-400 italic">No products found.</td>
                        </tr>
                        <tr v-if="loading">
                            <td colspan="6" class="p-10 text-center text-blue-500">Loading stock data...</td>
                        </tr>
                    </tbody>
                </table>
            </div>
        </div>
    </div>
</template>
