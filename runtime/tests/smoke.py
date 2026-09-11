#!/usr/bin/env python3
"""Smoke test del runtime embebido, pensado para CI (ver PLAN.md §11 Fase 0).

Corre con el Python EMBEBIDO (el de runtime/dist/<plat>/python/), no con
el Python del sistema:

    runtime/dist/windows-x64/python/python.exe runtime/tests/smoke.py

Chequea:
1. `import pgzero; import pygame` no truena.
2. El sketch de runtime/tests/fixture_sketch corre unos frames via
   launcher.py y sale con codigo 0 (usa ARCADEZERO_AUTOCLOSE_FRAMES).
3. Provocar un crash a proposito (ARCADEZERO_CRASH_AFTER) y confirmar que
   el excepthook emite una linea ##ARCADEZERO##{...} valida por stderr.

Sin pytest / frameworks: son tres asserts y `sys.exit` con el resultado.
"""
from __future__ import annotations

import json
import os
import subprocess
import sys
from pathlib import Path

THIS_DIR = Path(__file__).resolve().parent
RUNTIME_DIR = THIS_DIR.parent
FIXTURE_SKETCH = THIS_DIR / "fixture_sketch"
LAUNCHER = RUNTIME_DIR / "launcher.py"
ERROR_PREFIX = "##ARCADEZERO##"

# launcher.py hace este mismo insert cuando corre como subproceso; lo
# repetimos aca para que el `import pgzero` de check_imports() tambien
# encuentre el vendorizado (dev: runtime/vendored, prod: sidecar/vendored).
VENDORED_DIR = RUNTIME_DIR / "vendored"
if VENDORED_DIR.is_dir() and str(VENDORED_DIR) not in sys.path:
    sys.path.insert(0, str(VENDORED_DIR))


def check_imports() -> None:
    import pgzero  # noqa: F401
    import pygame  # noqa: F401
    print(f"[smoke] pygame {pygame.version.ver}, pgzero {pgzero.__version__} importan OK")


def run_launcher(env_extra: dict) -> subprocess.CompletedProcess:
    env = {**os.environ, "SDL_VIDEODRIVER": os.environ.get("SDL_VIDEODRIVER", "dummy"), **env_extra}
    return subprocess.run(
        [sys.executable, str(LAUNCHER), str(FIXTURE_SKETCH)],
        capture_output=True,
        text=True,
        timeout=30,
        env=env,
    )


def check_clean_run() -> None:
    result = run_launcher({"ARCADEZERO_AUTOCLOSE_FRAMES": "5"})
    print("[smoke] --- stdout (clean run) ---")
    print(result.stdout)
    print("[smoke] --- stderr (clean run) ---")
    print(result.stderr)
    assert result.returncode == 0, f"esperaba exit 0, dio {result.returncode}"
    assert "frame 1" in result.stdout, "no vi salida de update() en stdout"
    print("[smoke] corrida limpia OK (exit 0, stdout con frames)")


def check_crash_excepthook() -> None:
    result = run_launcher({"ARCADEZERO_CRASH_AFTER": "2"})
    print("[smoke] --- stderr (crash run) ---")
    print(result.stderr)
    assert result.returncode != 0, "esperaba exit != 0 tras el crash"

    error_line = next(
        (line for line in result.stderr.splitlines() if line.startswith(ERROR_PREFIX)),
        None,
    )
    assert error_line is not None, "no encontre una linea ##ARCADEZERO## en stderr"

    payload = json.loads(error_line[len(ERROR_PREFIX):])
    assert payload["type"] == "RuntimeError", payload
    assert "crash de prueba" in payload["message"], payload
    assert payload["file"] and payload["file"].endswith("main.py"), payload
    assert isinstance(payload["lineno"], int), payload
    print(f"[smoke] excepthook OK: {payload['type']} en linea {payload['lineno']}")


def main() -> int:
    if not FIXTURE_SKETCH.exists():
        print(f"[smoke] falta el fixture: {FIXTURE_SKETCH}", file=sys.stderr)
        return 1

    check_imports()
    check_clean_run()
    check_crash_excepthook()

    print("[smoke] TODO OK")
    return 0


if __name__ == "__main__":
    sys.exit(main())
