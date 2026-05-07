// ═══════════════════════════════════════════════════════════════════════
//  Cuarzo 4D — Visualización interactiva de las 4 capas + predicción
//  El cubo 3D que tenías en HTML/CSS, ahora nativo con gestos táctiles
// ═══════════════════════════════════════════════════════════════════════

import { useState, useEffect, useRef } from "react";
import { motion, useMotionValue, useTransform, animate } from "framer-motion";
import { Atom, Zap, RotateCw, Download, ChevronRight } from "lucide-react";
import { RadarChart, Radar, PolarGrid, PolarAngleAxis, ResponsiveContainer } from "recharts";
import { useAppStore } from "../store/appStore";
import { triggerImpactFeedback } from "@tauri-apps/plugin-haptics";
import { useTranslation } from "react-i18next";

const LAYER_CONFIGS = [
  { id: 0, label: "quartz4d.layers.physical",      desc: "quartz4d.layers.physicalDesc",       emoji: "⚛️",  color: "#60a5fa" },
  { id: 1, label: "quartz4d.layers.topological",  desc: "quartz4d.layers.topologicalDesc",   emoji: "🔗",  color: "#a78bfa" },
  { id: 2, label: "quartz4d.layers.holographic", desc: "quartz4d.layers.holographicDesc",    emoji: "💎",  color: "#34d399" },
  { id: 3, label: "quartz4d.layers.energy",  desc: "quartz4d.layers.energyDesc",       emoji: "⚡",  color: "#fbbf24" },
];

export default function Quartz4DScreen() {
  const { t } = useTranslation(["common", "quartz4d"]);
  const { quartz, prediction, predictGrid, storeQuartz, loading } = useAppStore();
  const [activeLayer, setActiveLayer] = useState<number | null>(null);
  const [hoursAhead, setHoursAhead]   = useState(24);
  const [rotating, setRotating]       = useState(false);

  // Gestos para rotar el cubo 4D
  const rotateY = useMotionValue(0);
  const rotateX = useMotionValue(-15);

  const handleLayerTap = async (layerId: number) => {
    await triggerImpactFeedback({ style: "Light" }).catch(() => {});
    setActiveLayer(activeLayer === layerId ? null : layerId);
  };

  const handlePredict = async () => {
    await triggerImpactFeedback({ style: "Medium" }).catch(() => {});
    setRotating(true);
    await predictGrid(hoursAhead, 6);
    setTimeout(() => setRotating(false), 1200);
  };

  const handleStore = async () => {
    await triggerImpactFeedback({ style: "Heavy" }).catch(() => {});
    if (!prediction) return;
    const hash = await storeQuartz(
      `prediction-${Date.now()}`,
      prediction,
      2  // Capa holográfica
    );
    alert(`${t("quartz4d.saved", "Guardado en Cuarzo 4D")}\nHash: ${hash.slice(0, 16)}...`);
  };

  // Datos para el radar de capas
  const radarData = LAYER_CONFIGS.map((l) => {
    const layer = prediction?.layers?.find(pl => pl.id === l.id);
    return {
      layer: t(l.label, l.label),
      amplitud:    (layer?.amplitude  ?? 0.5) * 100,
      entanglement:(layer?.entanglement ?? 0.4) * 100,
      activa:      layer?.active ? 100 : 20,
    };
  });

  return (
    <div className="screen-content">
      <div className="screen-header">
        <div>
          <h1 className="screen-title">💎 {t("quartz4d.title", "Cuarzo 4D")}</h1>
          <p className="screen-subtitle">{t("quartz4d.subtitle", "Almacenamiento topológico holográfico")}</p>
        </div>
        {quartz && (
          <div className="badge badge-purple">
            {quartz.majorana_qubits} qubits
          </div>
        )}
      </div>

      {/* Cubo 4D interactivo — drag para rotar */}
      <motion.div
        className="cube-container"
        drag
        dragConstraints={{ left: 0, right: 0, top: 0, bottom: 0 }}
        dragElastic={0}
        onDrag={(_, info) => {
          rotateY.set(rotateY.get() + info.delta.x * 0.5);
          rotateX.set(rotateX.get() - info.delta.y * 0.5);
        }}
      >
        <motion.div
          className="cube-scene"
          style={{
            rotateY,
            rotateX,
            animation: rotating ? "spin-y 1.2s ease-in-out" : undefined,
          }}
        >
          {LAYER_CONFIGS.map((layer, i) => (
            <CubeFace
              key={layer.id}
              layer={layer}
              faceIndex={i}
              isActive={activeLayer === layer.id}
              onTap={() => handleLayerTap(layer.id)}
              layerData={prediction?.layers?.find(l => l.id === layer.id)}
              t={t}
            />
          ))}
          {/* Caras superior e inferior */}
          <div className="cube-face cube-top"    style={{ background: "rgba(96,165,250,0.1)" }}>
            <span>Quantum Energy OS</span>
          </div>
          <div className="cube-face cube-bottom" style={{ background: "rgba(96,165,250,0.1)" }}>
            <span>Kardashev Tipo 1</span>
          </div>
        </motion.div>
        <p className="cube-hint">↕ ↔ {t("quartz4d.dragHint", "Arrastra para rotar")}</p>
      </motion.div>

      {/* Info de capa activa */}
      {activeLayer !== null && (
        <motion.div
          initial={{ opacity: 0, height: 0 }}
          animate={{ opacity: 1, height: "auto" }}
          exit={{ opacity: 0, height: 0 }}
          className="card layer-detail"
          style={{ borderColor: LAYER_CONFIGS[activeLayer].color }}
        >
          <div className="layer-detail-header">
            <span className="layer-emoji">{LAYER_CONFIGS[activeLayer].emoji}</span>
            <div>
              <h3 style={{ color: LAYER_CONFIGS[activeLayer].color }}>
                {t("quartz4d.layer", "Capa")} {t(LAYER_CONFIGS[activeLayer].label, LAYER_CONFIGS[activeLayer].label)}
              </h3>
              <p className="text-muted">{t(LAYER_CONFIGS[activeLayer].desc, LAYER_CONFIGS[activeLayer].desc)}</p>
            </div>
          </div>
          {prediction?.layers?.[activeLayer] && (
            <div className="layer-stats">
              <Stat label={t("quartz4d.stats.amplitude", "Amplitud")}     value={`${(prediction.layers[activeLayer].amplitude * 100).toFixed(1)}%`} />
              <Stat label={t("quartz4d.stats.phase", "Fase")}         value={`${prediction.layers[activeLayer].phase_rad.toFixed(2)} rad`} />
              <Stat label={t("quartz4d.stats.entanglement", "Entrelazamiento")} value={`${(prediction.layers[activeLayer].entanglement * 100).toFixed(0)}%`} />
              <Stat label={t("quartz4d.stats.active", "Estado")}       value={prediction.layers[activeLayer].active ? t("quartz4d.stats.active", "Activa ✓") : t("quartz4d.stats.inactive", "Inactiva")} />
            </div>
          )}
        </motion.div>
      )}

      {/* Predicción cuántica */}
      <div className="card">
        <h3 className="card-title">🔮 {t("quartz4d.prediction", "Predicción Cuántica")}</h3>
        <div className="predict-controls">
          <div className="slider-wrap">
            <label>{t("quartz4d.hoursAhead", "Horas hacia adelante: {{hours}}h", { hours: hoursAhead })}</label>
            <input
              type="range" min={1} max={72} value={hoursAhead}
              onChange={e => setHoursAhead(+e.target.value)}
              className="slider"
            />
          </div>
          <motion.button
            whileTap={{ scale: 0.95 }}
            className="btn btn-primary"
            onClick={handlePredict}
            disabled={loading.predict}
          >
            <RotateCw size={16} className={loading.predict ? "spin" : ""} />
            {loading.predict ? t("quartz4d.calculating", "Calculando...") : t("quartz4d.calculate", "Predecir")}
          </motion.button>
        </div>

        {prediction && (
          <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }}>
            {/* Radar de capas */}
            <ResponsiveContainer width="100%" height={200}>
              <RadarChart data={radarData}>
                <PolarGrid stroke="#334155" />
                <PolarAngleAxis dataKey="layer" tick={{ fill: "#94a3b8", fontSize: 11 }} />
                <Radar name={t("quartz4d.stats.amplitude", "Amplitud")} dataKey="amplitud"
                    stroke="#a78bfa" fill="#a78bfa" fillOpacity={0.25} />
                <Radar name={t("quartz4d.stats.entanglement", "Entrelazamiento")} dataKey="entanglement"
                    stroke="#34d399" fill="#34d399" fillOpacity={0.15} />
              </RadarChart>
            </ResponsiveContainer>

            <div className="prediction-summary">
              <Stat label={t("quartz4d.stats.gridEfficiency", "Eficiencia de red")}    value={`${(prediction.grid_efficiency * 100).toFixed(1)}%`} />
              <Stat label={t("quartz4d.stats.topologicalProtection", "Protección topológica")} value={`${(prediction.topological_protection * 100).toFixed(0)}%`} />
              <Stat label={t("quartz4d.stats.braidOps", "Operaciones braid")}     value={prediction.braid_operations.toString()} />
            </div>

            <motion.button
              whileTap={{ scale: 0.95 }}
              className="btn btn-secondary full-width"
              onClick={handleStore}
            >
              <Download size={14} />
              {t("quartz4d.saveToQuartz", "Guardar en Cuarzo 4D")}
            </motion.button>
          </motion.div>
        )}
      </div>

      {/* Coherencia */}
      {quartz && (
        <div className="card">
          <div className="coherence-row">
            <Atom size={14} className="text-purple-400" />
            <span>{t("quartz4d.coherenceTime", "Tiempo de coherencia")}</span>
            <strong className="text-purple-300">{quartz.coherence_time_ms.toFixed(0)} ms</strong>
          </div>
        </div>
      )}
    </div>
  );
}

function CubeFace({ layer, faceIndex, isActive, onTap, layerData, t }: {
  layer: typeof LAYER_CONFIGS[0];
  faceIndex: number;
  isActive: boolean;
  onTap: () => void;
  layerData?: { amplitude: number; active: boolean };
  t: (key: string, fallback: string) => string;
}) {
  const faceClass = ["cube-front","cube-back","cube-right","cube-left"][faceIndex];
  return (
    <motion.div
      className={`cube-face ${faceClass}`}
      style={{
        background: isActive
          ? `${layer.color}40`
          : "rgba(15, 23, 42, 0.7)",
        border: `1px solid ${layer.color}`,
        boxShadow: isActive ? `0 0 20px ${layer.color}60` : undefined,
      }}
      whileTap={{ scale: 0.97 }}
      onClick={onTap}
    >
      <span className="face-emoji">{layer.emoji}</span>
      <span className="face-label">{t(layer.label, layer.label)}</span>
      {layerData && (
        <span className="face-amp" style={{ color: layer.color }}>
          {(layerData.amplitude * 100).toFixed(0)}%
        </span>
      )}
    </motion.div>
  );
}

function Stat({ label, value }: { label: string; value: string }) {
  return (
    <div className="stat-row">
      <span className="stat-label">{label}</span>
      <span className="stat-value">{value}</span>
    </div>
  );
}