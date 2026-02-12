<script setup>
import { ref, onMounted, nextTick, watch } from 'vue';
import { useChat } from '../composables/useChat';

const { conversations, currentConversationId, messages, loading, sending, loadConversations, loadMessages, createConversation, deleteConversation, sendMessage } = useChat();

const input = ref('');
const messagesContainer = ref(null);

onMounted(() => {
    loadConversations();
});

const createNew = async () => {
    const title = prompt("Enter chat title:");
    if (title) {
        try {
            await createConversation(title);
        } catch (e) {
            alert("Failed to create chat: " + e);
        }
    }
};

const selectChat = (id) => {
    loadMessages(id);
};

const deleteChat = (id) => {
    if (confirm("Delete this chat?")) {
        deleteConversation(id);
    }
};

const send = async () => {
    if (!input.value.trim() || sending.value) return;
    const text = input.value;
    input.value = ''; // Clear immediately for UX

    try {
        await sendMessage(text);
    } catch (e) {
        input.value = text; // Restore on fail
        const errorMsg = typeof e === 'string' ? e : (e.message || JSON.stringify(e));
        alert("Failed to send: " + errorMsg);
    }
};

// Auto scroll
watch(messages, () => {
    nextTick(() => {
        if (messagesContainer.value) {
            messagesContainer.value.scrollTop = messagesContainer.value.scrollHeight;
        }
    });
}, { deep: true });
</script>

<template>
    <div class="flex h-[calc(100vh-theme('spacing.16'))] bg-gray-50 border rounded-xl overflow-hidden shadow-sm"
        style="height: calc(100vh - 100px);">
        <!-- Sidebar -->
        <div class="w-64 bg-white border-r flex flex-col h-full">
            <div class="p-4 border-b">
                <button @click="createNew"
                    class="w-full bg-blue-600 text-white rounded-lg px-4 py-2 font-bold hover:bg-blue-700 transition flex items-center justify-center gap-2 shadow-sm">
                    <span>+</span> New Chat
                </button>
            </div>
            <div class="flex-1 overflow-y-auto p-2 space-y-1">
                <div v-for="chat in conversations" :key="chat.id" @click="selectChat(chat.id)"
                    class="p-3 rounded-lg cursor-pointer transition flex justify-between group items-center"
                    :class="currentConversationId === chat.id ? 'bg-blue-50 text-blue-700 font-bold border border-blue-100' : 'hover:bg-gray-50 text-gray-700'">
                    <span class="truncate text-sm flex-1">{{ chat.title }}</span>
                    <button @click.stop="deleteChat(chat.id)"
                        class="opacity-0 group-hover:opacity-100 text-gray-400 hover:text-red-500 p-1 rounded hover:bg-red-50 transition-all font-bold">×</button>
                </div>
                <div v-if="conversations.length === 0" class="text-center p-4 text-gray-400 text-xs italic">
                    No conversations yet
                </div>
            </div>
        </div>

        <!-- Main Chat Area -->
        <div class="flex-1 flex flex-col h-full bg-white">
            <div v-if="!currentConversationId"
                class="flex-1 flex items-center justify-center text-gray-400 flex-col opacity-50">
                <div class="text-6xl mb-4">🤖</div>
                <p class="text-lg font-medium">Select a conversation or create a new one</p>
            </div>
            <template v-else>
                <!-- Messages -->
                <div class="flex-1 overflow-y-auto p-6 space-y-6" ref="messagesContainer">
                    <div v-for="msg in messages" :key="msg.id" class="flex w-full"
                        :class="msg.sender === 'user' ? 'justify-end' : 'justify-start'">
                        <div class="max-w-[85%] rounded-2xl px-5 py-3 shadow-sm border"
                            :class="msg.sender === 'user' ? 'bg-blue-600 text-white rounded-tr-none border-transparent' : 'bg-gray-50 border-gray-100 rounded-tl-none text-gray-800'">
                            <div class="text-[10px] uppercase font-bold tracking-wider mb-1 opacity-70 flex justify-between gap-4"
                                :class="msg.sender === 'user' ? 'text-blue-100' : 'text-gray-500'">
                                <span>{{ msg.sender === 'user' ? 'You' : 'Assistant' }}</span>
                                <span>{{ new Date(msg.created_at).toLocaleTimeString([], {
                                    hour: '2-digit',
                                    minute: '2-digit'
                                }) }}</span>
                            </div>
                            <div class="whitespace-pre-wrap leading-relaxed text-sm markdown-body">{{ msg.content }}
                            </div>
                        </div>
                    </div>

                    <!-- Typing Indicator -->
                    <div v-if="sending" class="flex justify-start w-full">
                        <div
                            class="bg-gray-50 border border-gray-100 rounded-2xl rounded-tl-none px-5 py-4 shadow-sm flex items-center space-x-1.5">
                            <span class="w-2 h-2 bg-gray-400 rounded-full animate-bounce"></span>
                            <span class="w-2 h-2 bg-gray-400 rounded-full animate-bounce delay-150"></span>
                            <span class="w-2 h-2 bg-gray-400 rounded-full animate-bounce delay-300"></span>
                        </div>
                    </div>
                </div>

                <!-- Input -->
                <div class="p-4 bg-white border-t">
                    <form @submit.prevent="send" class="flex gap-2 relative">
                        <input v-model="input" type="text" placeholder="Type your message..."
                            class="flex-1 border border-gray-300 rounded-xl px-4 py-3 focus:outline-none focus:ring-2 focus:ring-blue-500 pr-24 shadow-sm"
                            :disabled="sending" ref="inputRef" autofocus />
                        <button type="submit" :disabled="!input.trim() || sending"
                            class="absolute right-2 top-2 bottom-2 bg-blue-600 text-white rounded-lg px-4 font-bold hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed transition text-sm">
                            Send
                        </button>
                    </form>
                </div>
            </template>
        </div>
    </div>
</template>

<style scoped>
/* Optional: Add some basic markdown styling if needed, or rely on tailwind typography */
.markdown-body {
    font-family: inherit;
}
</style>
