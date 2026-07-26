import { create } from 'zustand';

interface SettingsState {
  language: 'en' | 'es';
  darkMode: boolean;
  notifications: boolean;
  setLanguage: (lang: 'en' | 'es') => void;
  toggleDarkMode: () => void;
  toggleNotifications: () => void;
}

export const useSettingsStore = create<SettingsState>((set) => ({
  language: (localStorage.getItem('qeos-language') as 'en' | 'es') || 'es',
  darkMode: true,
  notifications: true,

  setLanguage: (lang) => {
    localStorage.setItem('qeos-language', lang);
    set({ language: lang });
  },

  toggleDarkMode: () => set((state) => ({ darkMode: !state.darkMode })),
  toggleNotifications: () => set((state) => ({ notifications: !state.notifications })),
}));