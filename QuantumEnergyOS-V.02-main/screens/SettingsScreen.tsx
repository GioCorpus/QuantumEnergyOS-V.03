import React from 'react';
import { useTranslation } from 'react-i18next';
import { Settings, Globe, Bell, Moon, Sun } from 'lucide-react';
import { useLanguage } from '../hooks/useLanguage';

const SettingsScreen = () => {
  const { t } = useTranslation(['common', 'settings']);
  const { currentLanguage, changeLanguage, languages } = useLanguage();

  return (
    <div className="p-6 bg-gray-950 text-white min-h-screen">
      <h1 className="text-3xl font-bold mb-6 flex items-center gap-2">
        <Settings />
        {t('navigation.settings', 'Config')}
      </h1>

      <div className="space-y-6">
        {/* Language Section */}
        <section className="card">
          <h2 className="text-xl font-semibold mb-4 flex items-center gap-2">
            <Globe className="text-blue-400" />
            {t('settings.language', 'Idioma / Language')}
          </h2>
          <div className="space-y-2">
            {languages.map((lang) => (
              <button
                key={lang.code}
                className={`w-full p-3 rounded-lg flex items-center justify-between ${
                  currentLanguage === lang.code ? 'bg-blue-500/20 border border-blue-400' : 'bg-gray-800'
                }`}
                onClick={() => changeLanguage(lang.code)}
              >
                <span className="flex items-center gap-2">
                  <span>{lang.flag}</span>
                  <span>{lang.name}</span>
                </span>
                {currentLanguage === lang.code && (
                  <span className="text-blue-400">✓</span>
                )}
              </button>
            ))}
          </div>
        </section>

        {/* Appearance Section */}
        <section className="card">
          <h2 className="text-xl font-semibold mb-4">
            {t('settings.appearance', 'Apariencia')}
          </h2>
          <div className="space-y-3">
            <button className="w-full p-3 rounded-lg bg-gray-800 flex items-center justify-between">
              <span className="flex items-center gap-2">
                <Moon size={18} />
                {t('settings.darkMode', 'Modo Oscuro')}
              </span>
              <span className="text-green-400">ON</span>
            </button>
          </div>
        </section>

        {/* Notifications Section */}
        <section className="card">
          <h2 className="text-xl font-semibold mb-4 flex items-center gap-2">
            <Bell className="text-yellow-400" />
            {t('settings.notifications', 'Notificaciones')}
          </h2>
          <p className="text-gray-400">
            {t('settings.notificationDesc', 'Configuración de alertas solares y del sistema')}
          </p>
        </section>

        {/* About Section */}
        <section className="card">
          <h2 className="text-xl font-semibold mb-4">
            {t('settings.about', 'Acerca de')}
          </h2>
          <div className="space-y-2 text-sm">
            <p><strong>QuantumEnergyOS</strong> V.02</p>
            <p>{t('settings.mission', 'Nunca más apagones en Mexicali')}</p>
            <p className="text-gray-500">Mexicali, Baja California, México</p>
          </div>
        </section>
      </div>
    </div>
  );
};

export default SettingsScreen;