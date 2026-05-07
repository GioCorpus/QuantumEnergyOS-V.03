"""
test_signal_integrity.py — Tests for Grid-Scope signal integrity module
"""
import pytest
from signal_integrity import (
    calculate_thd,
    calculate_crest_factor,
    calculate_phase_shift,
    calculate_eqs,
    simulate_grid_scope,
    get_scope_status,
    generate_waveform,
    THD_NOMINAL,
    THD_CRITICAL,
    CREST_NOMINAL,
    CREST_SATURATION,
    PHASE_NOMINAL,
    PHASE_RISK,
)


class TestWaveformGeneration:
    def test_generate_waveform_length(self):
        voltage, current = generate_waveform(n_samples=256)
        assert len(voltage) == 256
        assert len(current) == 256

    def test_generate_waveform_with_harmonics(self):
        voltage, _ = generate_waveform(thd_percent=5.0)
        assert voltage is not None
        assert all(isinstance(v, float) for v in voltage)


class TestTHD:
    def test_thd_calculation(self):
        voltage, _ = generate_waveform(thd_percent=3.0)
        thd = calculate_thd(voltage)
        assert 0 <= thd <= 1.0

    def test_thd_nominal_range(self):
        voltage, _ = generate_waveform(thd_percent=THD_NOMINAL * 100)
        thd = calculate_thd(voltage)
        assert thd < THD_CRITICAL


class TestCrestFactor:
    def test_crest_factor_pure_sine(self):
        voltage, _ = generate_waveform(thd_percent=0.0)
        cf = calculate_crest_factor(voltage)
        assert cf >= 1.0
        assert cf <= CREST_NOMINAL * 1.5

    def test_crest_factor_with_distortion(self):
        voltage, _ = generate_waveform(thd_percent=10.0)
        cf = calculate_crest_factor(voltage)
        assert cf > CREST_SATURATION


class TestPhaseShift:
    def test_phase_shift_calculation(self):
        voltage, current = generate_waveform(phase_shift=0.3)
        pf = calculate_phase_shift(voltage, current)
        assert 0 <= pf <= 1.0


class TestEQS:
    def test_eqs_nominal_conditions(self):
        result = calculate_eqs(THD_NOMINAL, CREST_NOMINAL, PHASE_NOMINAL)
        assert 70 <= result.score <= 100

    def test_eqs_degraded_conditions(self):
        result = calculate_eqs(THD_CRITICAL, CREST_SATURATION, PHASE_RISK)
        assert result.score < 60

    def test_eqs_warning_threshold(self):
        result = calculate_eqs(0.08, 1.3, 0.8)
        if result.score < 60:
            assert result.advisory is not None

    def test_eqs_risk_flags(self):
        result = calculate_eqs(0.15, 1.1, 0.7)
        assert 'THD_CRITICAL' in result.risk_flags or 'TRANSFORMER_SATURATION' in result.risk_flags


class TestSimulateGridScope:
    def test_simulate_returns_eqs_result(self):
        result = simulate_grid_scope(node_id=0, load_percent=0.8)
        assert 0 <= result.score <= 100
        assert result.metrics is not None

    def test_simulate_high_load_stress(self):
        result = simulate_grid_scope(node_id=0, load_percent=0.95)
        assert result.score < 100


class TestScopeStatus:
    def test_status_excellent(self):
        assert get_scope_status(90) == "EXCELLENT"

    def test_status_good(self):
        assert get_scope_status(75) == "GOOD"

    def test_status_warning(self):
        assert get_scope_status(62) == "WARNING"

    def test_status_critical(self):
        assert get_scope_status(45) == "CRITICAL"

    def test_status_danger(self):
        assert get_scope_status(30) == "DANGER"


class TestThresholds:
    def test_thd_thresholds(self):
        assert THD_NOMINAL < THD_CRITICAL
        assert THD_NOMINAL == 0.05
        assert THD_CRITICAL == 0.12

    def test_crest_thresholds(self):
        assert CREST_SATURATION < CREST_NOMINAL
        assert CREST_NOMINAL == 1.41
        assert CREST_SATURATION == 1.2

    def test_phase_thresholds(self):
        assert PHASE_RISK < PHASE_NOMINAL
        assert PHASE_NOMINAL == 0.95
        assert PHASE_RISK == 0.75


if __name__ == "__main__":
    pytest.main([__file__, "-v"])