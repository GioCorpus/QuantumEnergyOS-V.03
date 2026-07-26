import { useTranslation } from 'react-i18next';

export const useLanguage = () => {
  const { i18n } = useTranslation();

  const changeLanguage = (lng: string) => {
    i18n.changeLanguage(lng);
    localStorage.setItem('qeos-language', lng);
  };

  const currentLanguage = i18n.language as 'en' | 'es';

  return {
    currentLanguage,
    changeLanguage,
    languages: [
      { code: 'es', name: 'Español', flag: '🇲🇽' },
      { code: 'en', name: 'English', flag: '🇺🇸' },
    ],
  };
};