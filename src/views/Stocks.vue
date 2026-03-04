<script setup>
import { ref, onMounted, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import ProductDetailsModal from '../components/ProductDetailsModal.vue';


const products = ref([]);
const searchQuery = ref("");
const loading = ref(true);
const selectedProduct = ref(null);
const showViewModal = ref(false);


const currencySymbol = ref('৳');
async function loadSettings() {
    try {
        const settings = await invoke('get_settings');
        const currency = settings.find(s => s.key === 'currency_symbol');
        if (currency) currencySymbol.value = currency.value;
    } catch (err) {
        console.error("Failed to load settings", err);
    }
}


async function loadStock() {
    try {
        loading.value = true;
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
    } catch (err) {
        console.error("Failed to load stock", err);
    } finally {
        loading.value = false;
    }
}


async function viewHistory(product) {
    selectedProduct.value = product;
    showViewModal.value = true;
}


function formatDate(dateStr) {
    if (!dateStr) return 'N/A';
    return new Date(dateStr).toLocaleString();
}


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

onMounted(() => {
    loadSettings();
    loadStock();
});

</script>

<template>
    <div class="flex flex-col space-y-6">
        <div class="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-2 sm:gap-0">
            <h1 class="text-2xl sm:text-3xl font-bold text-gray-800">Available Stock</h1>
            <button @click="loadStock"
                class="text-blue-600 hover:text-blue-800 text-sm font-medium w-full sm:w-auto text-left sm:text-right">Refresh
                Data</button>
        </div>

        <div class="bg-white p-3 sm:p-4 rounded-xl shadow-sm border border-gray-100 mb-2 sm:mb-4">
            <input v-model="searchQuery" @input="currentPage = 1" type="text"
                placeholder="Search product by name or SKU..."
                class="w-full border border-gray-200 rounded-lg px-4 py-2 text-sm sm:text-base focus:ring-2 focus:ring-blue-500 focus:outline-none transition-all">
        </div>

        <!-- Card Grid View -->
        <div class="flex-1 overflow-y-auto pr-2 pb-4">
            <div v-if="loading"
                class="bg-white rounded-xl shadow-sm p-10 text-center text-blue-500 flex flex-col items-center justify-center">
                <svg class="animate-spin h-8 w-8 mb-4 border-2 border-t-blue-500 border-r-blue-500 border-b-transparent border-l-transparent rounded-full"
                    viewBox="0 0 24 24"></svg>
                Loading stock data...
            </div>
            <div v-else-if="filteredProducts.length === 0"
                class="bg-white rounded-xl shadow-sm p-10 text-center text-gray-400 italic">
                No products found.
            </div>
            <div v-else class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
                <div v-for="product in paginatedProducts" :key="product.id"
                    class="bg-white rounded-xl shadow-sm border border-gray-100 overflow-hidden hover:shadow-md transition-shadow hover:border-blue-100 flex flex-col relative group">

                    <!-- Top section with image and key info -->
                    <div class="p-4 flex gap-4 items-start border-b border-gray-50 bg-gray-50/50">
                        <!-- Tiny image box -->
                        <div
                            class="w-16 h-16 rounded-xl bg-white border border-gray-100 flex items-center justify-center text-gray-400 font-bold overflow-hidden flex-shrink-0 shadow-sm relative">
                            <img v-if="product._thumb" :src="product._thumb" class="w-full h-full object-cover">
                            <span v-else class="text-xl opacity-50">{{ product.product_name.charAt(0) }}</span>
                        </div>

                        <div class="flex-1 min-w-0">
                            <h3 class="font-bold text-gray-800 text-base leading-tight truncate"
                                :title="product.product_name">{{ product.product_name }}</h3>
                            <div class="flex flex-wrap items-center gap-2 mt-1 -ml-0.5">
                                <span
                                    class="bg-gray-100 text-gray-600 px-2 py-0.5 rounded text-[10px] font-mono tracking-wider truncate">{{
                                        product.product_code || 'NO-SKU' }}</span>
                                <span class="text-xs text-gray-400 truncate">{{ product.category || 'General' }}</span>
                            </div>
                        </div>
                    </div>

                    <!-- Main Body: Stock Info -->
                    <div class="p-4 flex-1 flex flex-col relative overflow-hidden">

                        <!-- Decorative background for low stock -->
                        <div v-if="product.stock_quantity <= 5"
                            class="absolute -right-6 -top-6 w-24 h-24 rounded-full bg-red-50 blur-xl z-0 pointer-events-none opacity-50">
                        </div>

                        <div class="relative z-10 flex items-center justify-between">
                            <div>
                                <div class="text-[10px] uppercase font-bold text-gray-400 tracking-wider mb-1">Current
                                    Stock</div>
                                <div class="flex items-baseline gap-1.5 focus:outline-none">
                                    <span class="text-3xl font-black tracking-tighter"
                                        :class="[product.stock_quantity <= 0 ? 'text-red-600' : product.stock_quantity <= 5 ? 'text-amber-500' : 'text-blue-600']">
                                        {{ product.stock_quantity }}
                                    </span>
                                    <span class="text-xs font-semibold text-gray-500">{{ product.unit || 'pcs' }}</span>
                                </div>
                            </div>

                            <!-- Badges -->
                            <div class="flex flex-col items-end gap-1 shrink-0">
                                <span v-if="product.stock_quantity <= 0"
                                    class="bg-red-50 text-red-600 border border-red-100 px-2.5 py-1 rounded-md text-[10px] font-black uppercase tracking-wider shadow-sm">Out
                                    of Stock</span>
                                <span v-else-if="product.stock_quantity <= 5"
                                    class="bg-amber-50 text-amber-600 border border-amber-100 px-2.5 py-1 rounded-md text-[10px] font-black uppercase tracking-wider shadow-sm">Low
                                    Stock</span>
                                <span v-else
                                    class="bg-green-50 text-green-600 border border-green-100 px-2.5 py-1 rounded-md text-[10px] font-black uppercase tracking-wider">Available</span>
                            </div>
                        </div>

                        <!-- Action Footer -->
                        <div class="mt-auto pt-4 relative z-10">
                            <button @click="viewHistory(product)"
                                class="w-full flex justify-center items-center gap-2 py-2.5 px-3 bg-white hover:bg-blue-50 text-blue-600 rounded-lg text-sm font-bold transition-all border border-gray-200 hover:border-blue-200 group-hover:shadow-sm">
                                <svg xmlns="http://www.w3.org/2000/svg"
                                    class="h-4 w-4 opacity-70 group-hover:opacity-100" fill="none" viewBox="0 0 24 24"
                                    stroke="currentColor">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                                        d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                                </svg>
                                Stock History & Details
                            </button>
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

        <!-- Product Details Modal Component -->
        <ProductDetailsModal :show="showViewModal" :product="selectedProduct" :currency-symbol="currencySymbol"
            @close="showViewModal = false" />
    </div>
</template>
