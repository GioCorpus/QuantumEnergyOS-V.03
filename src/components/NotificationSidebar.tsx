/**
 * Notification Sidebar Component
 * Barra lateral derecha con notificaciones del Climate Orchestrator
 */

import { useEffect, useRef } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { 
  Bell, X, Check, Trash2, Settings, Volume2, VolumeX,
  Zap, Sun, Shield, AlertTriangle, ChevronRight, ChevronLeft
} from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { 
  useNotificationStore, 
  useUnreadCount, 
  useCriticalCount,
  useFilteredNotifications 
} from '../store/notificationStore';
import { useClimateAlerts } from '../hooks/useClimateAlerts';
import type { Notification } from '../types/notifications';

// Solicitar permiso para notificaciones del sistema
const requestNotificationPermission = async () => {
  if (!('Notification' in window)) return false;
  const permission = await Notification.requestPermission();
  return permission === 'granted';
};

// Mostrar notificación del sistema
const showSystemNotification = (notification: Notification) => {
  if (!('Notification' in window) || Notification.permission !== 'granted') return;
  
  const options: NotificationOptions = {
    body: notification.description,
    icon: '/favicon.ico',
    tag: notification.id,
    requireInteraction: notification.severity === 'critical',
  };
  
  new Notification(notification.title, options);
};

export const NotificationSidebar = () => {
  const { t } = useTranslation();
  const { 
    isOpen, 
    isCollapsed, 
    doNotDisturb, 
    soundEnabled, 
    position,
    toggleSidebar, 
    toggleCollapse, 
    toggleDoNotDisturb,
    toggleSound,
    markAsRead,
    dismiss,
    markAllAsRead,
    dismissAll,
    clearHistory,
  } = useNotificationStore();
  
  const unreadCount = useUnreadCount();
  const criticalCount = useCriticalCount();
  const notifications = useFilteredNotifications();
  
  // Iniciar conexión con Climate Orchestrator
  useClimateAlerts();
  
  // Solicitar permiso para notificaciones al montar
  useEffect(() => {
    requestNotificationPermission();
  }, []);
  
  // Mostrar notificación del sistema cuando llegue una crítica
  const prevCriticalCount = usePrevious(criticalCount);
  useEffect(() => {
    if (criticalCount > prevCriticalCount && criticalCount > 0 && !doNotDisturb) {
      const latest = notifications.find(n => n.severity === 'critical' && !n.read);
      if (latest) showSystemNotification(latest);
    }
  }, [criticalCount, prevCriticalCount, doNotDisturb, notifications]);
  
  // Generar alerta de prueba
  const simulateAlert = () => {
    const severities = ['critical', 'high', 'medium'] as const;
    const types = ['climate-risk', 'grid-collapse', 'sensor-failure'] as const;
    
    const testAlert = {
      type: types[Math.floor(Math.random() * types.length)],
      severity: severities[Math.floor(Math.random() * severities.length)],
      title: 'Prueba de Alerta - ' + new Date().toLocaleTimeString(),
      description: 'Esta es una alerta simulada para pruebas de la barra de notificaciones.',
      location: 'Zona de Prueba - Mexicali',
      affectedNodes: ['NODO-TEST-01', 'NODO-TEST-02'],
      source: 'System' as const,
      actions: [
        { id: 'ack', label: 'Reconocer', icon: 'Check' },
        { id: 'dismiss', label: 'Descartar', icon: 'X' }
      ]
    };
    
    useNotificationStore.getState().addNotification(testAlert);
  };

  const severityColors = {
    critical: 'border-l-red-500 bg-red-500/10',
    high: 'border-l-orange-500 bg-orange-500/10',
    medium: 'border-l-yellow-500 bg-yellow-500/10',
    info: 'border-l-blue-500 bg-blue-500/10'
  };

  const severityIcons = {
    critical: Zap,
    high: AlertTriangle,
    medium: Shield,
    info: Bell
  };

  return (
    <>
      {/* Botón de toggle en el borde derecho */}
      {!isOpen && (
        <motion.button
          initial={{ x: 20 }}
          animate={{ x: 0 }}
          onClick={toggleSidebar}
          className="fixed right-4 top-1/2 -translate-y-1/2 z-50 bg-[#0a0f1a] border border-[#00d4ff]/30 rounded-l-lg p-3 hover:bg-[#00d4ff]/10 transition-colors"
          aria-label="Abrir barra de notificaciones"
        >
          <Bell className="w-5 h-5 text-[#00d4ff]" />
          {unreadCount > 0 && (
            <span className="absolute -top-1 -left-1 bg-red-500 text-white text-xs rounded-full w-5 h-5 flex items-center justify-center">
              {unreadCount > 9 ? '9+' : unreadCount}
            </span>
          )}
        </motion.button>
      )}

      {/* Overlay para móvil */}
      <AnimatePresence>
        {isOpen && !isCollapsed && (
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            onClick={toggleSidebar}
            className="fixed inset-0 bg-black/50 z-40 lg:hidden"
          />
        )}
      </AnimatePresence>

      {/* Sidebar */}
      <AnimatePresence>
        {isOpen && (
          <motion.aside
            initial={{ width: 0 }}
            animate={{ width: isCollapsed ? 68 : 380 }}
            exit={{ width: 0 }}
            transition={{ duration: 0.3, ease: 'easeInOut' }}
            className={`fixed top-0 ${position}-0 h-full bg-[#0a0f1a] border-l border-[#00d4ff]/20 z-50 flex flex-col overflow-hidden`}
            role="complementary"
            aria-label="Barra de notificaciones"
          >
            {/* Header */}
            <div className="flex items-center justify-between p-4 border-b border-[#00d4ff]/20">
              {!isCollapsed && (
                <h2 className="text-lg font-semibold text-white flex items-center gap-2">
                  <Bell className="w-5 h-5 text-[#00d4ff]" />
                  {t('notifications.title', 'Notificaciones')}
                  {unreadCount > 0 && (
                    <span className="bg-[#00d4ff] text-[#0a0f1a] text-xs px-2 py-0.5 rounded-full">
                      {unreadCount}
                    </span>
                  )}
                </h2>
              )}
              <div className="flex items-center gap-1">
                {!isCollapsed && (
                  <button
                    onClick={simulateAlert}
                    className="p-2 text-gray-400 hover:text-[#00d4ff] transition-colors"
                    title="Simular alerta"
                  >
                    <Sun className="w-4 h-4" />
                  </button>
                )}
                <button
                  onClick={toggleCollapse}
                  className="p-2 text-gray-400 hover:text-white transition-colors"
                  aria-label={isCollapsed ? 'Expandir' : 'Colapsar'}
                >
                  {isCollapsed ? <ChevronLeft className="w-4 h-4" /> : <ChevronRight className="w-4 h-4" />}
                </button>
                <button
                  onClick={toggleSidebar}
                  className="p-2 text-gray-400 hover:text-white transition-colors"
                  aria-label="Cerrar"
                >
                  <X className="w-4 h-4" />
                </button>
              </div>
            </div>

            {/* Contenido colapsado */}
            {isCollapsed ? (
              <div className="flex flex-col items-center py-4">
                <Bell className="w-6 h-6 text-[#00d4ff] mb-2" />
                {unreadCount > 0 && (
                  <span className="bg-red-500 text-white text-xs rounded-full w-5 h-5 flex items-center justify-center">
                    {unreadCount > 9 ? '9+' : unreadCount}
                  </span>
                )}
              </div>
            ) : (
              <>
                {/* Lista de notificaciones */}
                <div className="flex-1 overflow-y-auto">
                  {notifications.length === 0 ? (
                    <div className="flex flex-col items-center justify-center h-full text-gray-500">
                      <Bell className="w-12 h-12 mb-2" />
                      <p>{t('notifications.empty', 'No hay notificaciones')}</p>
                    </div>
                  ) : (
                    <div className="p-2 space-y-2">
                      {notifications.map((notification) => (
                        <NotificationItem
                          key={notification.id}
                          notification={notification}
                          onRead={markAsRead}
                          onDismiss={dismiss}
                          severityColors={severityColors}
                          severityIcons={severityIcons}
                        />
                      ))}
                    </div>
                  )}
                </div>

                {/* Footer con acciones */}
                <div className="p-4 border-t border-[#00d4ff]/20 space-y-3">
                  <div className="flex items-center justify-between">
                    <button
                      onClick={toggleDoNotDisturb}
                      className={`flex items-center gap-2 text-sm ${
                        doNotDisturb ? 'text-red-400' : 'text-gray-400'
                      } hover:text-white transition-colors`}
                    >
                      {doNotDisturb ? <VolumeX className="w-4 h-4" /> : <Volume2 className="w-4 h-4" />}
                      {doNotDisturb ? t('notifications.dnd_on', 'No molestar') : t('notifications.dnd_off', 'Sonido activado')}
                    </button>
                    <button
                      onClick={toggleSound}
                      className="text-gray-400 hover:text-white transition-colors"
                      aria-label={soundEnabled ? 'Desactivar sonido' : 'Activar sonido'}
                    >
                      {soundEnabled ? <Volume2 className="w-4 h-4" /> : <VolumeX className="w-4 h-4" />}
                    </button>
                  </div>
                  
                  {notifications.length > 0 && (
                    <div className="flex gap-2">
                      <button
                        onClick={markAllAsRead}
                        className="flex-1 px-3 py-1.5 text-xs bg-[#00d4ff]/10 text-[#00d4ff] rounded hover:bg-[#00d4ff]/20 transition-colors"
                      >
                        {t('notifications.mark_all_read', 'Marcar leído')}
                      </button>
                      <button
                        onClick={clearHistory}
                        className="flex-1 px-3 py-1.5 text-xs bg-red-500/10 text-red-400 rounded hover:bg-red-500/20 transition-colors"
                      >
                        {t('notifications.clear_all', 'Limpiar')}
                      </button>
                    </div>
                  )}
                </div>
              </>
            )}
          </motion.aside>
        )}
      </AnimatePresence>
    </>
  );
};

// Hook para obtener valor anterior
function usePrevious<T>(value: T): T {
  const ref = useRef<T>(value);
  useEffect(() => {
    ref.current = value;
  }, [value]);
  return ref.current;
}

// Item de notificación individual
interface NotificationItemProps {
  notification: Notification;
  onRead: (id: string) => void;
  onDismiss: (id: string) => void;
  severityColors: Record<string, string>;
  severityIcons: Record<string, React.ElementType>;
}

const NotificationItem = ({ 
  notification, 
  onRead, 
  onDismiss, 
  severityColors, 
  severityIcons 
}: NotificationItemProps) => {
  const Icon = severityIcons[notification.severity] || Bell;
  const timeAgo = new Date(notification.timestamp).toLocaleTimeString();

  return (
    <motion.div
      initial={{ opacity: 0, y: -10 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, x: 100 }}
      className={`border-l-4 rounded-r-lg p-3 bg-[#0a0f1a] ${
        severityColors[notification.severity] || severityColors.info
      }`}
    >
      <div className="flex items-start justify-between gap-2">
        <div className="flex-1">
          <div className="flex items-center gap-2 mb-1">
            <Icon className="w-4 h-4 text-white flex-shrink-0" />
            <h4 className="font-medium text-white text-sm">{notification.title}</h4>
          </div>
          <p className="text-gray-400 text-xs mb-2">{notification.description}</p>
          <div className="flex items-center gap-2 text-xs text-gray-500">
            <span>{notification.location}</span>
            <span>•</span>
            <span>{timeAgo}</span>
          </div>
          {notification.affectedNodes.length > 0 && (
            <div className="flex flex-wrap gap-1 mt-2">
              {notification.affectedNodes.map(node => (
                <span key={node} className="text-xs px-1.5 py-0.5 bg-[#00d4ff]/10 text-[#00d4ff] rounded">
                  {node}
                </span>
              ))}
            </div>
          )}
        </div>
        <div className="flex flex-col gap-1">
          {!notification.read && (
            <button
              onClick={() => onRead(notification.id)}
              className="p-1 text-gray-400 hover:text-green-400 transition-colors"
              aria-label="Marcar como leído"
            >
              <Check className="w-3 h-3" />
            </button>
          )}
          <button
            onClick={() => onDismiss(notification.id)}
            className="p-1 text-gray-400 hover:text-red-400 transition-colors"
            aria-label="Descartar"
          >
            <X className="w-3 h-3" />
          </button>
        </div>
      </div>
    </motion.div>
  );
};