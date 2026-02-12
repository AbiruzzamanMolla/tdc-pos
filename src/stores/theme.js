import { defineStore } from 'pinia';
import { ref, watch } from 'vue';

const themes = {
  midnight: {
    id: 'midnight',
    name: 'Midnight',
    emoji: '🌙',
    // Sidebar
    sidebarBg: '#111827',
    sidebarBorder: '#1f2937',
    sidebarHover: '#1f2937',
    sidebarUserBg: 'rgba(31,41,55,0.2)',
    sidebarSectionText: '#6b7280',
    sidebarText: '#ffffff',
    sidebarMuted: '#9ca3af',
    sidebarFooterBg: 'rgba(0,0,0,0.2)',
    sidebarFooterText: '#6b7280',
    // Accent (active nav, Logo badge, avatar)
    accent: '#3b82f6',
    accentHover: '#2563eb',
    accentShadow: 'rgba(59,130,246,0.2)',
    accentLight: '#dbeafe',
    accentLightText: '#1e40af',
    // Main content
    mainBg: '#f9fafb',
    mainCardBg: '#ffffff',
    mainCardBorder: '#f3f4f6',
    mainText: '#111827',
    mainMuted: '#9ca3af',
    // Logout
    logoutBg: '#1f2937',
    logoutText: '#9ca3af',
    logoutHoverBg: 'rgba(220,38,38,0.2)',
    logoutHoverText: '#f87171',
  },
  ocean: {
    id: 'ocean',
    name: 'Ocean',
    emoji: '🌊',
    sidebarBg: '#0f172a',
    sidebarBorder: '#1e293b',
    sidebarHover: '#1e293b',
    sidebarUserBg: 'rgba(30,41,59,0.3)',
    sidebarSectionText: '#64748b',
    sidebarText: '#e2e8f0',
    sidebarMuted: '#94a3b8',
    sidebarFooterBg: 'rgba(0,0,0,0.25)',
    sidebarFooterText: '#64748b',
    accent: '#06b6d4',
    accentHover: '#0891b2',
    accentShadow: 'rgba(6,182,212,0.25)',
    accentLight: '#cffafe',
    accentLightText: '#155e75',
    mainBg: '#f0f9ff',
    mainCardBg: '#ffffff',
    mainCardBorder: '#e0f2fe',
    mainText: '#0c4a6e',
    mainMuted: '#7dd3fc',
    logoutBg: '#1e293b',
    logoutText: '#94a3b8',
    logoutHoverBg: 'rgba(220,38,38,0.15)',
    logoutHoverText: '#f87171',
  },
  emerald: {
    id: 'emerald',
    name: 'Emerald',
    emoji: '🌿',
    sidebarBg: '#022c22',
    sidebarBorder: '#064e3b',
    sidebarHover: '#064e3b',
    sidebarUserBg: 'rgba(6,78,59,0.3)',
    sidebarSectionText: '#6ee7b7',
    sidebarText: '#ecfdf5',
    sidebarMuted: '#a7f3d0',
    sidebarFooterBg: 'rgba(0,0,0,0.3)',
    sidebarFooterText: '#6ee7b7',
    accent: '#10b981',
    accentHover: '#059669',
    accentShadow: 'rgba(16,185,129,0.25)',
    accentLight: '#d1fae5',
    accentLightText: '#065f46',
    mainBg: '#f0fdf4',
    mainCardBg: '#ffffff',
    mainCardBorder: '#dcfce7',
    mainText: '#14532d',
    mainMuted: '#86efac',
    logoutBg: '#064e3b',
    logoutText: '#a7f3d0',
    logoutHoverBg: 'rgba(220,38,38,0.2)',
    logoutHoverText: '#fca5a5',
  },
  sunset: {
    id: 'sunset',
    name: 'Sunset',
    emoji: '🌅',
    sidebarBg: '#1c1917',
    sidebarBorder: '#292524',
    sidebarHover: '#292524',
    sidebarUserBg: 'rgba(41,37,36,0.3)',
    sidebarSectionText: '#a8a29e',
    sidebarText: '#fafaf9',
    sidebarMuted: '#d6d3d1',
    sidebarFooterBg: 'rgba(0,0,0,0.2)',
    sidebarFooterText: '#78716c',
    accent: '#f97316',
    accentHover: '#ea580c',
    accentShadow: 'rgba(249,115,22,0.25)',
    accentLight: '#ffedd5',
    accentLightText: '#9a3412',
    mainBg: '#fffbeb',
    mainCardBg: '#ffffff',
    mainCardBorder: '#fef3c7',
    mainText: '#78350f',
    mainMuted: '#fbbf24',
    logoutBg: '#292524',
    logoutText: '#a8a29e',
    logoutHoverBg: 'rgba(220,38,38,0.15)',
    logoutHoverText: '#f87171',
  },
  royal: {
    id: 'royal',
    name: 'Royal',
    emoji: '👑',
    sidebarBg: '#1e1b4b',
    sidebarBorder: '#312e81',
    sidebarHover: '#312e81',
    sidebarUserBg: 'rgba(49,46,129,0.3)',
    sidebarSectionText: '#a5b4fc',
    sidebarText: '#eef2ff',
    sidebarMuted: '#c7d2fe',
    sidebarFooterBg: 'rgba(0,0,0,0.25)',
    sidebarFooterText: '#818cf8',
    accent: '#8b5cf6',
    accentHover: '#7c3aed',
    accentShadow: 'rgba(139,92,246,0.25)',
    accentLight: '#ede9fe',
    accentLightText: '#5b21b6',
    mainBg: '#faf5ff',
    mainCardBg: '#ffffff',
    mainCardBorder: '#f3e8ff',
    mainText: '#3b0764',
    mainMuted: '#c084fc',
    logoutBg: '#312e81',
    logoutText: '#a5b4fc',
    logoutHoverBg: 'rgba(220,38,38,0.2)',
    logoutHoverText: '#fca5a5',
  },
  rose: {
    id: 'rose',
    name: 'Rose',
    emoji: '🌹',
    sidebarBg: '#1a1a2e',
    sidebarBorder: '#16213e',
    sidebarHover: '#16213e',
    sidebarUserBg: 'rgba(22,33,62,0.3)',
    sidebarSectionText: '#fda4af',
    sidebarText: '#fff1f2',
    sidebarMuted: '#fecdd3',
    sidebarFooterBg: 'rgba(0,0,0,0.25)',
    sidebarFooterText: '#fb7185',
    accent: '#f43f5e',
    accentHover: '#e11d48',
    accentShadow: 'rgba(244,63,94,0.25)',
    accentLight: '#ffe4e6',
    accentLightText: '#9f1239',
    mainBg: '#fff1f2',
    mainCardBg: '#ffffff',
    mainCardBorder: '#ffe4e6',
    mainText: '#881337',
    mainMuted: '#fda4af',
    logoutBg: '#16213e',
    logoutText: '#fecdd3',
    logoutHoverBg: 'rgba(220,38,38,0.2)',
    logoutHoverText: '#fca5a5',
  }
};

function applyThemeToDOM(theme) {
  const root = document.documentElement;
  Object.entries(theme).forEach(([key, value]) => {
    if (key === 'id' || key === 'name' || key === 'emoji') return;
    // Convert camelCase to kebab-case for CSS var
    const cssVar = '--t-' + key.replace(/([A-Z])/g, '-$1').toLowerCase();
    root.style.setProperty(cssVar, value);
  });
}

export const useThemeStore = defineStore('theme', () => {
  const currentThemeId = ref(localStorage.getItem('theme') || 'midnight');
  const showPicker = ref(false);

  const allThemes = Object.values(themes);

  const currentTheme = () => themes[currentThemeId.value] || themes.midnight;

  function setTheme(themeId) {
    if (!themes[themeId]) return;
    currentThemeId.value = themeId;
    localStorage.setItem('theme', themeId);
    applyThemeToDOM(themes[themeId]);
  }

  function initTheme() {
    applyThemeToDOM(currentTheme());
  }

  return {
    currentThemeId,
    showPicker,
    allThemes,
    currentTheme,
    setTheme,
    initTheme
  };
});
