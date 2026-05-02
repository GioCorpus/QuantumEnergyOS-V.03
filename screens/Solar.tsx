import React from 'react';
import { useTranslation } from 'react-i18next';
import { Sun } from 'lucide-react';

const SolarScreen = () => {
  const { t } = useTranslation(['common', 'dashboard']);

  return (
    <div className="p-6 bg-gray-950 text-white">
      <h1 className="text-3xl font-bold mb-4 flex items-center gap-2">
        <Sun className="text-yellow-400" />
        {t('navigation.solar', 'Solar')}
      </h1>
      <p className="text-gray-400">
        {t('solar.subtitle', 'Predicción de actividad solar y su impacto en la red')}
      </p>
      {/* TODO: Implement solar prediction UI */}
    </div>
  );
};

export default SolarScreen;