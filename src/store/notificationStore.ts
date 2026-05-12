/**
 * Notification Store - Zustand
 * Gestión de estado para el Notification Sidebar
 */

import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import type { 
  Notification, 
  NotificationState, 
  NotificationFilter,
  SeverityLevel,
  NotificationType 
} from '../types/notifications';

const MAX_NOTIFICATIONS = 50;

const defaultFilter: NotificationFilter = {
  severity: 'all',
  type: 'all',
  unreadOnly: false,
};

export const useNotificationStore = create<NotificationState>()(
  persist(
    (set, get) => ({
      notifications: [],
      isOpen: true,
      isCollapsed: false,
      filter: defaultFilter,
      doNotDisturb: false,
      soundEnabled: true,
      position: 'right',

      addNotification: (notificationData) => {
        const newNotification: Notification = {
          ...notificationData,
          id: `notif-${Date.now()}-${Math.random().toString(36).slice(2, 9)}`,
          timestamp: new Date(),
          read: false,
          dismissed: false,
        };

        set((state) => {
          const notifications = [newNotification, ...state.notifications]
            .slice(0, MAX_NOTIFICATIONS);
          return { notifications };
        });
      },

      markAsRead: (id) => {
        set((state) => ({
          notifications: state.notifications.map((n) =>
            n.id === id ? { ...n, read: true } : n
          ),
        }));
      },

      markAllAsRead: () => {
        set((state) => ({
          notifications: state.notifications.map((n) => ({ ...n, read: true })),
        }));
      },

      dismiss: (id) => {
        set((state) => ({
          notifications: state.notifications.map((n) =>
            n.id === id ? { ...n, dismissed: true } : n
          ),
        }));
      },

      dismissAll: () => {
        set((state) => ({
          notifications: state.notifications.map((n) => ({ ...n, dismissed: true })),
        }));
      },

      toggleSidebar: () => {
        set((state) => ({ isOpen: !state.isOpen }));
      },

      toggleCollapse: () => {
        set((state) => ({ isCollapsed: !state.isCollapsed }));
      },

      setFilter: (filter) => {
        set((state) => ({
          filter: { ...state.filter, ...filter },
        }));
      },

      toggleDoNotDisturb: () => {
        set((state) => ({ doNotDisturb: !state.doNotDisturb }));
      },

      toggleSound: () => {
        set((state) => ({ soundEnabled: !state.soundEnabled }));
      },

      setPosition: (position) => {
        set({ position });
      },

      getUnreadCount: () => {
        return get().notifications.filter((n) => !n.read && !n.dismissed).length;
      },

      getCriticalCount: () => {
        return get().notifications.filter(
          (n) => (n.severity === 'critical' || n.severity === 'high') && !n.dismissed
        ).length;
      },

      clearHistory: () => {
        set({ notifications: [] });
      },
    }),
    {
      name: 'qeos-notifications',
      partialize: (state) => ({
        notifications: state.notifications,
        isCollapsed: state.isCollapsed,
        doNotDisturb: state.doNotDisturb,
        soundEnabled: state.soundEnabled,
        position: state.position,
      }),
    }
  )
);

// Selectores útiles
export const useUnreadCount = () => useNotificationStore((s) => s.getUnreadCount());
export const useCriticalCount = () => useNotificationStore((s) => s.getCriticalCount());
export const useFilteredNotifications = () => {
  const { notifications, filter } = useNotificationStore();
  return notifications.filter((n) => {
    if (n.dismissed) return false;
    if (filter.severity !== 'all' && n.severity !== filter.severity) return false;
    if (filter.type !== 'all' && n.type !== filter.type) return false;
    if (filter.unreadOnly && n.read) return false;
    return true;
  });
};