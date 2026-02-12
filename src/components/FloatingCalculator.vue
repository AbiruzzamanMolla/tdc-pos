<script setup>
import { ref, onMounted, onBeforeUnmount, watch } from 'vue'

const isOpen = ref(false)
const displayValue = ref('0')
const previousInput = ref(null)
const operator = ref(null)
const operatorClicked = ref(false)
const history = ref('')

// Check if there is active data
const isDirty = () => {
    return (displayValue.value !== '0' && displayValue.value !== '') || previousInput.value !== null
}

const append = (number) => {
    if (operatorClicked.value) {
        displayValue.value = ''
        operatorClicked.value = false
    }
    if (displayValue.value === '0' && number !== '.') {
        displayValue.value = number
    } else {
        // Prevent multiple decimals
        if (number === '.' && displayValue.value.includes('.')) return

        displayValue.value = `${displayValue.value}${number}`
    }
}

const setOperation = (op) => {
    if (previousInput.value !== null && !operatorClicked.value) {
        calculate()
    }
    previousInput.value = displayValue.value
    operator.value = op
    operatorClicked.value = true
    history.value = `${previousInput.value} ${op}`
}

const calculate = () => {
    if (operator.value === null || previousInput.value === null) return

    let result = 0
    const prev = parseFloat(previousInput.value)
    const current = parseFloat(displayValue.value)

    switch (operator.value) {
        case '+': result = prev + current; break;
        case '-': result = prev - current; break;
        case '*': result = prev * current; break;
        case '/':
            if (current === 0) {
                displayValue.value = 'Error'
                previousInput.value = null
                operator.value = null
                history.value = ''
                return
            }
            result = prev / current
            break;
    }

    // Round to reasonable decimals ideally, but standard float behavior is OK
    // JS float precision issue might occur (0.1+0.2), let's limit generic precision if needed
    // user didn't ask for rounding precision, but good practice:
    result = Math.round(result * 1000000000) / 1000000000

    displayValue.value = String(result)
    previousInput.value = null
    operator.value = null
    operatorClicked.value = true // allowing next number to start fresh
    history.value = ''
}

const clear = () => {
    displayValue.value = '0'
    previousInput.value = null
    operator.value = null
    operatorClicked.value = false
    history.value = ''
}

const percentage = () => {
    const val = parseFloat(displayValue.value)
    displayValue.value = String(val / 100)
}

const toggleSign = () => {
    if (displayValue.value === '0') return
    displayValue.value = displayValue.value.charAt(0) === '-'
        ? displayValue.value.slice(1)
        : `-${displayValue.value}`
}

// Warning Logic
const handleBeforeUnload = (e) => {
    if (isDirty()) {
        e.preventDefault()
        e.returnValue = '' // Chrome requires this to show prompt
        return ''
    }
}

onMounted(() => {
    window.addEventListener('beforeunload', handleBeforeUnload)
})

onBeforeUnmount(() => {
    window.removeEventListener('beforeunload', handleBeforeUnload)
})
</script>

<template>
    <div class="fixed bottom-6 right-6 z-50 flex flex-col items-end print:hidden">
        <!-- Calculator Body -->
        <transition name="calc-anim">
            <div v-show="isOpen" class="mb-4 rounded-2xl shadow-2xl overflow-hidden calculator-card w-80 border">
                <!-- Header -->
                <div class="flex items-center justify-between px-4 py-3 border-b calc-header">
                    <h3 class="font-bold text-sm">Calculator</h3>
                    <button @click="clear"
                        class="text-xs font-medium text-red-500 hover:text-red-600 hover:bg-red-50 px-2 py-1 rounded transition-colors">
                        Clear
                    </button>
                </div>

                <!-- Display -->
                <div class="px-4 py-4 calc-display bg-opacity-50">
                    <div class="text-right text-xs h-4 mb-1 font-mono truncate opacity-60">
                        {{ history || '&nbsp;' }}
                    </div>
                    <input type="text" readonly :value="displayValue"
                        class="w-full text-right text-3xl font-bold bg-transparent border-0 p-0 focus:ring-0 font-mono tracking-wider outline-none cursor-default" />
                </div>

                <!-- Keypad -->
                <div class="grid grid-cols-4 gap-px bg-gray-200 p-px">
                    <!-- Row 1 -->
                    <button @click="clear" class="calc-btn text-gray-600 bg-white hover:bg-gray-50">C</button>
                    <button @click="toggleSign" class="calc-btn text-gray-600 bg-white hover:bg-gray-50">+/-</button>
                    <button @click="percentage" class="calc-btn text-gray-600 bg-white hover:bg-gray-50">%</button>
                    <button @click="setOperation('/')"
                        class="calc-btn text-blue-600 font-bold bg-blue-50 hover:bg-blue-100">÷</button>

                    <!-- Row 2 -->
                    <button @click="append('7')" class="calc-btn font-medium bg-white hover:bg-gray-50">7</button>
                    <button @click="append('8')" class="calc-btn font-medium bg-white hover:bg-gray-50">8</button>
                    <button @click="append('9')" class="calc-btn font-medium bg-white hover:bg-gray-50">9</button>
                    <button @click="setOperation('*')"
                        class="calc-btn text-blue-600 font-bold bg-blue-50 hover:bg-blue-100">×</button>

                    <!-- Row 3 -->
                    <button @click="append('4')" class="calc-btn font-medium bg-white hover:bg-gray-50">4</button>
                    <button @click="append('5')" class="calc-btn font-medium bg-white hover:bg-gray-50">5</button>
                    <button @click="append('6')" class="calc-btn font-medium bg-white hover:bg-gray-50">6</button>
                    <button @click="setOperation('-')"
                        class="calc-btn text-blue-600 font-bold bg-blue-50 hover:bg-blue-100">−</button>

                    <!-- Row 4 -->
                    <button @click="append('1')" class="calc-btn font-medium bg-white hover:bg-gray-50">1</button>
                    <button @click="append('2')" class="calc-btn font-medium bg-white hover:bg-gray-50">2</button>
                    <button @click="append('3')" class="calc-btn font-medium bg-white hover:bg-gray-50">3</button>
                    <button @click="setOperation('+')"
                        class="calc-btn text-blue-600 font-bold bg-blue-50 hover:bg-blue-100">+</button>

                    <!-- Row 5 -->
                    <button @click="append('0')"
                        class="calc-btn font-medium col-span-2 bg-white hover:bg-gray-50 pl-6 text-left">0</button>
                    <button @click="append('.')" class="calc-btn font-medium bg-white hover:bg-gray-50">.</button>
                    <button @click="calculate"
                        class="calc-btn bg-blue-600 text-white hover:bg-blue-700 shadow-inner">=</button>
                </div>
            </div>
        </transition>

        <!-- Toggle Button -->
        <button @click="isOpen = !isOpen"
            class="h-14 w-14 rounded-full shadow-xl flex items-center justify-center transition-all duration-300 hover:scale-110 active:scale-95 z-50 float-btn"
            :class="isOpen ? 'bg-red-500 text-white rotate-45' : 'bg-blue-600 text-white'" title="Calculator">
            <svg v-if="isOpen" class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4"></path>
            </svg>
            <svg v-else xmlns="http://www.w3.org/2000/svg" class="w-6 h-6" viewBox="0 0 24 24" fill="none"
                stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <rect x="4" y="2" width="16" height="20" rx="2" ry="2"></rect>
                <line x1="8" y1="6" x2="16" y2="6"></line>
                <line x1="16" y1="14" x2="16" y2="14"></line>
                <line x1="12" y1="14" x2="12" y2="14"></line>
                <line x1="8" y1="14" x2="8" y2="14"></line>
                <line x1="16" y1="18" x2="16" y2="18"></line>
                <line x1="12" y1="18" x2="12" y2="18"></line>
                <line x1="8" y1="18" x2="8" y2="18"></line>
            </svg>
        </button>
    </div>
</template>

<style scoped>
.calculator-card {
    background-color: var(--t-main-card-bg, #ffffff);
    border-color: var(--t-main-card-border, #e5e7eb);
    color: var(--t-main-text, #111827);
}

.calc-header {
    background-color: var(--t-main-bg, #f9fafb);
    border-bottom-color: var(--t-main-card-border, #e5e7eb);
}

.calc-display {
    background-color: var(--t-main-input-bg, #f9fafb);
    color: var(--t-main-text, #111827);
}

.calc-btn {
    height: 3.5rem;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.1s;
    font-size: 1.125rem;
    /* Respect theme colors */
    background-color: var(--t-main-card-bg, #ffffff);
    color: var(--t-main-text-secondary, #374151);
}

.calc-btn:hover {
    background-color: var(--t-sidebar-hover, #f3f4f6);
}

/* Operation buttons special styling if needed, currently reusing classes but might want separate tokens */
.float-btn {
    background-color: var(--t-accent, #3b82f6);
}

.float-btn.rotate-45 {
    background-color: #ef4444;
    /* Close state usually red */
}

/* Animations */
.calc-anim-enter-active,
.calc-anim-leave-active {
    transition: all 0.3s cubic-bezier(0.16, 1, 0.3, 1);
}

.calc-anim-enter-from,
.calc-anim-leave-to {
    transform: translateY(20px) scale(0.95);
    opacity: 0;
}
</style>
