<script setup>
import { ref, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';

const props = defineProps({
    show: Boolean,
    product: Object,
    currencySymbol: { type: String, default: '৳' }
});

const emit = defineEmits(['close']);

const stockHistory = ref([]);
const loadingHistory = ref(false);

async function loadStockHistory() {
    if (!props.product?.id) return;

    loadingHistory.value = true;
    stockHistory.value = [];
    try {
        stockHistory.value = await invoke('get_product_stock_history', { productId: props.product.id });
    } catch (error) {
        console.error("Failed to load stock history:", error);
    } finally {
        loadingHistory.value = false;
    }
}

function formatDate(dateStr) {
    if (!dateStr) return 'N/A';
    return new Date(dateStr).toLocaleString();
}

function getEntityName(mv) {
    if (mv.entity_name) return mv.entity_name;
    return mv.movement_type === 'OUT' ? 'Walk-in Customer' : 'Unknown Supplier';
}

watch(() => props.show, (newVal) => {
    if (newVal && props.product) {
        loadStockHistory();
    }
});
</script>

<template>
    <div v-if="show && product"
        class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-[100] p-4 font-sans animate-in fade-in duration-200">
        <div
            class="bg-white rounded-xl shadow-2xl w-full max-w-2xl p-6 relative max-h-[90vh] flex flex-col transform transition-all scale-100">
            <button @click="emit('close')"
                class="absolute top-4 right-4 text-gray-400 hover:text-gray-600 transition-colors text-2xl z-10 p-1">✕</button>

            <div class="flex items-start gap-6 mb-8 shrink-0">
                <div
                    class="w-24 h-24 rounded-xl bg-gray-50 border border-gray-100 overflow-hidden flex-shrink-0 shadow-inner">
                    <img v-if="product._thumb" :src="product._thumb" class="w-full h-full object-cover" />
                    <div v-else
                        class="w-full h-full flex items-center justify-center text-gray-300 text-3xl font-bold bg-gray-50">
                        {{ product.product_name.charAt(0) }}
                    </div>
                </div>
                <div class="flex-1 min-w-0">
                    <h2 class="text-2xl font-extrabold text-gray-900 truncate mb-1">{{ product.product_name }}</h2>
                    <div class="flex flex-wrap gap-2 items-center">
                        <span
                            class="px-2.5 py-1 bg-blue-50 text-blue-700 text-xs font-bold rounded-lg border border-blue-100 uppercase tracking-wider">
                            {{ product.category || 'No Category' }}
                        </span>
                        <span class="text-gray-400 text-xs font-mono select-all">{{ product.product_code || 'No SKU'
                            }}</span>
                    </div>
                </div>
            </div>

            <div class="flex-1 overflow-y-auto pr-2 space-y-8 text-left">
                <!-- Price Grid -->
                <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
                    <div class="p-4 bg-gray-50 rounded-2xl border border-gray-100">
                        <span
                            class="block text-[10px] font-black text-gray-400 uppercase tracking-widest mb-1.5">Original</span>
                        <span class="text-lg font-bold text-gray-700">{{ currencySymbol }}{{
                            product.original_price?.toFixed(2) || '0.00' }}</span>
                    </div>
                    <div class="p-4 bg-blue-50 rounded-2xl border border-blue-100">
                        <span
                            class="block text-[10px] font-black text-blue-400 uppercase tracking-widest mb-1.5">Buying</span>
                        <span class="text-lg font-bold text-blue-700">{{ currencySymbol }}{{
                            product.buying_price?.toFixed(2) || '0.00' }}</span>
                    </div>
                    <div class="p-4 bg-green-50 rounded-2xl border border-green-100">
                        <span
                            class="block text-[10px] font-black text-green-400 uppercase tracking-widest mb-1.5">Selling</span>
                        <span class="text-lg font-bold text-green-700">{{ currencySymbol }}{{
                            product.default_selling_price?.toFixed(2) || '0.00' }}</span>
                    </div>
                    <div class="p-4 bg-amber-50 rounded-2xl border border-amber-100">
                        <span
                            class="block text-[10px] font-black text-amber-500 uppercase tracking-widest mb-1.5">Tax</span>
                        <span class="text-lg font-bold text-amber-700">{{ product.tax_percentage }}%</span>
                    </div>
                </div>

                <!-- Calculated Metrics -->
                <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
                    <div
                        class="flex items-center justify-between p-5 bg-gradient-to-br from-gray-50 to-white rounded-2xl border border-gray-100 shadow-sm transition-all hover:shadow-md">
                        <div>
                            <span
                                class="block text-[10px] font-black text-gray-400 uppercase tracking-widest mb-1 text-left">Buying
                                Cost</span>
                            <span class="text-xs text-gray-500 block text-left">(Buy - Orig)</span>
                        </div>
                        <span class="text-2xl font-black text-gray-900">{{ currencySymbol }}{{ ((product.buying_price ||
                            0) - (product.original_price || 0)).toFixed(2) }}</span>
                    </div>
                    <div
                        class="flex items-center justify-between p-5 bg-gradient-to-br from-green-50 to-white rounded-2xl border border-green-100 shadow-sm transition-all hover:shadow-md">
                        <div>
                            <span
                                class="block text-[10px] font-black text-green-500 uppercase tracking-widest mb-1 text-left">Expected
                                Profit</span>
                            <span class="text-xs text-gray-400 block text-left">(Sell - Buy)</span>
                        </div>
                        <span class="text-2xl font-black text-green-600">{{ currencySymbol }}{{
                            ((product.default_selling_price || 0) - (product.buying_price || 0)).toFixed(2) }}</span>
                    </div>
                </div>

                <!-- Inventory & Brand -->
                <div class="grid grid-cols-2 gap-8">
                    <div class="space-y-4">
                        <h4
                            class="text-[10px] font-black text-gray-400 uppercase tracking-widest border-b border-gray-100 pb-2">
                            Inventory Details</h4>
                        <div class="space-y-3">
                            <div class="flex justify-between items-center text-sm">
                                <span class="text-gray-500">Current Stock</span>
                                <span class="font-bold px-2 py-0.5 rounded bg-gray-100"
                                    :class="product.stock_quantity <= 5 ? 'text-red-600' : 'text-gray-800'">
                                    {{ product.stock_quantity }} {{ product.unit || 'pcs' }}
                                </span>
                            </div>
                            <div class="flex justify-between items-center text-sm">
                                <span class="text-gray-500">Brand</span>
                                <span class="font-medium text-gray-800">{{ product.brand || 'No Brand' }}</span>
                            </div>
                        </div>
                    </div>

                    <div class="space-y-4">
                        <h4
                            class="text-[10px] font-black text-gray-400 uppercase tracking-widest border-b border-gray-100 pb-2">
                            Links & Social</h4>
                        <div class="space-y-3">
                            <div class="flex flex-col gap-2">
                                <a v-if="product.facebook_link" :href="product.facebook_link" target="_blank"
                                    class="flex items-center gap-2 text-sm text-blue-600 hover:text-blue-800 font-medium bg-blue-50 px-3 py-2 rounded-xl transition-colors">
                                    <span class="flex-shrink-0 w-2 h-2 rounded-full bg-blue-500"></span>
                                    Facebook Link
                                </a>
                                <span v-else class="text-xs text-gray-400 italic px-3 py-2">No Facebook Link</span>

                                <a v-if="product.product_link" :href="product.product_link" target="_blank"
                                    class="flex items-center gap-2 text-sm text-indigo-600 hover:text-indigo-800 font-medium bg-indigo-50 px-3 py-2 rounded-xl transition-colors">
                                    <span class="flex-shrink-0 w-2 h-2 rounded-full bg-indigo-500"></span>
                                    Product Page
                                </a>
                                <span v-else class="text-xs text-gray-400 italic px-3 py-2">No Product Link</span>
                            </div>
                        </div>
                    </div>
                </div>

                <!-- Stock History Section -->
                <div class="space-y-4">
                    <div class="flex items-center justify-between border-b border-gray-100 pb-2">
                        <h4 class="text-[10px] font-black text-gray-400 uppercase tracking-widest">Stock History
                            (In/Out)</h4>
                        <span v-if="loadingHistory"
                            class="text-[10px] text-blue-500 animate-pulse font-bold">Updating...</span>
                    </div>

                    <div v-if="loadingHistory && stockHistory.length === 0" class="py-10 flex justify-center">
                        <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
                    </div>

                    <div v-else-if="stockHistory.length === 0"
                        class="py-8 text-center bg-gray-50 rounded-xl border border-dashed border-gray-200">
                        <p class="text-xs text-gray-400 italic">No stock movements recorded yet.</p>
                    </div>

                    <div v-else class="overflow-hidden rounded-xl border border-gray-100 shadow-sm">
                        <table class="w-full text-left border-collapse text-xs">
                            <thead class="bg-gray-50 font-bold text-gray-500 uppercase text-[9px] tracking-tighter">
                                <tr>
                                    <th class="p-3">Date</th>
                                    <th class="p-3 text-center">Type</th>
                                    <th class="p-3">Entity</th>
                                    <th class="p-3 text-center">Qty</th>
                                    <th class="p-3 text-right">Price</th>
                                </tr>
                            </thead>
                            <tbody class="divide-y divide-gray-50 bg-white">
                                <tr v-for="(mv, idx) in stockHistory" :key="idx"
                                    class="hover:bg-gray-50 transition-colors">
                                    <td class="p-3 text-gray-500 font-mono">{{ formatDate(mv.date) }}</td>
                                    <td class="p-3 text-center">
                                        <span
                                            :class="mv.movement_type === 'IN' ? 'bg-green-100 text-green-700' : 'bg-red-100 text-red-700'"
                                            class="px-2 py-0.5 rounded-full font-black text-[9px] uppercase">
                                            {{ mv.movement_type }}
                                        </span>
                                    </td>
                                    <td class="p-3 font-medium text-gray-700 truncate max-w-[120px]">
                                        {{ getEntityName(mv) }}
                                    </td>
                                    <td class="p-3 text-center font-bold">
                                        {{ mv.movement_type === 'IN' ? '+' : '-' }}{{ mv.quantity }}
                                    </td>
                                    <td class="p-3 text-right text-gray-600">
                                        {{ currencySymbol }}{{ mv.price?.toFixed(2) }}
                                    </td>
                                </tr>
                            </tbody>
                        </table>
                    </div>
                </div>
            </div>

            <div class="mt-8 shrink-0 pt-4 border-t border-gray-100 flex justify-end">
                <button @click="emit('close')"
                    class="px-8 py-2.5 bg-gray-900 text-white rounded-xl font-bold text-sm shadow-lg hover:bg-gray-800 transition-all active:scale-95">
                    Close
                </button>
            </div>
        </div>
    </div>
</template>
