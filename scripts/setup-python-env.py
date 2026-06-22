#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
QuantumEnergyOS V.03 — Python Environment Bootstrap

Reads qeos/packages/python-requirements.txt and creates a virtual
environment with the pinned/scientific stack.
"""

from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path

REQ_FILE = Path(__file__).resolve().parents[1] / "python-requirements.txt"
if not REQ_FILE.exists():
    print(f"[!] Requirements file not found: {REQ_FILE}")
    sys.exit(1)

VENV = Path(".venv")
PIP = VENV / "bin" / "pip"


def ensure_venv() -> None:
    if not VENV.exists():
        print(f"[*] Creating virtual environment at {VENV}")
        subprocess.check_call([sys.executable, "-m", "venv", str(VENV)])


def upgrade_pip() -> None:
    print("[*] Upgrading pip...")
    subprocess.check_call([str(PIP), "install", "--upgrade", "pip", "setuptools", "wheel"])


def install_requirements() -> None:
    print(f"[*] Installing requirements from {REQ_FILE}")
    subprocess.check_call([str(PIP), "install", "-r", str(REQ_FILE)])


def main() -> int:
    ensure_venv()
    upgrade_pip()
    install_requirements()
    print("[*] Done. Activate with: source .venv/bin/activate")
    return 0


if __name__ == "__main__":
    sys.exit(main())
