<script setup>
import { ref, onMounted, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';

const logs = ref([]);
const loading = ref(false);
const page = ref(0);
const pageSize = 50;
const hasMore = ref(true);
const filterAction = ref('');
const filterEntity = ref('');
const searchQuery = ref('');

const actionTypes = ['CREATE', 'UPDATE', 'DELETE', 'LOGIN', 'BACKUP', 'RESTORE', 'PASSWORD_CHANGE', 'ROLE_CHANGE', 'SETTINGS'];
const entityTypes = ['Product', 'Order', 'Purchase', 'User', 'Settings', 'Backup', 'System'];

const actionColors = {
    CREATE: { bg: 'bg-emerald-100', text: 'text-emerald-700', dot: '🟢' },
    UPDATE: { bg: 'bg-blue-100', text: 'text-blue-700', dot: '🔵' },
    DELETE: { bg: 'bg-red-100', text: 'text-red-700', dot: '🔴' },
    LOGIN: { bg: 'bg-purple-100', text: 'text-purple-700', dot: '🟣' },
    BACKUP: { bg: 'bg-amber-100', text: 'text-amber-700', dot: '🟡' },
    RESTORE: { bg: 'bg-orange-100', text: 'text-orange-700', dot: '🟠' },
    PASSWORD_CHANGE: { bg: 'bg-cyan-100', text: 'text-cyan-700', dot: '🔑' },
    ROLE_CHANGE: { bg: 'bg-indigo-100', text: 'text-indigo-700', dot: '👑' },
    SETTINGS: { bg: 'bg-gray-100', text: 'text-gray-700', dot: '⚙️' },
};

const filteredLogs = computed(() => {
    let result = logs.value;
    if (filterAction.value) result = result.filter(l => l.action === filterAction.value);
    if (filterEntity.value) result = result.filter(l => l.entity_type === filterEntity.value);
    if (searchQuery.value) {
        const q = searchQuery.value.toLowerCase();
        result = result.filter(l =>
            l.description.toLowerCase().includes(q) ||
            l.username.toLowerCase().includes(q)
        );
    }
    return result;
});

async function loadLogs(reset = false) {
    if (reset) { page.value = 0; logs.value = []; hasMore.value = true; }
    try {
        loading.value = true;
        const data = await invoke('get_activity_logs', { limit: pageSize, offset: page.value * pageSize });
        if (data.length < pageSize) hasMore.value = false;
        if (reset) {
            logs.value = data;
        } else {
            logs.value.push(...data);
        }
    } catch (err) {
        console.error('Failed to load logs', err);
    } finally {
        loading.value = false;
    }
}

function loadMore() {
    page.value++;
    loadLogs(false);
}

function getActionStyle(action) {
    return actionColors[action] || { bg: 'bg-gray-100', text: 'text-gray-600', dot: '⚪' };
}

function formatDate(dateStr) {
    if (!dateStr) return '-';
    const d = new Date(dateStr + 'Z');
    const now = new Date();
    const diffMs = now - d;
    const diffMin = Math.floor(diffMs / 60000);
    if (diffMin < 1) return 'Just now';
    if (diffMin < 60) return `${diffMin}m ago`;
    const diffHr = Math.floor(diffMin / 60);
    if (diffHr < 24) return `${diffHr}h ago`;
    return d.toLocaleDateString('en-US', { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' });
}

onMounted(() => loadLogs(true));
</script>

<template>
    <div class="h-full flex flex-col space-y-6">
        <!-- Header -->
        <div class="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-3">
            <div>
                <h1 class="text-3xl font-black text-gray-900 tracking-tight">Activity Log</h1>
                <p class="text-gray-400 text-sm font-medium">Track all system changes and user actions</p>
            </div>
            <button @click="loadLogs(true)" :disabled="loading"
                class="bg-white border border-gray-200 hover:border-gray-300 text-gray-600 px-5 py-2.5 rounded-xl font-bold text-sm transition-all active:scale-95 shadow-sm">
                🔄 Refresh
            </button>
        </div>

        <!-- Filters -->
        <div class="flex flex-wrap gap-3">
            <input v-model="searchQuery" type="text" placeholder="Search by description or user..."
                class="flex-1 min-w-[200px] bg-white border border-gray-200 rounded-xl px-4 py-2.5 text-sm focus:ring-2 focus:ring-blue-500 outline-none">
            <select v-model="filterAction"
                class="bg-white border border-gray-200 rounded-xl px-4 py-2.5 text-sm focus:ring-2 focus:ring-blue-500 outline-none font-bold">
                <option value="">All Actions</option>
                <option v-for="a in actionTypes" :key="a" :value="a">{{ a.replace('_', ' ') }}</option>
            </select>
            <select v-model="filterEntity"
                class="bg-white border border-gray-200 rounded-xl px-4 py-2.5 text-sm focus:ring-2 focus:ring-blue-500 outline-none font-bold">
                <option value="">All Entities</option>
                <option v-for="e in entityTypes" :key="e" :value="e">{{ e }}</option>
            </select>
        </div>

        <!-- Logs List -->
        <div class="bg-white rounded-2xl shadow-sm border border-gray-100 flex-1 flex flex-col min-h-0 overflow-hidden">
            <div class="flex-1 overflow-y-auto">
                <div v-if="filteredLogs.length === 0 && !loading" class="flex items-center justify-center py-20">
                    <div class="text-center">
                        <div class="text-5xl mb-4">📋</div>
                        <p class="text-gray-400 font-bold text-sm">No activity logs yet</p>
                        <p class="text-gray-300 text-xs mt-1">Actions will appear here as you use the system</p>
                    </div>
                </div>

                <div v-else class="divide-y divide-gray-50">
                    <div v-for="log in filteredLogs" :key="log.id"
                        class="flex items-start gap-4 px-6 py-4 hover:bg-gray-50/50 transition-colors group">
                        <!-- Action Icon -->
                        <div class="flex-shrink-0 mt-0.5 text-lg">
                            {{ getActionStyle(log.action).dot }}
                        </div>

                        <!-- Content -->
                        <div class="flex-1 min-w-0">
                            <div class="flex items-center gap-2 flex-wrap">
                                <span class="px-2 py-0.5 rounded-full text-[9px] font-black uppercase tracking-widest"
                                    :class="[getActionStyle(log.action).bg, getActionStyle(log.action).text]">
                                    {{ log.action.replace('_', ' ') }}
                                </span>
                                <span class="text-[10px] font-black text-gray-300 uppercase tracking-widest">{{
                                    log.entity_type }}</span>
                                <span v-if="log.entity_id" class="text-[10px] font-mono text-gray-300">#{{ log.entity_id
                                    }}</span>
                            </div>
                            <p class="text-sm text-gray-700 font-medium mt-1 leading-relaxed">{{ log.description }}</p>
                        </div>

                        <!-- Meta -->
                        <div class="flex-shrink-0 text-right">
                            <div class="text-xs font-bold text-gray-500">{{ log.username }}</div>
                            <div class="text-[10px] text-gray-300 font-mono mt-0.5">{{ formatDate(log.created_at) }}
                            </div>
                        </div>
                    </div>
                </div>

                <!-- Load More -->
                <div v-if="hasMore && filteredLogs.length > 0" class="p-4 text-center">
                    <button @click="loadMore" :disabled="loading"
                        class="text-xs font-black text-blue-600 hover:text-blue-800 uppercase tracking-widest disabled:opacity-50">
                        {{ loading ? 'Loading...' : 'Load More' }}
                    </button>
                </div>
            </div>

            <!-- Footer Stats -->
            <div
                class="border-t border-gray-50 px-6 py-3 bg-gray-50/50 flex items-center justify-between text-[10px] font-black text-gray-400 uppercase tracking-widest">
                <span>Showing {{ filteredLogs.length }} of {{ logs.length }} loaded</span>
                <span>Page {{ page + 1 }}</span>
            </div>
        </div>
    </div>
</template>
