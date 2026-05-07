"""
signal_integrity.py — Grid-Scope Virtual Oscilloscope for QuantumEnergyOS V.02
Monitors electrical waveform integrity using Energy Quality Score (EQS)
"""
from __future__ import annotations
from dataclasses import dataclass
from typing import List, Tuple, Optional
from datetime import datetime, timezone
import math
import random

THD_NOMINAL = 0.05
THD_CRITICAL = 0.12
CREST_NOMINAL = 1.41
CREST_SATURATION = 1.2
PHASE_NOMINAL = 0.95
PHASE_RISK = 0.75
EQS_WARNING = 60


@dataclass
class WaveformMetrics:
    thd: float
    crest_factor: float
    power_factor: float
    v_rms: float
    v_peak: float
    frequency: float
    timestamp: str


@dataclass
class EQSResult:
    score: float
    thd_deviation: float
    crest_deviation: float
    phase_deviation: float
    advisory: Optional[str]
    risk_flags: List[str]
    metrics: WaveformMetrics


def generate_waveform(
    n_samples: int = 256,
    fundamental_v: float = 120.0,
    thd_percent: float = 3.0,
    phase_shift: float = 0.0,
    frequency: float = 60.0,
) -> Tuple[List[float], List[float]]:
    """
    Generate voltage/current waveforms with harmonics.
    Returns (voltage, current) waveforms.
    """
    samples = []
    currents = []
    omega = 2 * math.pi * frequency / n_samples

    for i in range(n_samples):
        t = i / n_samples
        v = fundamental_v * math.sin(omega * i + phase_shift)
        i_curr = fundamental_v * math.sin(omega * i) / fundamental_v

        for h in range(2, 6):
            harmonic_v = (thd_percent / 100) * fundamental_v / h
            v += harmonic_v * math.sin(h * omega * i + phase_shift * h)

        samples.append(v)
        currents.append(i_curr * 0.8)

    return samples, currents


def calculate_thd(voltage: List[float], fundamental_v: float = 120.0) -> float:
    """Calculate Total Harmonic Distortion: sqrt(sum(V_n^2 for n>1)) / V_1"""
    n = len(voltage)
    if n == 0:
        return 0.0
    
    fundamental = fundamental_v
    harmonic_power = 0.0
    
    for i in range(n):
        harmonic_power += voltage[i] ** 2
    
    total_power = harmonic_power
    harmonic_power = total_power - (voltage[0] ** 2 if n > 0 else 0)
    
    if fundamental == 0:
        return 0.0
    
    thd_squared = harmonic_power / max(total_power, 0.001)
    return math.sqrt(max(thd_squared, 0.0))


def calculate_crest_factor(voltage: List[float]) -> float:
    """Calculate Crest Factor: V_peak / V_rms (1.41 = pure sine)"""
    v_rms = math.sqrt(sum(v**2 for v in voltage) / len(voltage))
    if v_rms == 0:
        return 0.0
    v_peak = max(abs(v) for v in voltage)
    return v_peak / v_rms


def calculate_phase_shift(voltage: List[float], current: List[float]) -> float:
    """Calculate power factor cos(phi) from voltage/current waveforms."""
    n = len(voltage)
    m = len(current)
    if n == 0 or m == 0 or n != m:
        return PHASE_NOMINAL
    
    p = sum(v * i for v, i in zip(voltage, current)) / n
    v_rms = math.sqrt(sum(v**2 for v in voltage) / n)
    i_rms = math.sqrt(sum(i**2 for i in current) / n)

    if v_rms * i_rms == 0:
        return PHASE_NOMINAL
    
    pf = p / (v_rms * i_rms)
    return max(0.5, min(1.0, abs(pf)))


def calculate_eqs(
    thd: float,
    crest_factor: float,
    power_factor: float,
) -> EQSResult:
    """
    Calculate Energy Quality Score (EQS) from waveform metrics.
    EQS = 100 - (ΔTHD × 0.4) - (ΔCrest × 0.3) - (ΔPhase × 0.3)
    """
    delta_thd = abs(thd - THD_NOMINAL)
    delta_crest = abs(crest_factor - CREST_NOMINAL)
    delta_phase = abs(power_factor - PHASE_NOMINAL)

    max_thd_dev = THD_CRITICAL - THD_NOMINAL
    max_crest_dev = CREST_NOMINAL - CREST_SATURATION
    max_phase_dev = PHASE_NOMINAL - PHASE_RISK

    delta_thd_pct = (delta_thd / max_thd_dev) * 100 if max_thd_dev > 0 else 0
    delta_crest_pct = (delta_crest / max_crest_dev) * 100 if max_crest_dev > 0 else 0
    delta_phase_pct = (delta_phase / max_phase_dev) * 100 if max_phase_dev > 0 else 0

    score = 100.0 - (delta_thd_pct * 0.4) - (delta_crest_pct * 0.3) - (delta_phase_pct * 0.3)
    score = max(0.0, min(100.0, score))

    risk_flags = []
    advisory = None

    if thd > THD_CRITICAL:
        risk_flags.append("THD_CRITICAL")
    if crest_factor < CREST_SATURATION:
        risk_flags.append("TRANSFORMER_SATURATION")
    if power_factor < PHASE_RISK:
        risk_flags.append("ARC_FLASH_RISK")

    if score < EQS_WARNING:
        advisory = f"[ADVISORY] EQS={score:.1f} - Automatic disconnection recommended. Risk factors: {', '.join(risk_flags) if risk_flags else 'none'}"

    return EQSResult(
        score=round(score, 2),
        thd_deviation=round(delta_thd_pct, 2),
        crest_deviation=round(delta_crest_pct, 2),
        phase_deviation=round(delta_phase_pct, 2),
        advisory=advisory,
        risk_flags=risk_flags,
        metrics=WaveformMetrics(
            thd=thd,
            crest_factor=crest_factor,
            power_factor=power_factor,
            v_rms=math.sqrt(sum(v**2 for v in [120.0]) / 1),
            v_peak=max([120.0]) * crest_factor,
            frequency=60.0,
            timestamp=datetime.now(timezone.utc).isoformat(),
        ),
    )


def simulate_grid_scope(node_id: int = 0, load_percent: float = 0.8) -> EQSResult:
    """
    Simulate oscilloscope reading for a grid node.
    Applies realistic stress based on load_percent.
    """
    stress = load_percent * 0.4
    thd = THD_NOMINAL + stress * 0.05 + random.uniform(0.01, 0.03)
    thd = min(thd, THD_CRITICAL * 0.8)

    crest = CREST_NOMINAL - stress * 0.12 + random.uniform(-0.03, 0.03)
    crest = max(CREST_SATURATION + 0.05, min(CREST_NOMINAL, crest))

    pf = PHASE_NOMINAL - stress * 0.1 + random.uniform(-0.03, 0.03)
    pf = max(PHASE_RISK + 0.1, min(PHASE_NOMINAL, pf))

    return calculate_eqs(thd, crest, pf)


def get_scope_status(eqs: float) -> str:
    """Get human-readable status from EQS score."""
    if eqs >= 85:
        return "EXCELLENT"
    elif eqs >= 70:
        return "GOOD"
    elif eqs >= 60:
        return "WARNING"
    elif eqs >= 40:
        return "CRITICAL"
    else:
        return "DANGER"