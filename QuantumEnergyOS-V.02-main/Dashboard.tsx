// ═══════════════════════════════════════════════════════════════════════
//  Dashboard — Pantalla principal
//  Resumen en tiempo real: batería, estado solar, red, Cuarzo 4D
// ═══════════════════════════════════════════════════════════════════════

import React from 'react';
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer } from 'recharts';
import consumptionData from './data/mexicali-consumption.json';
import { useTranslation } from 'react-i18next';

const Dashboard = () => {
  const { t } = useTranslation(["common", "dashboard"]);
  const data = consumptionData.hourly;
  const peak = consumptionData.peakDemand;

  return (
    <div className="p-6 bg-gray-950 text-white">
      <h1 className="text-4xl font-bold mb-2">{t("dashboard.title", "Mexicali Energy Grid")}</h1>
      <p className="text-emerald-400 text-xl mb-8">
        {t("dashboard.currentTemp", "14 de abril 2026 - Temperatura actual: {{temp}}°C", { temp: consumptionData.temperature })}
      </p>
      
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        <div className="bg-gray-900 p-6 rounded-xl">
          <p className="text-gray-400">{t("dashboard.peakDemand", "Demanda Pico")}</p>
          <p className="text-6xl font-bold text-orange-400 mt-2">{peak} MW</p>
          <p className="text-sm text-gray-500 mt-1">{t("dashboard.atHour", "a las {{hour}}:00 hrs", { hour: consumptionData.peakHour })}</p>
        </div>
        
        <div className="bg-gray-900 p-6 rounded-xl">
          <p className="text-gray-400">{t("dashboard.maxTemperature", "Temperatura Máxima")}</p>
          <p className="text-6xl font-bold text-red-400 mt-2">{consumptionData.temperature}°C</p>
        </div>
      </div>

      <div className="mt-8 bg-gray-900 p-6 rounded-xl">
        <h2 className="text-xl mb-4">{t("dashboard.consumptionByHour", "Consumo por hora")}</h2>
        <ResponsiveContainer width="100%" height={380}>
          <LineChart data={data}>
            <CartesianGrid strokeDasharray="3 3" stroke="#374151" />
            <XAxis dataKey="hour" stroke="#9CA3AF" />
            <YAxis stroke="#9CA3AF" />
            <Tooltip />
            <Line type="natural" dataKey="demand" stroke="#22C55E" strokeWidth={4} dot={false} />
          </LineChart>
        </ResponsiveContainer>
      </div>
    </div>
  );
};

export default Dashboard;