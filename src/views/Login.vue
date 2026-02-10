<script setup>
import { ref } from 'vue';
import { useAuthStore } from '../stores/auth';
import { useRouter } from 'vue-router';
import { invoke } from '@tauri-apps/api/core';

const username = ref('');
const password = ref('');
const error = ref('');
const loading = ref(false);

const auth = useAuthStore();
const router = useRouter();

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
</script>

<template>
    <div class="min-h-screen flex items-center justify-center bg-gray-900 px-4">
        <div class="max-w-md w-full space-y-8 bg-white p-10 rounded-2xl shadow-2xl">
            <div>
                <div class="flex justify-center">
                    <div
                        class="bg-blue-600 text-white w-20 h-20 rounded-2xl flex items-center justify-center text-4xl font-black shadow-lg shadow-blue-500/50">
                        TDC
                    </div>
                </div>
                <h2 class="mt-6 text-center text-3xl font-extrabold text-gray-900">
                    Welcome back
                </h2>
                <p class="mt-2 text-center text-sm text-gray-600">
                    Sign in to manage your Point of Sale
                </p>
            </div>
            <form class="mt-8 space-y-6" @submit.prevent="handleLogin">
                <div class="rounded-md shadow-sm -space-y-px">
                    <div>
                        <label for="username" class="sr-only">Username</label>
                        <input v-model="username" id="username" name="username" type="text" required
                            class="appearance-none rounded-none relative block w-full px-3 py-3 border border-gray-300 placeholder-gray-500 text-gray-900 rounded-t-lg focus:outline-none focus:ring-blue-500 focus:border-blue-500 focus:z-10 sm:text-sm"
                            placeholder="Username">
                    </div>
                    <div>
                        <label for="password" class="sr-only">Password</label>
                        <input v-model="password" id="password" name="password" type="password" required
                            class="appearance-none rounded-none relative block w-full px-3 py-3 border border-gray-300 placeholder-gray-500 text-gray-900 rounded-b-lg focus:outline-none focus:ring-blue-500 focus:border-blue-500 focus:z-10 sm:text-sm"
                            placeholder="Password">
                    </div>
                </div>

                <div v-if="error" class="text-red-500 text-sm bg-red-50 p-2 rounded text-center">
                    {{ error }}
                </div>

                <div>
                    <button type="submit" :disabled="loading"
                        class="group relative w-full flex justify-center py-3 px-4 border border-transparent text-sm font-bold rounded-lg text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 transition-all active:scale-95 disabled:opacity-50">
                        <span v-if="loading">Signing in...</span>
                        <span v-else>Sign in</span>
                    </button>
                </div>
            </form>
            <div class="text-center text-xs text-gray-400 mt-4">
                Default: admin / admin123
            </div>
        </div>
    </div>
</template>
