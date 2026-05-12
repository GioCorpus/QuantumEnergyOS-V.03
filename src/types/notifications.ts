/**
 * Notification Sidebar Types for QuantumEnergyOS V.02
 * Sistema de notificaciones para operadores de sala de control en Mexicali
 */

// Nivel de severidad de la notificación
export type SeverityLevel = 'critical' | 'high' | 'medium' | 'info';

// Tipo de alerta/notificación
export type NotificationType = 
  | 'grid-collapse'       // Colapso/apagón en la red
  | 'climate-risk'        // Riesgo climático (ola de calor, tormenta solar)
  | 'node-overload'       // Sobrecarga de nodos
  | 'sensor-failure'      // Fallo en sensores críticos
  | 'climate-recommendation' // Recomendación del Climate Orchestrator
  | 'system';

// Fuente de la notificación
export type NotificationSource = 
  | 'Climate Orchestrator' 
  | 'Grid Monitor' 
  | 'Solar Forecast'
  | 'System';

// Acción rápida disponible en notificación
export interface QuickAction {
  id: string;
  label: string;
  icon: string; // nombre del icono de Lucide
  variant?: 'primary' | 'secondary' | 'danger';
}

// Notificación completa
export interface Notification {
  id: string;
  type: NotificationType;
  severity: SeverityLevel;
  title: string;
  description: string;
  location: string;
  affectedNodes: string[];
  timestamp: Date;
  source: NotificationSource;
  read: boolean;
  dismissed: boolean;
  actions: QuickAction[];
  autoExpire?: boolean; // Si debe expirar automáticamente
  expiresAt?: Date;
}

// Filtro de notificaciones
export interface NotificationFilter {
  severity: SeverityLevel | 'all';
  type: NotificationType | 'all';
  unreadOnly: boolean;
}

// Estado del store de notificaciones
export interface NotificationState {
  notifications: Notification[];
  isOpen: boolean;
  isCollapsed: boolean;
  filter: NotificationFilter;
  doNotDisturb: boolean;
  soundEnabled: boolean;
  position: 'left' | 'right';
  
  // Acciones
  addNotification: (notification: Omit<Notification, 'id' | 'timestamp' | 'read' | 'dismissed'>) => void;
  markAsRead: (id: string) => void;
  markAllAsRead: () => void;
  dismiss: (id: string) => void;
  dismissAll: () => void;
  toggleSidebar: () => void;
  toggleCollapse: () => void;
  setFilter: (filter: Partial<NotificationFilter>) => void;
  toggleDoNotDisturb: () => void;
  toggleSound: () => void;
  setPosition: (position: 'left' | 'right') => void;
  getUnreadCount: () => number;
  getCriticalCount: () => number;
  clearHistory: () => void;
}

// Payload del WebSocket
export interface ClimateStatusPayload {
  status: 'normal' | 'warning' | 'critical';
  risk_level: 'low' | 'medium' | 'high' | 'extreme';
  recommendations: string[];
  affected_areas: string[];
  temperature?: number;
  solar_activity?: 'low' | 'moderate' | 'high' | 'extreme';
}

// Configuración del WebSocket
export interface WebSocketConfig {
  url: string;
  reconnectInterval: number;
  maxReconnectAttempts: number;
}