<script setup>
import { ref, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import { APP_VERSION } from '../version';

const token = ref('');
const error = ref('');
const loading = ref(false);
const checking = ref(true);
const router = useRouter();

onMounted(async () => {
    // Check if token already exists and validate it softly
    const existingToken = localStorage.getItem('tdc_license_token');
    if (existingToken) {
        token.value = existingToken;
        await checkExistingToken(existingToken);
    } else {
        checking.value = false;
    }
});

async function checkExistingToken(existingToken) {
    try {
        const formData = new FormData();
        formData.append('token', existingToken);

        const response = await fetch('https://audiobookbangla.com/api/tdc/validate', {
            method: 'POST',
            body: formData,
            headers: { 'Accept': 'application/json' }
        });

        if (response.ok) {
            const data = await response.json();
            if (data.status === 'success') {
                router.push('/login');
                return;
            }
        }
        
        // If invalid, clear the localStorage
        localStorage.removeItem('tdc_license_token');
        error.value = 'Your previous license session has expired or is invalid.';
    } catch (err) {
        console.warn('Silent token check failed', err);
    } finally {
        checking.value = false;
    }
}

async function validateToken() {
    if (!token.value) {
        error.value = 'Please enter a license token';
        return;
    }

    loading.value = true;
    error.value = '';

    try {
        const formData = new FormData();
        formData.append('token', token.value);

        const response = await fetch('https://audiobookbangla.com/api/tdc/validate', {
            method: 'POST',
            body: formData,
            headers: {
                'Accept': 'application/json'
            }
        });

        const data = await response.json();

        if (response.ok && data.status === 'success') {
            localStorage.setItem('tdc_license_token', token.value);
            router.push('/login');
        } else {
            // Handle different error status codes based on documentation
            if (response.status === 404 || response.status === 403 || response.status === 401 || data.status === 'error') {
                 error.value = data.message || 'Invalid or expired token.';
            } else {
                 error.value = 'Failed to validate token.';
            }
        }
    } catch (err) {
        error.value = 'Failed to connect to the licensing server. Please check your internet connection.';
    } finally {
        loading.value = false;
    }
}
</script>

<template>
    <div class="min-h-screen flex items-center justify-center bg-gray-900 px-4">
        <!-- Loading state -->
        <div v-if="checking" class="text-gray-400 text-sm font-bold animate-pulse">Initializing...</div>

        <!-- License Validation Mode -->
        <div v-else
            class="max-w-md w-full space-y-5 sm:space-y-6 bg-white p-6 sm:p-10 rounded-3xl shadow-2xl animate-in fade-in duration-500 my-4">
            <div class="text-center">
                <div
                    class="bg-gradient-to-br from-indigo-500 to-purple-600 text-white w-16 h-16 sm:w-20 sm:h-20 rounded-2xl flex items-center justify-center text-3xl sm:text-4xl font-black shadow-lg shadow-indigo-500/30 mx-auto">
                    TDC
                </div>
                <h2 class="mt-4 sm:mt-6 text-xl sm:text-2xl font-black text-gray-900 tracking-tight">
                    Software Activation
                </h2>
                <p class="mt-2 text-xs sm:text-sm text-gray-500">
                    Enter your TDC license token to continue
                </p>
                <div class="mt-3 flex items-center justify-center gap-2">
                    <span
                        class="bg-indigo-100 text-indigo-700 px-3 py-1 rounded-full text-[10px] font-black uppercase tracking-widest">Security Check</span>
                </div>
            </div>

            <form class="space-y-4 mt-6" @submit.prevent="validateToken">
                <div>
                    <label class="block text-[10px] font-black text-gray-400 uppercase tracking-widest mb-1.5">License Token</label>
                    <input v-model="token" type="text" required placeholder="Enter your 32-character token"
                        class="w-full bg-gray-50 border border-gray-200 rounded-xl px-4 py-3 text-sm focus:ring-2 focus:ring-indigo-500 focus:border-transparent outline-none transition-all font-mono tracking-wider">
                </div>

                <div v-if="error" class="text-red-500 text-sm bg-red-50 p-3 rounded-xl text-center font-bold">
                    {{ error }}
                </div>
                <!-- Contact info -->
                <div class="text-center bg-gray-50 rounded-xl p-3 border border-gray-100">
                    <p class="text-xs text-gray-500 font-medium">Don't have a license?</p>
                    <p class="text-xs text-gray-700 font-bold mt-1">Contact: <a href="mailto:abiruzzaman.molla@gmail.com" class="text-indigo-600 hover:text-indigo-500">abiruzzaman.molla@gmail.com</a></p>
                </div>

                <button type="submit" :disabled="loading"
                    class="w-full flex justify-center py-3.5 px-4 border border-transparent text-sm font-black rounded-xl text-white bg-gradient-to-r from-indigo-500 to-purple-600 hover:from-indigo-600 hover:to-purple-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 transition-all active:scale-95 disabled:opacity-50 shadow-lg shadow-indigo-500/20 uppercase tracking-widest">
                    <span v-if="loading">Validating...</span>
                    <span v-else>Activate License</span>
                </button>
            </form>

            <div class="text-center text-[10px] text-gray-300 font-black uppercase tracking-widest">
                {{ APP_VERSION }}
            </div>
        </div>
    </div>
</template>
