<script setup>
import { ref, onMounted, reactive } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { useAuthStore } from '../stores/auth';
import { logActivity } from '../utils/activityLogger';

const auth = useAuthStore();
const users = ref([]);
const loading = ref(false);
const showAddModal = ref(false);
const showPasswordModal = ref(false);
const showRoleModal = ref(false);
const statusMsg = ref('');
const statusType = ref('');

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

const form = reactive({ username: '', password: '', role: 'worker' });

// Password change state
const passwordForm = reactive({
    targetUserId: null,
    targetUsername: '',
    currentPassword: '',
    newPassword: '',
    confirmPassword: '',
    isSelfChange: false
});

// Role change state
const roleForm = reactive({
    targetUserId: null,
    targetUsername: '',
    newRole: ''
});

function showStatus(msg, type = 'success') {
    statusMsg.value = msg;
    statusType.value = type;
    setTimeout(() => { statusMsg.value = ''; }, 4000);
}

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
        showStatus('Please fill all fields', 'error');
        return;
    }
    if (form.password.length < 6) {
        showStatus('Password must be at least 6 characters', 'error');
        return;
    }

    try {
        await invoke('create_user', { user: { ...form } });
        await logActivity('CREATE', 'User', null, `Created user: ${form.username} (role: ${form.role})`);
        showAddModal.value = false;
        form.username = '';
        form.password = '';
        form.role = 'worker';
        await loadUsers();
        showStatus('User created successfully');
    } catch (err) {
        showStatus('Failed: ' + err, 'error');
    }
}

async function deleteUser(id, username) {
    if (username === auth.user.username) {
        showStatus('Cannot delete your own account', 'error');
        return;
    }

    if (!confirm(`Are you sure you want to delete user "${username}"?`)) return;

    try {
        await invoke('delete_user', { id });
        await logActivity('DELETE', 'User', id, `Deleted user: ${username}`);
        await loadUsers();
        showStatus('User deleted');
    } catch (err) {
        showStatus('Failed: ' + err, 'error');
    }
}

// --- Password Change ---
function openPasswordModal(user, isSelf) {
    passwordForm.targetUserId = user.id;
    passwordForm.targetUsername = user.username;
    passwordForm.currentPassword = '';
    passwordForm.newPassword = '';
    passwordForm.confirmPassword = '';
    passwordForm.isSelfChange = isSelf;
    showPasswordModal.value = true;
}

async function handleChangePassword() {
    if (!passwordForm.newPassword || passwordForm.newPassword.length < 6) {
        showStatus('New password must be at least 6 characters', 'error');
        return;
    }
    if (passwordForm.newPassword !== passwordForm.confirmPassword) {
        showStatus('Passwords do not match', 'error');
        return;
    }
    if (passwordForm.isSelfChange && !passwordForm.currentPassword) {
        showStatus('Current password is required', 'error');
        return;
    }

    try {
        await invoke('change_password', {
            userId: passwordForm.targetUserId,
            currentPassword: passwordForm.currentPassword || '',
            newPassword: passwordForm.newPassword,
            isSuperAdmin: auth.isSuperAdmin && !passwordForm.isSelfChange
        });
        showPasswordModal.value = false;
        await logActivity('PASSWORD_CHANGE', 'User', passwordForm.targetUserId, `Password changed for: ${passwordForm.targetUsername}`);
        showStatus(`Password updated for ${passwordForm.targetUsername}`);
    } catch (err) {
        showStatus(err.toString(), 'error');
    }
}

// --- Role Change ---
function openRoleModal(user) {
    roleForm.targetUserId = user.id;
    roleForm.targetUsername = user.username;
    roleForm.newRole = user.role;
    showRoleModal.value = true;
}

async function handleRoleChange() {
    try {
        await invoke('update_user_role', {
            userId: roleForm.targetUserId,
            newRole: roleForm.newRole
        });
        showRoleModal.value = false;
        await logActivity('ROLE_CHANGE', 'User', roleForm.targetUserId, `Role changed for ${roleForm.targetUsername} to ${roleForm.newRole}`);
        await loadUsers();
        showStatus(`Role updated for ${roleForm.targetUsername}`);
    } catch (err) {
        showStatus('Failed: ' + err, 'error');
    }
}

function getRoleBadgeClass(role) {
    const map = {
        'super_admin': 'bg-purple-100 text-purple-700',
        'admin': 'bg-blue-100 text-blue-700',
        'manager': 'bg-green-100 text-green-700',
        'buy_manager': 'bg-orange-100 text-orange-700',
        'sell_manager': 'bg-orange-100 text-orange-700',
        'report_checker': 'bg-teal-100 text-teal-700',
        'inspector': 'bg-cyan-100 text-cyan-700',
        'worker': 'bg-gray-100 text-gray-600'
    };
    return map[role] || 'bg-gray-100 text-gray-600';
}

onMounted(loadUsers);
</script>

<template>
    <div class="h-full flex flex-col space-y-6">
        <div class="flex justify-between items-center">
            <div>
                <h1 class="text-3xl font-black text-gray-900 tracking-tight">User Management</h1>
                <p class="text-gray-400 text-sm font-medium">Manage accounts, roles & security</p>
            </div>
            <div class="flex gap-3">
                <!-- Self password change button  -->
                <button @click="openPasswordModal(auth.user, true)"
                    class="bg-white border border-gray-200 hover:border-blue-500 hover:text-blue-600 text-gray-600 px-5 py-2.5 rounded-xl font-bold text-sm transition-all active:scale-95 shadow-sm">
                    🔑 Change My Password
                </button>
                <button v-if="auth.canManageUsers" @click="showAddModal = true"
                    class="bg-gradient-to-r from-blue-600 to-indigo-600 hover:from-blue-700 hover:to-indigo-700 text-white px-6 py-2.5 rounded-xl shadow-lg shadow-blue-500/20 transition-all font-bold text-sm active:scale-95">
                    + Add User
                </button>
            </div>
        </div>

        <!-- Status Toast -->
        <div v-if="statusMsg"
            class="p-3 rounded-xl text-sm font-bold flex items-center gap-2 animate-in slide-in-from-top-4 duration-300"
            :class="statusType === 'error' ? 'bg-red-50 text-red-700 border border-red-200' : 'bg-green-50 text-green-700 border border-green-200'">
            <span>{{ statusType === 'error' ? '❌' : '✅' }}</span> {{ statusMsg }}
        </div>

        <!-- User Table -->
        <div class="bg-white rounded-2xl shadow-sm border border-gray-100 overflow-hidden flex-1 flex flex-col">
            <div class="overflow-x-auto">
                <table class="w-full text-left border-collapse">
                    <thead class="bg-gray-50 text-[10px] font-black text-gray-400 uppercase tracking-widest">
                        <tr>
                            <th class="px-6 py-4 border-b border-gray-100">User</th>
                            <th class="px-6 py-4 border-b border-gray-100">Role</th>
                            <th class="px-6 py-4 border-b border-gray-100">Created</th>
                            <th class="px-6 py-4 border-b border-gray-100 text-right">Actions</th>
                        </tr>
                    </thead>
                    <tbody class="text-gray-700 divide-y divide-gray-50">
                        <tr v-for="user in users" :key="user.id" class="hover:bg-blue-50/20 transition-colors group">
                            <td class="px-6 py-4">
                                <div class="flex items-center space-x-3">
                                    <div class="w-9 h-9 rounded-full flex items-center justify-center text-xs font-black text-white"
                                        :class="user.role === 'super_admin' ? 'bg-gradient-to-br from-purple-500 to-indigo-600' : 'bg-gray-300'">
                                        {{ user.username.charAt(0).toUpperCase() }}
                                    </div>
                                    <div>
                                        <span class="font-bold text-gray-800">{{ user.username }}</span>
                                        <span v-if="user.username === auth.user?.username"
                                            class="ml-2 text-[9px] font-black text-blue-500 bg-blue-50 px-1.5 py-0.5 rounded-full uppercase">You</span>
                                    </div>
                                </div>
                            </td>
                            <td class="px-6 py-4">
                                <span class="px-2.5 py-1 rounded-full text-[10px] font-black uppercase tracking-wider"
                                    :class="getRoleBadgeClass(user.role)">
                                    {{ user.role.replace(/_/g, ' ') }}
                                </span>
                            </td>
                            <td class="px-6 py-4 text-xs font-mono text-gray-400">{{ user.created_at || '-' }}</td>
                            <td class="px-6 py-4 text-right">
                                <div
                                    class="flex items-center justify-end gap-2 opacity-0 group-hover:opacity-100 transition-opacity">
                                    <!-- Super admin can change any user's password -->
                                    <button v-if="auth.isSuperAdmin" @click="openPasswordModal(user, false)"
                                        class="text-[10px] font-black text-blue-500 hover:text-blue-700 bg-blue-50 hover:bg-blue-100 px-2.5 py-1.5 rounded-lg uppercase tracking-widest transition-colors">
                                        Password
                                    </button>
                                    <!-- Super admin can change roles -->
                                    <button v-if="auth.isSuperAdmin && user.username !== auth.user?.username"
                                        @click="openRoleModal(user)"
                                        class="text-[10px] font-black text-purple-500 hover:text-purple-700 bg-purple-50 hover:bg-purple-100 px-2.5 py-1.5 rounded-lg uppercase tracking-widest transition-colors">
                                        Role
                                    </button>
                                    <!-- Delete -->
                                    <button v-if="auth.canManageUsers && user.username !== auth.user?.username"
                                        @click="deleteUser(user.id, user.username)"
                                        class="text-[10px] font-black text-red-400 hover:text-red-600 bg-red-50 hover:bg-red-100 px-2.5 py-1.5 rounded-lg uppercase tracking-widest transition-colors">
                                        Delete
                                    </button>
                                </div>
                            </td>
                        </tr>
                        <tr v-if="users.length === 0 && !loading">
                            <td colspan="4" class="p-16 text-center">
                                <div class="text-4xl mb-2">👥</div>
                                <div class="text-gray-400 font-bold text-sm">No users found</div>
                            </td>
                        </tr>
                    </tbody>
                </table>
            </div>
        </div>

        <!-- Add User Modal -->
        <div v-if="showAddModal"
            class="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-50 p-4">
            <div
                class="bg-white rounded-2xl shadow-2xl w-full max-w-md p-8 relative animate-in zoom-in-95 duration-200">
                <button @click="showAddModal = false"
                    class="absolute top-4 right-4 text-gray-400 hover:text-gray-600 text-lg">✕</button>
                <h2 class="text-xl font-black text-gray-900 mb-6 uppercase tracking-tight">Create User</h2>

                <form @submit.prevent="createUser" class="space-y-4">
                    <div>
                        <label
                            class="block text-[10px] font-black text-gray-400 uppercase tracking-widest mb-1.5">Username</label>
                        <input v-model="form.username" type="text" required
                            class="w-full bg-gray-50 border border-gray-200 rounded-xl px-4 py-3 text-sm focus:ring-2 focus:ring-blue-500 outline-none transition-all">
                    </div>
                    <div>
                        <label
                            class="block text-[10px] font-black text-gray-400 uppercase tracking-widest mb-1.5">Password
                            (min 6 chars)</label>
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
                    <div class="pt-2">
                        <button type="submit"
                            class="w-full bg-blue-600 text-white font-black py-3.5 rounded-xl shadow-lg shadow-blue-500/20 hover:bg-blue-700 transition-all active:scale-95 uppercase tracking-widest text-xs">
                            Confirm & Save
                        </button>
                    </div>
                </form>
            </div>
        </div>

        <!-- Change Password Modal -->
        <div v-if="showPasswordModal"
            class="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-50 p-4">
            <div
                class="bg-white rounded-2xl shadow-2xl w-full max-w-md p-8 relative animate-in zoom-in-95 duration-200">
                <button @click="showPasswordModal = false"
                    class="absolute top-4 right-4 text-gray-400 hover:text-gray-600 text-lg">✕</button>
                <h2 class="text-xl font-black text-gray-900 mb-1 uppercase tracking-tight">Change Password</h2>
                <p class="text-gray-400 text-xs font-bold mb-6">For user: <span class="text-blue-600">{{
                    passwordForm.targetUsername }}</span></p>

                <form @submit.prevent="handleChangePassword" class="space-y-4">
                    <div v-if="passwordForm.isSelfChange">
                        <label
                            class="block text-[10px] font-black text-gray-400 uppercase tracking-widest mb-1.5">Current
                            Password</label>
                        <input v-model="passwordForm.currentPassword" type="password" required
                            class="w-full bg-gray-50 border border-gray-200 rounded-xl px-4 py-3 text-sm focus:ring-2 focus:ring-blue-500 outline-none transition-all">
                    </div>
                    <div v-else
                        class="bg-amber-50 border border-amber-200 p-3 rounded-xl text-xs text-amber-700 font-bold">
                        ⚡ Super Admin override — no current password required.
                    </div>
                    <div>
                        <label class="block text-[10px] font-black text-gray-400 uppercase tracking-widest mb-1.5">New
                            Password (min 6 chars)</label>
                        <input v-model="passwordForm.newPassword" type="password" required
                            class="w-full bg-gray-50 border border-gray-200 rounded-xl px-4 py-3 text-sm focus:ring-2 focus:ring-blue-500 outline-none transition-all">
                    </div>
                    <div>
                        <label
                            class="block text-[10px] font-black text-gray-400 uppercase tracking-widest mb-1.5">Confirm
                            New Password</label>
                        <input v-model="passwordForm.confirmPassword" type="password" required
                            class="w-full bg-gray-50 border border-gray-200 rounded-xl px-4 py-3 text-sm focus:ring-2 focus:ring-blue-500 outline-none transition-all">
                    </div>
                    <div class="pt-2">
                        <button type="submit"
                            class="w-full bg-blue-600 text-white font-black py-3.5 rounded-xl shadow-lg shadow-blue-500/20 hover:bg-blue-700 transition-all active:scale-95 uppercase tracking-widest text-xs">
                            Update Password
                        </button>
                    </div>
                </form>
            </div>
        </div>

        <!-- Role Change Modal -->
        <div v-if="showRoleModal"
            class="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-50 p-4">
            <div
                class="bg-white rounded-2xl shadow-2xl w-full max-w-md p-8 relative animate-in zoom-in-95 duration-200">
                <button @click="showRoleModal = false"
                    class="absolute top-4 right-4 text-gray-400 hover:text-gray-600 text-lg">✕</button>
                <h2 class="text-xl font-black text-gray-900 mb-1 uppercase tracking-tight">Change Role</h2>
                <p class="text-gray-400 text-xs font-bold mb-6">For user: <span class="text-purple-600">{{
                    roleForm.targetUsername }}</span></p>

                <form @submit.prevent="handleRoleChange" class="space-y-4">
                    <div>
                        <label class="block text-[10px] font-black text-gray-400 uppercase tracking-widest mb-1.5">New
                            Role</label>
                        <select v-model="roleForm.newRole"
                            class="w-full bg-gray-50 border border-gray-200 rounded-xl px-4 py-3 text-sm focus:ring-2 focus:ring-purple-500 outline-none transition-all">
                            <option v-for="role in roles" :key="role.value" :value="role.value">{{ role.label }}
                            </option>
                        </select>
                    </div>
                    <div class="pt-2">
                        <button type="submit"
                            class="w-full bg-purple-600 text-white font-black py-3.5 rounded-xl shadow-lg shadow-purple-500/20 hover:bg-purple-700 transition-all active:scale-95 uppercase tracking-widest text-xs">
                            Update Role
                        </button>
                    </div>
                </form>
            </div>
        </div>
    </div>
</template>
