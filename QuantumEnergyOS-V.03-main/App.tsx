// ═══════════════════════════════════════════════════════════════════════
//  QuantumEnergyOS Mobile — App principal
//  Dark mode nativo, gestos táctiles, 4 pantallas principales
// ═══════════════════════════════════════════════════════════════════════

import { useState, useEffect } from "react";
import { Routes, Route, useNavigate, useLocation } from "react-router-dom";
import { AnimatePresence, motion } from "framer-motion";
import { Zap, Grid, Atom, Sun, Settings, Globe, Shield } from "lucide-react";
import { useTranslation } from "react-i18next";

// Screens
import DashboardScreen   from "./screens/Dashboard";
import GridScreen        from "./screens/GridBalance";
import Quartz4DScreen    from "./screens/Quartz4D";
import SolarScreen       from "./screens/Solar";
import SettingsScreen    from "./screens/SettingsScreen";
import QuantumCryptoScreen from "./screens/QuantumCrypto";

// Notification Sidebar
import { NotificationSidebar } from "./components/NotificationSidebar";

// Store global
import { useAppStore }   from "./store/appStore";

// Hooks
import { useLanguage } from "./hooks/useLanguage";

// Tauri plugins
import { isPermissionGranted, requestPermission, sendNotification }
    from "@tauri-apps/plugin-notification";

const NAV_ITEMS = [
  { path: "/",        icon: Zap,      label: "navigation.dashboard" },
  { path: "/grid",    icon: Grid,     label: "navigation.grid"      },
  { path: "/quartz",  icon: Atom,     label: "navigation.quartz"    },
  { path: "/crypto",  icon: Shield,   label: "navigation.crypto"    },
  { path: "/solar",   icon: Sun,      label: "navigation.solar"     },
  { path: "/settings",icon: Settings, label: "navigation.settings"  },
];

export default function App() {
  const navigate    = useNavigate();
  const location    = useLocation();
  const { init, solarRisk } = useAppStore();
  const { t } = useTranslation("common");
  const { currentLanguage, changeLanguage, languages } = useLanguage();

  // Inicializar al montar
  useEffect(() => {
    init();
    setupNotifications();
  }, []);

  // Notificación cuando hay riesgo solar alto
  useEffect(() => {
    if (solarRisk === "HIGH" || solarRisk === "EXTREME") {
      notifyUser(t("common.solarAlert", "⚠️ Alerta Solar"),
        t("common.solarAlertBody", `Tormenta solar ${solarRisk.toLowerCase()} detectada. Red en riesgo.`));
    }
  }, [solarRisk, t]);

  return (
    <div className="app-shell">
      {/* Header con selector de idioma */}
      <header className="app-header">
        <div className="header-title">
          <h1>💎 QuantumEnergyOS</h1>
          <p>Nunca más apagones en Mexicali</p>
        </div>
        <div className="language-selector-container">
          <select
            value={currentLanguage}
            onChange={(e) => changeLanguage(e.target.value)}
            className="language-select"
            aria-label="Select language"
          >
            {languages.map((lang) => (
              <option key={lang.code} value={lang.code}>
                {lang.flag} {lang.name}
              </option>
            ))}
          </select>
        </div>
      </header>

{/* Área de contenido — scroll vertical */}
       <main className="screen-area">
         <NotificationSidebar />
         <AnimatePresence mode="wait">
          <Routes location={location} key={location.pathname}>
            <Route path="/"        element={<PageWrapper><DashboardScreen /></PageWrapper>} />
            <Route path="/grid"    element={<PageWrapper><GridScreen /></PageWrapper>} />
            <Route path="/quartz"  element={<PageWrapper><Quartz4DScreen /></PageWrapper>} />
            <Route path="/crypto"  element={<PageWrapper><QuantumCryptoScreen /></PageWrapper>} />
            <Route path="/solar"   element={<PageWrapper><SolarScreen /></PageWrapper>} />
            <Route path="/settings"element={<PageWrapper><SettingsScreen /></PageWrapper>} />
          </Routes>
        </AnimatePresence>
      </main>

      {/* Bottom navigation — iOS/Android style */}
      <nav className="bottom-nav">
        {NAV_ITEMS.map(({ path, icon: Icon, label }) => {
          const active = location.pathname === path;
          return (
            <button
              key={path}
              className={`nav-item ${active ? "active" : ""}`}
              onClick={() => navigate(path)}
              aria-label={t(label)}
            >
              <Icon size={22} strokeWidth={active ? 2.5 : 1.8} />
              <span className="nav-label">{t(label)}</span>
            </button>
          );
        })}
      </nav>
    </div>
  );
}

function PageWrapper({ children }: { children: React.ReactNode }) {
  return (
    <motion.div
      initial={{ opacity: 0, y: 12 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: -8 }}
      transition={{ duration: 0.22, ease: "easeOut" }}
      className="page"
    >
      {children}
    </motion.div>
  );
}

async function setupNotifications() {
  const granted = await isPermissionGranted();
  if (!granted) {
    const perm = await requestPermission();
    if (perm !== "granted") return;
  }
}

async function notifyUser(title: string, body: string) {
  try {
    const granted = await isPermissionGranted();
    if (granted) {
      await sendNotification({ title, body, icon: "icons/128x128.png" });
    }
  } catch (e) {
    console.warn("Notification error:", e);
  }
}