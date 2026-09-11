#!/usr/bin/env python3
"""Shim de arranque de ArcadeZero (ver PLAN.md §3.2).

Uso:
    python launcher.py <carpeta-del-sketch>

Responsabilidades:
1. chdir a la carpeta del sketch (images/ sounds/ music/ resuelven solas).
2. Instalar un sys.excepthook que serializa la excepcion a JSON en stderr
   con el prefijo ##ARCADEZERO## para que Rust lo parsee.
3. Fijar titulo/posicion de la ventana del juego.
4. Arrancar pgzero por API (prepare_mod + PGZeroGame().run()), no por su
   CLI `pgzrun`.
5. Imprimir el codigo de salida al terminar.
"""
from __future__ import annotations

import json
import os
import sys
import traceback
from pathlib import Path

ERROR_PREFIX = "##ARCADEZERO##"

LAUNCHER_DIR = Path(__file__).resolve().parent

# Dev: runtime/launcher.py corre junto a runtime/vendored/pgzero.
# Prod: build_runtime.py copia ambos al mismo nivel en runtime/dist/<plat>/.
# En ambos casos "vendored" es hermano de este archivo.
VENDORED_DIR = LAUNCHER_DIR / "vendored"
if VENDORED_DIR.is_dir() and str(VENDORED_DIR) not in sys.path:
    sys.path.insert(0, str(VENDORED_DIR))


def _serialize_exception(exc_type, exc_value, tb) -> dict:
    """Arma el dict que se emite como JSON. friendly_es/en se rellenan
    aca mismo con lo minimo; el catalogo fino vive en friendly-errors (Fase 3)."""
    if isinstance(exc_value, SyntaxError):
        # SyntaxError no deja frames utiles en tb (falla en compile(), no en
        # exec()); la ubicacion real vive en los atributos del propio error.
        user_file = exc_value.filename
        user_lineno = exc_value.lineno
    else:
        frames = traceback.extract_tb(tb)
        # El ultimo frame que pertenece al codigo del alumno (no a pgzero/launcher).
        user_frame = None
        for frame in reversed(frames):
            if "vendored" not in frame.filename and frame.filename != __file__:
                user_frame = frame
                break
        user_file = user_frame.filename if user_frame else None
        user_lineno = user_frame.lineno if user_frame else None

    message = str(exc_value)
    return {
        "type": exc_type.__name__,
        "message": message,
        "file": user_file,
        "lineno": user_lineno,
        "friendly_es": f"Ocurrio un error: {exc_type.__name__}: {message}",
        "friendly_en": f"An error occurred: {exc_type.__name__}: {message}",
        "traceback": "".join(traceback.format_exception(exc_type, exc_value, tb)),
    }


def arcadezero_excepthook(exc_type, exc_value, tb) -> None:
    payload = _serialize_exception(exc_type, exc_value, tb)
    print(ERROR_PREFIX + json.dumps(payload), file=sys.stderr, flush=True)


def run_sketch(sketch_dir: Path) -> int:
    main_py = sketch_dir / "main.py"
    if not main_py.exists():
        print(f"No encuentro {main_py}", file=sys.stderr)
        return 1

    os.chdir(sketch_dir)

    # Centrar la ventana del juego (SDL respeta esta env var antes de
    # pygame.display.set_mode).
    os.environ.setdefault("SDL_VIDEO_WINDOW_POS", "center")

    from pgzero import runner  # importa pygame-ce y arma pygame.init()

    src = main_py.read_text(encoding="utf-8")
    code = compile(src, str(main_py), "exec", dont_inherit=True)

    import types
    mod = types.ModuleType("main")
    mod.__file__ = str(main_py)
    sys.modules["main"] = mod

    runner.prepare_mod(mod)

    # Titulo por defecto de la ventana si el sketch no define el suyo.
    # (pgzero lee mod.TITLE en cada frame; ver runtime/vendored/pgzero/game.py)
    default_title = f"ArcadeZero — {sketch_dir.name}"

    with runner.temp_window():
        exec(code, mod.__dict__)

    if not hasattr(mod, "TITLE"):
        mod.TITLE = default_title

    import pygame
    pygame.display.init()
    from pgzero.game import PGZeroGame
    PGZeroGame.show_default_icon()

    try:
        runner.run_mod(mod)
    finally:
        pygame.display.quit()
        from pgzero import clock as pgzero_clock
        pgzero_clock.clock.clear()
        sys.modules.pop("main", None)

    return 0


def main() -> int:
    if len(sys.argv) < 2:
        print("Uso: launcher.py <carpeta-del-sketch>", file=sys.stderr)
        return 2
    sketch_dir = Path(sys.argv[1]).resolve()
    if not sketch_dir.is_dir():
        print(f"No es una carpeta: {sketch_dir}", file=sys.stderr)
        return 2

    sys.excepthook = arcadezero_excepthook
    try:
        return run_sketch(sketch_dir)
    except SystemExit as e:
        return int(e.code or 0)
    except Exception:
        arcadezero_excepthook(*sys.exc_info())
        return 1


if __name__ == "__main__":
    exit_code = main()
    print(f"[launcher] exit_code={exit_code}", flush=True)
    sys.exit(exit_code)
