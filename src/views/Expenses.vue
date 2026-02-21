<script setup>
import { ref, onMounted, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { logActivity } from '../utils/activityLogger';

const expenses = ref([]);
const loading = ref(false);
const viewMode = ref('history'); // 'history' or 'new'
const editingExpenseId = ref(null);

const startDate = ref(new Date().toISOString().split('T')[0]);
const endDate = ref(new Date().toISOString().split('T')[0]);
const searchQuery = ref('');

const form = ref({
    expense_date: new Date().toISOString().split('T')[0],
    category: 'Other',
    amount: 0,
    notes: ''
});

const categories = [
    "Transport / Travel",
    "Packaging / Materials",
    "Utility Bills (Internet/Phone)",
    "Rent",
    "Meals / Entertainment",
    "Office Supplies",
    "Marketing / Ads",
    "Salary / Wages",
    "Other"
];

const filteredExpenses = computed(() => {
    if (!searchQuery.value) return expenses.value;
    const q = searchQuery.value.toLowerCase();
    return expenses.value.filter(item =>
        item.category.toLowerCase().includes(q) ||
        (item.notes && item.notes.toLowerCase().includes(q))
    );
});

const totalExpenses = computed(() => {
    return filteredExpenses.value.reduce((sum, item) => sum + item.amount, 0);
});

async function loadExpenses() {
    loading.value = true;
    try {
        const data = await invoke('get_expenses', {
            startDate: startDate.value,
            endDate: endDate.value
        });
        expenses.value = data;
    } catch (error) {
        console.error("Failed to load expenses:", error);
        alert("Error loading expenses.");
    } finally {
        loading.value = false;
    }
}

function setDatePreset(preset) {
    const today = new Date();
    endDate.value = today.toISOString().split('T')[0];
    if (preset === 'today') {
        startDate.value = endDate.value;
    } else if (preset === 'week') {
        const d = new Date(today);
        d.setDate(d.getDate() - 6);
        startDate.value = d.toISOString().split('T')[0];
    } else if (preset === 'month') {
        startDate.value = new Date(today.getFullYear(), today.getMonth(), 1).toISOString().split('T')[0];
    } else if (preset === 'year') {
        startDate.value = new Date(today.getFullYear(), 0, 1).toISOString().split('T')[0];
    } else if (preset === 'all') {
        startDate.value = '2020-01-01'; // Far past
    }
    loadExpenses();
}

async function saveExpense() {
    if (form.value.amount <= 0) {
        alert("Amount must be greater than 0");
        return;
    }

    try {
        loading.value = true;
        if (editingExpenseId.value) {
            await invoke('update_expense', { id: editingExpenseId.value, expense: form.value });
            await logActivity('UPDATE', 'Expense', editingExpenseId.value, `Updated expense: ${form.value.category}`);
        } else {
            await invoke('create_expense', { expense: form.value });
            await logActivity('CREATE', 'Expense', null, `Added new expense: ${form.value.category} / ${form.value.amount}`);
        }

        resetForm();
        viewMode.value = 'history';
        loadExpenses();
    } catch (error) {
        console.error("Failed to save expense:", error);
        alert("Error saving expense: " + error);
    } finally {
        loading.value = false;
    }
}

function editExpense(expense) {
    editingExpenseId.value = expense.id;
    form.value = {
        expense_date: expense.expense_date ? expense.expense_date.split('T')[0] : new Date().toISOString().split('T')[0],
        category: expense.category,
        amount: expense.amount,
        notes: expense.notes || ''
    };
    viewMode.value = 'new';
}

async function deleteExpense(id) {
    if (!confirm("Are you sure you want to delete this expense record?")) return;

    try {
        loading.value = true;
        await invoke('delete_expense', { id });
        await logActivity('DELETE', 'Expense', id, `Deleted expense record #${id}`);
        loadExpenses();
    } catch (error) {
        console.error("Failed to delete expense:", error);
        alert("Failed to delete expense: " + error);
    } finally {
        loading.value = false;
    }
}

function resetForm() {
    editingExpenseId.value = null;
    form.value = {
        expense_date: new Date().toISOString().split('T')[0],
        category: 'Other',
        amount: 0,
        notes: ''
    };
}

onMounted(() => {
    setDatePreset('month'); // Default to current month
});
</script>

<template>
    <div class="h-full flex flex-col space-y-6 animate-in fade-in duration-300">
        <!-- Header -->
        <div class="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-3">
            <div>
                <h1 class="text-3xl font-black text-gray-900 tracking-tight">Business Expenses</h1>
                <p class="text-gray-400 text-sm font-medium">Track your operational costs and outflows</p>
            </div>

            <div class="flex gap-2">
                <button v-if="viewMode === 'history'" @click="resetForm(); viewMode = 'new'"
                    class="bg-rose-600 hover:bg-rose-700 text-white px-6 py-2.5 rounded-xl shadow-lg shadow-rose-600/20 font-bold text-sm transition-all active:scale-95 flex items-center gap-2">
                    <span>+</span> Add Expense
                </button>
                <button v-if="viewMode === 'new'" @click="resetForm(); viewMode = 'history'"
                    class="bg-gray-200 hover:bg-gray-300 text-gray-800 px-6 py-2.5 rounded-xl font-bold text-sm transition-all active:scale-95">
                    Cancel
                </button>
            </div>
        </div>

        <!-- MAIN VIEW -->
        <div v-if="viewMode === 'history'" class="flex flex-col flex-1 min-h-0 space-y-4">

            <!-- Controls -->
            <div
                class="bg-white p-4 rounded-2xl shadow-sm border border-gray-100 flex flex-col md:flex-row gap-4 justify-between items-center">
                <!-- Date Filters -->
                <div class="flex flex-wrap gap-2 items-center w-full md:w-auto">
                    <div class="flex bg-gray-50 border border-gray-200 rounded-xl overflow-hidden">
                        <button
                            v-for="p in [{ label: 'Today', key: 'today' }, { label: 'Week', key: 'week' }, { label: 'Month', key: 'month' }, { label: 'Year', key: 'year' }, { label: 'All', key: 'all' }]"
                            :key="p.key" @click="setDatePreset(p.key)"
                            class="px-3 py-1.5 text-[10px] font-black uppercase tracking-widest hover:bg-rose-50 hover:text-rose-600 text-gray-500 transition-colors border-r border-gray-200 last:border-r-0"
                            :class="{ 'bg-rose-50 text-rose-600': false }">
                            <!-- Can add active state logic here based on current dates -->
                            {{ p.label }}
                        </button>
                    </div>
                    <input v-model="startDate" type="date"
                        class="border border-gray-200 rounded-lg px-2 py-1.5 text-xs bg-gray-50 outline-none focus:ring-1 focus:ring-rose-500">
                    <span class="text-gray-300 text-xs font-bold">→</span>
                    <input v-model="endDate" type="date"
                        class="border border-gray-200 rounded-lg px-2 py-1.5 text-xs bg-gray-50 outline-none focus:ring-1 focus:ring-rose-500">
                    <button @click="loadExpenses"
                        class="bg-gray-900 text-white px-4 py-1.5 rounded-lg hover:bg-gray-800 text-xs font-bold transition-colors active:scale-95">Filter</button>
                </div>

                <div class="relative w-full md:w-64">
                    <span class="absolute left-3 top-2.5 text-gray-400 text-sm">🔍</span>
                    <input v-model="searchQuery" type="text" placeholder="Search expenses..."
                        class="w-full border border-gray-200 rounded-xl pl-9 pr-3 py-2 text-sm focus:ring-2 focus:ring-rose-500 focus:border-transparent outline-none bg-gray-50 transition-all">
                </div>
            </div>

            <!-- KPI Summary -->
            <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                <div class="bg-rose-50 border border-rose-100 p-4 rounded-2xl flex items-center justify-between">
                    <div>
                        <div class="text-[10px] font-black text-rose-500 uppercase tracking-widest">Total Expenses</div>
                        <div class="text-2xl font-black text-rose-800 mt-1">৳{{ totalExpenses.toLocaleString(undefined,
                            { minimumFractionDigits: 2}) }}</div>
                    </div>
                    <div class="text-3xl opacity-50">💸</div>
                </div>
                <div class="bg-gray-50 border border-gray-200 p-4 rounded-2xl flex items-center justify-between">
                    <div>
                        <div class="text-[10px] font-black text-gray-500 uppercase tracking-widest">Expense Count</div>
                        <div class="text-xl font-black text-gray-800 mt-1">{{ filteredExpenses.length }} <span
                                class="text-sm font-medium text-gray-500">records</span></div>
                    </div>
                    <div class="text-3xl opacity-50">🧾</div>
                </div>
            </div>

            <!-- Table View -->
            <div class="bg-white rounded-2xl shadow-sm border border-gray-100 flex-1 overflow-auto">
                <table class="w-full text-left min-w-[700px]">
                    <thead
                        class="bg-gray-50 text-[10px] font-black text-gray-400 uppercase tracking-widest sticky top-0 z-10">
                        <tr>
                            <th class="px-5 py-4 border-b border-gray-100">Date</th>
                            <th class="px-5 py-4 border-b border-gray-100">Category</th>
                            <th class="px-5 py-4 border-b border-gray-100 w-1/3">Notes / Description</th>
                            <th class="px-5 py-4 border-b border-gray-100 text-right">Amount</th>
                            <th class="px-5 py-4 border-b border-gray-100 text-right">Actions</th>
                        </tr>
                    </thead>
                    <tbody class="text-sm text-gray-700">
                        <tr v-if="loading" class="animate-pulse">
                            <td colspan="5" class="px-5 py-8 text-center text-gray-400 font-bold">Loading...</td>
                        </tr>
                        <tr v-else-if="filteredExpenses.length === 0">
                            <td colspan="5" class="px-5 py-12 text-center">
                                <div class="text-gray-300 text-4xl mb-2">🎈</div>
                                <div class="font-bold text-gray-500">No expenses recorded</div>
                                <div class="text-xs text-gray-400 mt-1">Try adding a new expense or adjust date filters
                                </div>
                            </td>
                        </tr>
                        <tr v-for="expense in filteredExpenses" :key="expense.id"
                            class="border-b border-gray-50 last:border-b-0 hover:bg-rose-50/30 transition-colors">
                            <td class="px-5 py-3 text-gray-500 text-xs font-mono">{{ expense.expense_date?.split('T')[0]
                                }}</td>
                            <td class="px-5 py-3 font-bold text-gray-800">{{ expense.category }}</td>
                            <td class="px-5 py-3 text-gray-600 text-xs">{{ expense.notes || '—' }}</td>
                            <td class="px-5 py-3 text-right font-black text-rose-600 text-lg">৳{{
                                expense.amount.toFixed(2) }}</td>
                            <td class="px-5 py-3 text-right space-x-2">
                                <button @click="editExpense(expense)"
                                    class="text-xs font-black text-blue-500 hover:bg-blue-50 px-3 py-1.5 rounded-lg transition-colors">EDIT</button>
                                <button @click="deleteExpense(expense.id)"
                                    class="text-xs font-black text-red-500 hover:bg-red-50 px-3 py-1.5 rounded-lg transition-colors">DEL</button>
                            </td>
                        </tr>
                    </tbody>
                </table>
            </div>
        </div>

        <!-- NEW / EDIT VIEW -->
        <div v-if="viewMode === 'new'"
            class="bg-white p-8 rounded-2xl shadow-sm border border-gray-100 max-w-2xl mx-auto w-full">
            <h2 class="text-xl font-bold text-gray-800 mb-6 border-b pb-2">
                {{ editingExpenseId ? 'Edit Expense Record' : 'Record New Expense' }}
            </h2>

            <div class="space-y-5">
                <div class="grid grid-cols-2 gap-5">
                    <div>
                        <label
                            class="block text-xs font-bold text-gray-700 uppercase tracking-widest mb-1.5">Date</label>
                        <input v-model="form.expense_date" type="date"
                            class="w-full bg-gray-50 border border-gray-200 rounded-xl px-4 py-3 text-sm focus:ring-2 focus:ring-rose-500 outline-none transition-all">
                    </div>
                    <div>
                        <label class="block text-xs font-bold text-gray-700 uppercase tracking-widest mb-1.5">Amount
                            (৳)</label>
                        <input v-model.number="form.amount" type="number" min="0" step="0.01" placeholder="0.00"
                            class="w-full bg-gray-50 border border-gray-200 rounded-xl px-4 py-3 text-sm focus:ring-2 focus:ring-rose-500 outline-none transition-all font-black text-rose-600">
                    </div>
                </div>

                <div>
                    <label
                        class="block text-xs font-bold text-gray-700 uppercase tracking-widest mb-1.5">Category</label>
                    <select v-model="form.category"
                        class="w-full bg-gray-50 border border-gray-200 rounded-xl px-4 py-3 text-sm focus:ring-2 focus:ring-rose-500 outline-none transition-all appearance-none cursor-pointer">
                        <option v-for="cat in categories" :key="cat" :value="cat">{{ cat }}</option>
                    </select>
                </div>

                <div>
                    <label class="block text-xs font-bold text-gray-700 uppercase tracking-widest mb-1.5">Description /
                        Notes</label>
                    <textarea v-model="form.notes" rows="3" placeholder="Additional details..."
                        class="w-full bg-gray-50 border border-gray-200 rounded-xl px-4 py-3 text-sm focus:ring-2 focus:ring-rose-500 outline-none transition-all resize-none"></textarea>
                </div>

                <div class="pt-4 flex justify-end">
                    <button @click="saveExpense" :disabled="loading || form.amount <= 0"
                        class="bg-gray-900 hover:bg-gray-800 disabled:opacity-50 text-white px-8 py-3 rounded-xl font-bold flex items-center transition-transform active:scale-95 shadow-lg">
                        {{ loading ? 'Saving...' : (editingExpenseId ? 'Update Record' : 'Save Expense') }}
                    </button>
                </div>
            </div>

        </div>
    </div>
</template>
