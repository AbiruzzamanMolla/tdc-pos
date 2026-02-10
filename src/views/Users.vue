<script setup>
import { ref, onMounted, reactive } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { useAuthStore } from '../stores/auth';

const auth = useAuthStore();
const users = ref([]);
const loading = ref(false);
const showAddModal = ref(false);

const roles = [
    { value: 'super_admin', label: 'Super Admin' },
    { value: 'admin', label: 'Admin' },
    { value: 'manager', label: 'Manager' },
    { value: 'buy_manager', label: 'Buy Manager' },
    { value: 'sell_manager', label: 'Sell Manager' },
    { value: 'report_checker', label: 'Report Checker' },
    { value: 'inspector', label: 'Inspector' },
    { value: 'worker', label: 'Worker' }
];

const form = reactive({
    username: '',
    password: '',
    role: 'worker'
});

async function loadUsers() {
    try {
        loading.value = true;
        users.value = await invoke('get_users');
    } catch (err) {
        console.error("Failed to load users", err);
    } finally {
        loading.value = false;
    }
}

async function createUser() {
    if (!form.username || !form.password) {
        alert("Please fill all fields");
        return;
    }

    try {
        await invoke('create_user', { user: { ...form } });
        showAddModal.value = false;
        form.username = '';
        form.password = '';
        form.role = 'worker';
        await loadUsers();
    } catch (err) {
        alert("Failed to create user: " + err);
    }
}

async function deleteUser(id, username) {
    if (username === 'admin') {
        alert("Cannot delete primary admin account");
        return;
    }

    if (username === auth.user.username) {
        alert("Cannot delete your own account");
        return;
    }

    if (!confirm(`Are you sure you want to delete user "${username}"?`)) return;

    try {
        await invoke('delete_user', { id });
        await loadUsers();
    } catch (err) {
        alert("Failed to delete user: " + err);
    }
}

onMounted(loadUsers);
</script>

<template>
    <div class="h-full flex flex-col space-y-6">
        <div class="flex justify-between items-center">
            <h1 class="text-3xl font-bold text-gray-800">User Management</h1>
            <button v-if="auth.canManageUsers" @click="showAddModal = true"
                class="bg-blue-600 hover:bg-blue-700 text-white px-6 py-2 rounded-xl shadow-lg shadow-blue-500/30 transition-all font-bold">
                Add New User
            </button>
        </div>

        <div class="bg-white rounded-2xl shadow-sm border border-gray-100 overflow-hidden flex-1 flex flex-col">
            <div class="overflow-x-auto">
                <table class="w-full text-left border-collapse">
                    <thead class="bg-gray-50 text-[10px] font-black text-gray-400 uppercase tracking-widest">
                        <tr>
                            <th class="px-6 py-4 border-b">Username</th>
                            <th class="px-6 py-4 border-b">Role</th>
                            <th class="px-6 py-4 border-b">Created At</th>
                            <th class="px-6 py-4 border-b text-right">Actions</th>
                        </tr>
                    </thead>
                    <tbody class="text-gray-700 divide-y divide-gray-50">
                        <tr v-for="user in users" :key="user.id" class="hover:bg-blue-50/20 transition-colors">
                            <td class="px-6 py-4">
                                <div class="flex items-center space-x-3">
                                    <div
                                        class="w-8 h-8 rounded-full bg-gray-200 flex items-center justify-center text-xs font-bold text-gray-500">
                                        {{ user.username.charAt(0).toUpperCase() }}
                                    </div>
                                    <span class="font-bold">{{ user.username }}</span>
                                </div>
                            </td>
                            <td class="px-6 py-4">
                                <span class="px-2 py-1 rounded-full text-[10px] font-black uppercase tracking-wider"
                                    :class="{
                                        'bg-purple-100 text-purple-700': user.role === 'super_admin',
                                        'bg-blue-100 text-blue-700': user.role === 'admin',
                                        'bg-green-100 text-green-700': user.role === 'manager',
                                        'bg-orange-100 text-orange-700': ['buy_manager', 'sell_manager'].includes(user.role),
                                        'bg-gray-100 text-gray-600': user.role === 'worker'
                                    }">
                                    {{ user.role.replace('_', ' ') }}
                                </span>
                            </td>
                            <td class="px-6 py-4 text-xs font-mono text-gray-400">{{ user.created_at || '-' }}</td>
                            <td class="px-6 py-4 text-right">
                                <button v-if="user.username !== 'admin' && user.username !== auth.user.username"
                                    @click="deleteUser(user.id, user.username)"
                                    class="text-red-500 hover:text-red-700 text-xs font-black uppercase tracking-widest">
                                    Delete
                                </button>
                                <span v-else class="text-[10px] text-gray-300 font-black uppercase">System</span>
                            </td>
                        </tr>
                        <tr v-if="users.length === 0 && !loading">
                            <td colspan="4" class="p-10 text-center text-gray-400 italic">No users found.</td>
                        </tr>
                    </tbody>
                </table>
            </div>
        </div>

        <!-- Add User Modal -->
        <div v-if="showAddModal"
            class="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-50 p-4">
            <div class="bg-white rounded-2xl shadow-2xl w-full max-w-md p-8 relative">
                <button @click="showAddModal = false"
                    class="absolute top-4 right-4 text-gray-400 hover:text-gray-600">✕</button>
                <h2 class="text-2xl font-black text-gray-800 mb-6 uppercase tracking-tight italic">Create User</h2>

                <form @submit.prevent="createUser" class="space-y-4">
                    <div>
                        <label
                            class="block text-[10px] font-black text-gray-400 uppercase tracking-widest mb-1.5">Username</label>
                        <input v-model="form.username" type="text" required
                            class="w-full bg-gray-50 border border-gray-200 rounded-xl px-4 py-3 text-sm focus:ring-2 focus:ring-blue-500 outline-none transition-all">
                    </div>
                    <div>
                        <label
                            class="block text-[10px] font-black text-gray-400 uppercase tracking-widest mb-1.5">Password</label>
                        <input v-model="form.password" type="password" required
                            class="w-full bg-gray-50 border border-gray-200 rounded-xl px-4 py-3 text-sm focus:ring-2 focus:ring-blue-500 outline-none transition-all">
                    </div>
                    <div>
                        <label
                            class="block text-[10px] font-black text-gray-400 uppercase tracking-widest mb-1.5">Role</label>
                        <select v-model="form.role"
                            class="w-full bg-gray-50 border border-gray-200 rounded-xl px-4 py-3 text-sm focus:ring-2 focus:ring-blue-500 outline-none transition-all">
                            <option v-for="role in roles" :key="role.value" :value="role.value">{{ role.label }}
                            </option>
                        </select>
                    </div>

                    <div class="pt-4">
                        <button type="submit"
                            class="w-full bg-blue-600 text-white font-black py-4 rounded-xl shadow-lg shadow-blue-500/20 hover:bg-blue-700 transition-all active:scale-95 uppercase tracking-widest text-xs">
                            Confirm & Save
                        </button>
                    </div>
                </form>
            </div>
        </div>
    </div>
</template>
