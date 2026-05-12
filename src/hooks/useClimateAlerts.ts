/**
 * useClimateAlerts Hook
 * Conexión WebSocket y polling para alertas del Climate Orchestrator
 */

import { useEffect, useRef, useCallback } from 'react';
import { useNotificationStore } from '../store/notificationStore';
import type { ClimateStatusPayload } from '../types/notifications';

const WS_URL = '/api/v1/climate/status';
const ANALYSIS_URL = '/api/v1/climate/analyze';
const POLL_INTERVAL = 30000; // 30 segundos

export const useClimateAlerts = () => {
  const wsRef = useRef<WebSocket | null>(null);
  const pollIntervalRef = useRef<NodeJS.Timeout | null>(null);
  const { addNotification, doNotDisturb, soundEnabled } = useNotificationStore();

  // Reproducir sonido de alerta
  const playAlertSound = useCallback(() => {
    if (!soundEnabled || doNotDisturb) return;
    
    try {
      const audioContext = new (window.AudioContext || (window as any).webkitAudioContext)();
      const oscillator = audioContext.createOscillator();
      const gainNode = audioContext.createGain();
      
      oscillator.connect(gainNode);
      gainNode.connect(audioContext.destination);
      
      oscillator.frequency.value = 880; // Frecuencia aguda para alerta
      gainNode.gain.value = 0.3;
      
      oscillator.start();
      oscillator.stop(audioContext.currentTime + 0.5);
    } catch (e) {
      console.warn('Could not play alert sound:', e);
    }
  }, [soundEnabled, doNotDisturb]);

  // Mapear risk_level a severidad
  const getSeverityFromRisk = (risk: string): 'critical' | 'high' | 'medium' | 'info' => {
    switch (risk?.toLowerCase()) {
      case 'extreme': return 'critical';
      case 'high': return 'high';
      case 'medium': return 'medium';
      default: return 'info';
    }
  };

  // Mapear tipo de alerta
  const getNotificationType = (status: ClimateStatusPayload): 'climate-risk' | 'grid-collapse' | 'system' => {
    if (status.solar_activity === 'extreme') return 'climate-risk';
    if (status.status === 'critical') return 'grid-collapse';
    return 'system';
  };

  // Obtener datos de análisis climático
  const fetchAnalysis = useCallback(async () => {
    try {
      const response = await fetch(ANALYSIS_URL, {
        method: 'GET',
        headers: { 'Content-Type': 'application/json' },
      });
      
      if (!response.ok) throw new Error('Analysis fetch failed');
      
      const data: ClimateStatusPayload = await response.json();
      
      // Crear notificación basada en el análisis
      if (data.risk_level !== 'low') {
        const severity = getSeverityFromRisk(data.risk_level);
        const type = getNotificationType(data);
        
        addNotification({
          type,
          severity,
          title: getClimateTitle(data),
          description: getClimateDescription(data),
          location: data.affected_areas?.join(', ') || 'Mexicali',
          affectedNodes: data.affected_areas || [],
          source: 'Climate Orchestrator',
          actions: getClimateActions(data),
        });

        if (severity === 'critical' && !doNotDisturb) {
          playAlertSound();
        }
      }
    } catch (error) {
      console.error('Error fetching climate analysis:', error);
    }
  }, [addNotification, doNotDisturb, playAlertSound]);

  // Manejar mensaje WebSocket
  const handleWebSocketMessage = useCallback((data: ClimateStatusPayload) => {
    const severity = getSeverityFromRisk(data.risk_level);
    const type = getNotificationType(data);
    
    addNotification({
      type,
      severity,
      title: getClimateTitle(data),
      description: getClimateDescription(data),
      location: data.affected_areas?.join(', ') || 'Mexicali',
      affectedNodes: data.affected_areas || [],
      source: 'Climate Orchestrator',
      actions: getClimateActions(data),
    });

    if (severity === 'critical' && !doNotDisturb) {
      playAlertSound();
    }
  }, [addNotification, doNotDisturb, playAlertSound]);

  // Conectar WebSocket
  const connectWebSocket = useCallback(() => {
    if (wsRef.current?.readyState === WebSocket.OPEN) return;

    try {
      const ws = new WebSocket(
        window.location.protocol === 'https:' 
          ? WS_URL.replace('http', 'ws') 
          : WS_URL.replace('http', 'ws')
      );

      ws.onopen = () => {
        console.log('Climate alerts WebSocket connected');
      };

      ws.onmessage = (event) => {
        try {
          const data: ClimateStatusPayload = JSON.parse(event.data);
          handleWebSocketMessage(data);
        } catch (e) {
          console.error('Invalid WebSocket message:', e);
        }
      };

      ws.onerror = (error) => {
        console.error('WebSocket error:', error);
      };

      ws.onclose = () => {
        console.log('WebSocket closed, attempting reconnect...');
        setTimeout(connectWebSocket, 5000);
      };

      wsRef.current = ws;
    } catch (e) {
      console.error('WebSocket connection failed:', e);
    }
  }, [handleWebSocketMessage]);

  // Iniciar polling como fallback
  const startPolling = useCallback(() => {
    pollIntervalRef.current = setInterval(fetchAnalysis, POLL_INTERVAL);
  }, [fetchAnalysis]);

  useEffect(() => {
    // Intentar WebSocket primero
    connectWebSocket();
    
    // Polling como backup
    startPolling();

    return () => {
      if (wsRef.current) {
        wsRef.current.close();
      }
      if (pollIntervalRef.current) {
        clearInterval(pollIntervalRef.current);
      }
    };
  }, [connectWebSocket, startPolling]);

  return { fetchAnalysis };
};

// Helper functions
function getClimateTitle(data: ClimateStatusPayload): string {
  if (data.solar_activity === 'extreme') {
    return '🌞 Tormenta Solar Extrema Detectada';
  }
  if (data.status === 'critical') {
    return '⚡ Riesgo Crítico en la Red';
  }
  if (data.risk_level === 'high') {
    return '⚠️ Riesgo Climático Alto';
  }
  return '📊 Actualización Climática';
}

function getClimateDescription(data: ClimateStatusPayload): string {
  const parts = [];
  
  if (data.temperature) {
    parts.push(`Temperatura: ${data.temperature}°C`);
  }
  
  if (data.solar_activity) {
    parts.push(`Actividad solar: ${data.solar_activity}`);
  }
  
  if (data.recommendations?.length) {
    parts.push('Recomendaciones disponibles');
  }
  
  return parts.join(' | ') || 'Condiciones climáticas normales';
}

function getClimateActions(data: ClimateStatusPayload) {
  const actions = [
    { id: 'view-map', label: 'Ver en mapa', icon: 'Map' },
    { id: 'recommend', label: 'Ver recomendación', icon: 'Lightbulb' },
  ];
  
  if (data.risk_level === 'extreme' || data.risk_level === 'high') {
    actions.push({ id: 'silence', label: 'Silenciar 15 min', icon: 'VolumeX' });
  }
  
  actions.push({ id: 'dismiss', label: 'Marcar atendido', icon: 'Check' });
  
  return actions;
}