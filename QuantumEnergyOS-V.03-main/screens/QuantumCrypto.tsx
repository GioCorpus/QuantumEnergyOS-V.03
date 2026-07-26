// ═══════════════════════════════════════════════════════════════════════
//  QuantumCryptography — Panel de monitoreo de claves cuánticas
//  Visualiza sesiones QKD, tasas de error, detección de Eve
// ═══════════════════════════════════════════════════════════════════════

import React, { useState, useEffect } from 'react';
import { motion } from 'framer-motion';
import { Shield, Lock, AlertTriangle, Activity, Key, Eye, EyeOff } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, BarChart, Bar } from 'recharts';

interface QKDSession {
  session_id: string;
  protocol: string;
  error_rate: number;
  qber: number;
  eavesdropping_detected: boolean;
  final_key_length: number;
  timestamp: number;
}

interface CryptoStats {
  active_sessions: number;
  total_keys_generated: number;
  security_level: string;
  last_attack_detected: string | null;
}

const QuantumCryptoScreen: React.FC = () => {
  const { t } = useTranslation(['common', 'crypto']);
  const [sessions, setSessions] = useState<QKDSession[]>([]);
  const [stats, setStats] = useState<CryptoStats>({
    active_sessions: 0,
    total_keys_generated: 128,
    security_level: 'HYBRID_QKD_PQC',
    last_attack_detected: null
  });
  const [isRunningBB84, setIsRunningBB84] = useState(false);
  const [errorRateHistory, setErrorRateHistory] = useState<Array<{time: string, rate: number}>>([]);

  // Simular datos de QKD en tiempo real
  useEffect(() => {
    const interval = setInterval(() => {
      const now = new Date();
      const timeStr = `${now.getHours()}:${String(now.getMinutes()).padStart(2, '0')}:${String(now.getSeconds()).padStart(2, '0')}`;
      
      // Simular tasa de error QBER
      const rate = Math.random() * 5;
      setErrorRateHistory(prev => [...prev.slice(-20), { time: timeStr, rate: rate / 100 }]);
      
      // Actualizar sesiones
      if (Math.random() > 0.7) {
        const newSession: QKDSession = {
          session_id: `qkd_${Math.random().toString(36).substr(2, 9)}`,
          protocol: Math.random() > 0.5 ? 'BB84' : 'E91',
          error_rate: Math.random() * 0.2,
          qber: Math.random() * 0.15,
          eavesdropping_detected: Math.random() > 0.9,
          final_key_length: Math.floor(Math.random() * 256) + 128,
          timestamp: Date.now()
        };
        setSessions(prev => [newSession, ...prev.slice(0, 4)]);
        setStats(s => ({ ...s, active_sessions: s.active_sessions + 1, total_keys_generated: s.total_keys_generated + 1 }));
      }
    }, 2000);
    
    return () => clearInterval(interval);
  }, []);

  const runBB84Test = async () => {
    setIsRunningBB84(true);
    // En integración real: llamar al endpoint /crypto/qkd/bb84
    setTimeout(() => {
      const newSession: QKDSession = {
        session_id: `qkd_test_${Date.now()}`,
        protocol: 'BB84',
        error_rate: Math.random() * 0.1,
        qber: Math.random() * 0.08,
        eavesdropping_detected: Math.random() > 0.8,
        final_key_length: Math.floor(Math.random() * 256) + 128,
        timestamp: Date.now()
      };
      setSessions(prev => [newSession, ...prev.slice(0, 4)]);
      setIsRunningBB84(false);
    }, 1500);
  };

  const quantumBits = Array.from({ length: 128 }, (_, i) => ({
    bit: Math.random() > 0.5 ? 1 : 0,
    index: i
  }));

  return (
    <div className="p-6 bg-gray-950 text-white min-h-screen">
      {/* Header */}
      <div className="flex items-center justify-between mb-6">
        <div>
          <h1 className="text-3xl font-bold flex items-center gap-3">
            <Shield className="text-cyan-400" />
            {t('crypto.title', 'Quantum Cryptography')}
          </h1>
          <p className="text-gray-400 mt-1">
            {t('crypto.subtitle', 'Protección por física cuántica - Nunca más apagones en Mexicali')}
          </p>
        </div>
        <div className="flex items-center gap-2">
          <div className="w-3 h-3 bg-green-400 rounded-full animate-pulse"></div>
          <span className="text-sm text-gray-400">{t('crypto.status', 'Operacional')}</span>
        </div>
      </div>

      {/* Stats Cards */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4 mb-6">
        <motion.div 
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          className="bg-gray-900 p-4 rounded-xl"
        >
          <div className="flex items-center gap-3">
            <Activity className="text-cyan-400" />
            <div>
              <p className="text-gray-400 text-sm">{t('crypto.activeSessions', 'Sesiones Activas')}</p>
              <p className="text-2xl font-bold">{stats.active_sessions}</p>
            </div>
          </div>
        </motion.div>

        <motion.div 
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.1 }}
          className="bg-gray-900 p-4 rounded-xl"
        >
          <div className="flex items-center gap-3">
            <Key className="text-purple-400" />
            <div>
              <p className="text-gray-400 text-sm">{t('crypto.keysGenerated', 'Claves Generadas')}</p>
              <p className="text-2xl font-bold">{stats.total_keys_generated}</p>
            </div>
          </div>
        </motion.div>

        <motion.div 
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.2 }}
          className="bg-gray-900 p-4 rounded-xl"
        >
          <div className="flex items-center gap-3">
            <Lock className="text-emerald-400" />
            <div>
              <p className="text-gray-400 text-sm">{t('crypto.securityLevel', 'Nivel de Seguridad')}</p>
              <p className="text-lg font-bold text-emerald-400">{stats.security_level}</p>
            </div>
          </div>
        </motion.div>

        <motion.div 
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.3 }}
          className="bg-gray-900 p-4 rounded-xl"
        >
          <div className="flex items-center gap-3">
            <AlertTriangle className={stats.last_attack_detected ? 'text-red-400' : 'text-gray-500'} />
            <div>
              <p className="text-gray-400 text-sm">{t('crypto.lastAttack', 'Último Ataque')}</p>
              <p className="text-sm">{stats.last_attack_detected || t('crypto.noAttacks', 'Ningún ataque detectado')}</p>
            </div>
          </div>
        </motion.div>
      </div>

      {/* BB84 Test Button */}
      <div className="mb-6">
        <button
          onClick={runBB84Test}
          disabled={isRunningBB84}
          className={`px-6 py-3 rounded-xl font-bold flex items-center gap-2 transition-all ${
            isRunningBB84 
              ? 'bg-gray-700 cursor-not-allowed' 
              : 'bg-gradient-to-r from-cyan-600 to-blue-600 hover:from-cyan-500 hover:to-blue-500'
          }`}
        >
          <Shield size={20} />
          {isRunningBB84 ? t('crypto.running', 'Ejecutando BB84...') : t('crypto.runTest', 'Ejecutar Test BB84')}
        </button>
      </div>

      {/* Main Content Grid */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* QBER History Chart */}
        <div className="lg:col-span-2 bg-gray-900 p-4 rounded-xl">
          <h2 className="text-xl font-bold mb-4 flex items-center gap-2">
            <Activity className="text-cyan-400" />
            {t('crypto.qberHistory', 'Historial QBER (Quantum Bit Error Rate)')}
          </h2>
          <ResponsiveContainer width="100%" height={250}>
            <LineChart data={errorRateHistory}>
              <CartesianGrid strokeDasharray="3 3" stroke="#374151" />
              <XAxis dataKey="time" stroke="#9CA3AF" />
              <YAxis stroke="#9CA3AF" domain={[0, 0.2]} />
              <Tooltip 
                contentStyle={{ backgroundColor: '#1F2937', border: 'none' }}
                formatter={(value: number) => [`${(value as number * 100).toFixed(1)}%`, 'QBER']}
              />
              <Line 
                type="monotone" 
                dataKey="rate" 
                stroke="#22C55E" 
                strokeWidth={2} 
                dot={false}
              />
              {/* Umbral de detección */}
              <Line 
                type="monotone" 
                dataKey={() => 0.11} 
                stroke="#EF4444" 
                strokeWidth={1} 
                strokeDashArray="5 5"
              />
            </LineChart>
          </ResponsiveContainer>
          <p className="text-xs text-gray-500 mt-2">Línea roja: Umbral de detección de Eve (11%)</p>
        </div>

        {/* Quantum Key Visualization */}
        <div className="bg-gray-900 p-4 rounded-xl">
          <h2 className="text-xl font-bold mb-4 flex items-center gap-2">
            <Key className="text-purple-400" />
            {t('crypto.keyVisualization', 'Visualización de Clave')}
          </h2>
          <div className="grid grid-cols-16 gap-1">
            {quantumBits.map((qb) => (
              <motion.div
                key={qb.index}
                initial={{ scale: 0 }}
                animate={{ scale: 1 }}
                transition={{ delay: qb.index * 0.01 }}
                className={`w-4 h-4 rounded ${
                  qb.bit === 1 ? 'bg-purple-400' : 'bg-gray-700'
                }`}
                title={`Bit ${qb.index}: ${qb.bit}`}
              />
            ))}
          </div>
          <p className="text-xs text-gray-500 mt-3">
            {t('crypto.keyFormat', 'Formato: 1 = |1⟩ (vertical), 0 = |0⟩ (horizontal)')}
          </p>
        </div>
      </div>

      {/* Recent Sessions Table */}
      <div className="mt-6 bg-gray-900 p-4 rounded-xl">
        <h2 className="text-xl font-bold mb-4 flex items-center gap-2">
          <Eye className="text-cyan-400" />
          {t('crypto.recentSessions', 'Sesiones Recientes')}
        </h2>
        <div className="overflow-x-auto">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-gray-700">
                <th className="text-left py-2">Session ID</th>
                <th className="text-left py-2">Protocolo</th>
                <th className="text-left py-2">Error Rate</th>
                <th className="text-left py-2">QBER</th>
                <th className="text-left py-2">Eve Detectado</th>
                <th className="text-left py-2">Longitud Clave</th>
              </tr>
            </thead>
            <tbody>
              {sessions.map((s, i) => (
                <motion.tr
                  key={s.session_id}
                  initial={{ opacity: 0 }}
                  animate={{ opacity: 1 }}
                  transition={{ delay: i * 0.1 }}
                  className="border-b border-gray-800"
                >
                  <td className="py-2 font-mono text-xs">{s.session_id.substring(0, 16)}...</td>
                  <td className="py-2">
                    <span className={`px-2 py-1 rounded ${s.protocol === 'BB84' ? 'bg-cyan-900/50' : 'bg-purple-900/50'}`}>
                      {s.protocol}
                    </span>
                  </td>
                  <td className="py-2">{(s.error_rate * 100).toFixed(1)}%</td>
                  <td className="py-2">{(s.qber * 100).toFixed(1)}%</td>
                  <td className="py-2">
                    {s.eavesdropping_detected ? (
                      <span className="text-red-400 flex items-center gap-1">
                        <Eye size={14} /> Sí
                      </span>
                    ) : (
                      <span className="text-emerald-400 flex items-center gap-1">
                        <EyeOff size={14} /> No
                      </span>
                    )}
                  </td>
                  <td className="py-2">{s.final_key_length} bits</td>
                </motion.tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
};

export default QuantumCryptoScreen;