import { invoke } from '@tauri-apps/api/core';
import { useAuthStore } from '../stores/auth';

/**
 * Log an activity from the frontend.
 * Call this after any successful mutation (create, update, delete, etc.)
 */
export async function logActivity(action, entityType, entityId, description) {
  try {
    const auth = useAuthStore();
    await invoke('log_activity', {
      userId: auth.user?.id || null,
      username: auth.user?.username || 'system',
      action,
      entityType,
      entityId: entityId ?? null,
      description
    });
  } catch (err) {
    console.error('Failed to log activity:', err);
  }
}
