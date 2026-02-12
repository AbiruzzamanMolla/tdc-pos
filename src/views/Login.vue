<script setup>
import { ref, onMounted } from 'vue';
import { useAuthStore } from '../stores/auth';
import { useRouter } from 'vue-router';
import { invoke } from '@tauri-apps/api/core';
import { APP_VERSION } from '../version';

const username = ref('');
const password = ref('');
const confirmPassword = ref('');
const error = ref('');
const loading = ref(false);
const isSetupMode = ref(false);
const checkingSetup = ref(true);

const auth = useAuthStore();
const router = useRouter();

onMounted(async () => {
    try {
        isSetupMode.value = await invoke('check_setup_required');
    } catch (err) {
        console.error('Setup check failed', err);
    } finally {
        checkingSetup.value = false;
    }
});

async function handleLogin() {
    if (!username.value || !password.value) {
        error.value = 'Please enter both username and password';
        return;
    }

    loading.value = true;
    error.value = '';

    try {
        const user = await invoke('login', { username: username.value, password: password.value });
        auth.setUser(user);
        router.push('/');
    } catch (err) {
        error.value = err.toString();
    } finally {
        loading.value = false;
    }
}

async function handleSetup() {
    if (!username.value || !password.value) {
        error.value = 'Please fill in all fields';
        return;
    }
    if (password.value.length < 6) {
        error.value = 'Password must be at least 6 characters';
        return;
    }
    if (password.value !== confirmPassword.value) {
        error.value = 'Passwords do not match';
        return;
    }

    loading.value = true;
    error.value = '';

    try {
        const user = await invoke('setup_admin', { username: username.value, password: password.value });
        auth.setUser(user);
        router.push('/');
    } catch (err) {
        error.value = err.toString();
    } finally {
        loading.value = false;
    }
}
</script>

<template>
    <div class="min-h-screen flex items-center justify-center bg-gray-900 px-4">
        <!-- Loading state -->
        <div v-if="checkingSetup" class="text-gray-400 text-sm font-bold animate-pulse">Initializing...</div>

        <!-- Setup Mode (First Installation) -->
        <div v-else-if="isSetupMode"
            class="max-w-md w-full space-y-6 bg-white p-10 rounded-3xl shadow-2xl animate-in fade-in duration-500">
            <div class="text-center">
                <div
                    class="bg-gradient-to-br from-emerald-500 to-teal-600 text-white w-20 h-20 rounded-2xl flex items-center justify-center text-4xl font-black shadow-lg shadow-emerald-500/30 mx-auto">
                    TDC
                </div>
                <h2 class="mt-6 text-2xl font-black text-gray-900 tracking-tight">
                    Welcome to TDC-POS
                </h2>
                <p class="mt-2 text-sm text-gray-500">
                    Create your Super Admin account to get started
                </p>
                <div class="mt-3 flex items-center justify-center gap-2">
                    <span
                        class="bg-emerald-100 text-emerald-700 px-3 py-1 rounded-full text-[10px] font-black uppercase tracking-widest">First-time
                        Setup</span>
                </div>
            </div>

            <form class="space-y-4 mt-6" @submit.prevent="handleSetup">
                <div>
                    <label class="block text-[10px] font-black text-gray-400 uppercase tracking-widest mb-1.5">Admin
                        Username</label>
                    <input v-model="username" type="text" required placeholder="e.g., admin"
                        class="w-full bg-gray-50 border border-gray-200 rounded-xl px-4 py-3 text-sm focus:ring-2 focus:ring-emerald-500 focus:border-transparent outline-none transition-all">
                </div>
                <div>
                    <label
                        class="block text-[10px] font-black text-gray-400 uppercase tracking-widest mb-1.5">Password</label>
                    <input v-model="password" type="password" required placeholder="Minimum 6 characters"
                        class="w-full bg-gray-50 border border-gray-200 rounded-xl px-4 py-3 text-sm focus:ring-2 focus:ring-emerald-500 focus:border-transparent outline-none transition-all">
                </div>
                <div>
                    <label class="block text-[10px] font-black text-gray-400 uppercase tracking-widest mb-1.5">Confirm
                        Password</label>
                    <input v-model="confirmPassword" type="password" required placeholder="Re-enter password"
                        class="w-full bg-gray-50 border border-gray-200 rounded-xl px-4 py-3 text-sm focus:ring-2 focus:ring-emerald-500 focus:border-transparent outline-none transition-all">
                </div>

                <div v-if="error" class="text-red-500 text-sm bg-red-50 p-3 rounded-xl text-center font-bold">
                    {{ error }}
                </div>

                <button type="submit" :disabled="loading"
                    class="w-full flex justify-center py-3.5 px-4 border border-transparent text-sm font-black rounded-xl text-white bg-gradient-to-r from-emerald-500 to-teal-600 hover:from-emerald-600 hover:to-teal-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-emerald-500 transition-all active:scale-95 disabled:opacity-50 shadow-lg shadow-emerald-500/20 uppercase tracking-widest">
                    <span v-if="loading">Creating Account...</span>
                    <span v-else>Create Super Admin & Start</span>
                </button>
            </form>

            <div class="text-center text-[10px] text-gray-300 font-black uppercase tracking-widest">
                {{ APP_VERSION }}
            </div>
        </div>

        <!-- Normal Login Mode -->
        <div v-else
            class="max-w-md w-full space-y-6 bg-white p-10 rounded-3xl shadow-2xl animate-in fade-in duration-500">
            <div class="text-center">
                <div
                    class="bg-blue-600 text-white w-20 h-20 rounded-2xl flex items-center justify-center text-4xl font-black shadow-lg shadow-blue-500/30 mx-auto">
                    TDC
                </div>
                <h2 class="mt-6 text-2xl font-black text-gray-900 tracking-tight">
                    Welcome back
                </h2>
                <p class="mt-2 text-sm text-gray-500">
                    Sign in to manage your Point of Sale
                </p>
            </div>

            <form class="space-y-4 mt-6" @submit.prevent="handleLogin">
                <div>
                    <label
                        class="block text-[10px] font-black text-gray-400 uppercase tracking-widest mb-1.5">Username</label>
                    <input v-model="username" type="text" required placeholder="Enter your username"
                        class="w-full bg-gray-50 border border-gray-200 rounded-xl px-4 py-3 text-sm focus:ring-2 focus:ring-blue-500 focus:border-transparent outline-none transition-all">
                </div>
                <div>
                    <label
                        class="block text-[10px] font-black text-gray-400 uppercase tracking-widest mb-1.5">Password</label>
                    <input v-model="password" type="password" required placeholder="Enter your password"
                        class="w-full bg-gray-50 border border-gray-200 rounded-xl px-4 py-3 text-sm focus:ring-2 focus:ring-blue-500 focus:border-transparent outline-none transition-all">
                </div>

                <div v-if="error" class="text-red-500 text-sm bg-red-50 p-3 rounded-xl text-center font-bold">
                    {{ error }}
                </div>

                <button type="submit" :disabled="loading"
                    class="w-full flex justify-center py-3.5 px-4 border border-transparent text-sm font-black rounded-xl text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 transition-all active:scale-95 disabled:opacity-50 shadow-lg shadow-blue-500/20 uppercase tracking-widest">
                    <span v-if="loading">Signing in...</span>
                    <span v-else>Sign In</span>
                </button>
            </form>

            <div class="text-center text-[10px] text-gray-300 font-black uppercase tracking-widest">
                {{ APP_VERSION }}
            </div>
        </div>
    </div>
</template>
