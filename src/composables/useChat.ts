import { ref, reactive } from "vue";
import { invoke } from "@tauri-apps/api/core";

export interface Message {
  id: number;
  conversation_id: number;
  sender: "user" | "ai";
  content: string;
  created_at: string;
}

export interface Conversation {
  id: number;
  title: string;
  created_at: string;
}

// Global state using closure pattern (like a simple store)
const conversations = ref<Conversation[]>([]);
const currentConversationId = ref<number | null>(null);
const messages = ref<Message[]>([]);
// key can be stored here to avoid fetching every time
const apiKey = ref<string>("");

export function useChat() {
  const loading = ref(false);
  const sending = ref(false);

  async function loadConversations() {
    try {
      conversations.value = await invoke("get_conversations");
    } catch (e) {
      console.error(e);
    }
  }

  async function loadMessages(id: number) {
    try {
      loading.value = true;
      currentConversationId.value = id;
      messages.value = await invoke("get_messages", { conversationId: id });
    } catch (e) {
      console.error(e);
    } finally {
      loading.value = false;
    }
  }

  async function createConversation(title: string) {
    try {
      const id = await invoke("create_conversation", { title });
      console.log("Created conversation", id);
      await loadConversations();
      await loadMessages(id as number);
      return id;
    } catch (e) {
      console.error(e);
      throw e;
    }
  }

  async function deleteConversation(id: number) {
    try {
      await invoke("delete_conversation", { conversationId: id });
      if (currentConversationId.value === id) {
        currentConversationId.value = null;
        messages.value = [];
      }
      await loadConversations();
    } catch (e) {
      console.error(e);
    }
  }

  async function sendMessage(text: string) {
    if (!currentConversationId.value) return;
    try {
      sending.value = true;

      // We need API key. Fetch if not cached.
      if (!apiKey.value) {
        const settings: Record<string, string> = await invoke("get_settings");
        apiKey.value = settings["google_ai_key"] || "";
      }

      if (!apiKey.value) {
        throw new Error(
          "Google AI Studio API Key is missing. Please set it in Settings.",
        );
      }

      // Optimistic update for UI responsiveness
      const tempId = Date.now();
      const optimisticMsg: Message = {
        id: tempId,
        conversation_id: currentConversationId.value,
        sender: "user",
        content: text,
        created_at: new Date().toISOString(),
      };
      messages.value.push(optimisticMsg);

      // Call Backend
      // The backend saves BOTH user message and AI response
      const aiResponse: Message = await invoke("send_chat_message", {
        conversationId: currentConversationId.value,
        userMessage: text,
        apiKey: apiKey.value,
      });

      // Refresh messages to get correct IDs and timestamps for both
      await loadMessages(currentConversationId.value);
    } catch (e) {
      console.error(e);
      // Remove the optimistic message if it was the last one
      // messages.value.pop();
      throw e; // Let the component handle UI feedback (toast/alert)
    } finally {
      sending.value = false;
    }
  }

  return {
    conversations,
    currentConversationId,
    messages,
    loading,
    sending,
    loadConversations,
    loadMessages,
    createConversation,
    deleteConversation,
    sendMessage,
  };
}
