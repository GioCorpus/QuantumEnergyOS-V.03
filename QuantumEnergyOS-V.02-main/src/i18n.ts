import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';

const resources = {
  en: {
    common: () => import('../locales/en/common.json'),
    dashboard: () => import('../locales/en/dashboard.json'),
    quantum: () => import('../locales/en/quantum.json'),
  },
  es: {
    common: () => import('../locales/es/common.json'),
    dashboard: () => import('../locales/es/dashboard.json'),
    quantum: () => import('../locales/es/quantum.json'),
  },
};

i18n
  .use(initReactI18next)
  .init({
    resources,
    lng: (() => {
      const saved = localStorage.getItem('qeos-language');
      if (saved && ['en', 'es'].includes(saved)) return saved;
      
      const browserLang = navigator.language.split('-')[0];
      return browserLang === 'es' ? 'es' : 'en';
    })(),
    fallbackLng: 'en',
    interpolation: {
      escapeValue: false,
    },
    ns: ['common', 'dashboard', 'quantum'],
    defaultNS: 'common',
  });

export default i18n;