#!/usr/bin/env python3
"""Arma el runtime Python embebido de ArcadeZero.

Descarga un build relocatable de CPython 3.13 (python-build-standalone,
Astral), le instala pygame-ce (+ deps de pgzero) con pip, y copia
runtime/vendored/pgzero y runtime/launcher.py junto a el.

Resultado (no se commitea, ver .gitignore):

    runtime/dist/<platform>/
        python/            interprete standalone + site-packages
        vendored/pgzero/   copia de runtime/vendored/pgzero
        launcher.py        copia de runtime/launcher.py

Uso:
    python runtime/build_runtime.py

Ladder: python-build-standalone + `pip install` directo al arbol del
runtime resuelve el 90% del problema. No hay empaquetador de wheels
propio aqui a proposito.

Integridad del interprete descargado: la primera vez que se corre este
script para un asset nuevo (nuevo PBS_TAG), el sha256 de la descarga se
registra en `runtime/pbs_sha256.json` y hay que commitear ese archivo. De ahi
en mas, cualquier corrida (CI incluido) que descargue un archivo con otro
hash aborta en vez de empaquetar un runtime sin verificar.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import platform
import shutil
import subprocess
import sys
import tarfile
import urllib.request
from pathlib import Path

RUNTIME_DIR = Path(__file__).resolve().parent
REPO_ROOT = RUNTIME_DIR.parent

# Hashes conocidos de los assets de python-build-standalone que ya bajamos
# alguna vez (commiteado, a diferencia de runtime/dist y runtime/.cache — ver
# .gitignore). Primera vez que se agrega un asset nuevo: no hay nada que
# comparar, se registra el hash de esa descarga como "de confianza" (TOFU) y
# se commitea; de ahi en mas cualquier descarga (CI incluido) que no calce
# con lo commiteado aborta el build en vez de empaquetar un runtime corrupto
# o alterado en silencio.
PBS_SHA256_FILE = RUNTIME_DIR / "pbs_sha256.json"

# Pinned python-build-standalone release. Bump deliberadamente, no auto-latest,
# para que el build sea reproducible. Ver:
# https://github.com/astral-sh/python-build-standalone/releases
PBS_TAG = "20260901"
PBS_PYTHON_VERSION = "3.13.15"

# pgzero pide pygame>=2.1; pygame-ce se instala como modulo `pygame` y cumple
# esa condicion sin traer pygame clasico (ver PLAN.md §3.4).
PYGAME_CE_VERSION = "2.5.8"
PGZERO_EXTRA_DEPS = ["numpy", "pyfxr"]

PLATFORM_ASSETS = {
    # (system, machine) -> nombre de asset en el release de python-build-standalone
    ("Windows", "AMD64"): (
        f"cpython-{PBS_PYTHON_VERSION}+{PBS_TAG}-x86_64-pc-windows-msvc-install_only_stripped.tar.gz",
        "windows-x64",
    ),
}


def current_platform_key() -> tuple[str, str]:
    return (platform.system(), platform.machine())


def load_known_hashes() -> dict[str, str]:
    if not PBS_SHA256_FILE.is_file():
        return {}
    return json.loads(PBS_SHA256_FILE.read_text(encoding="utf-8"))


def verify_or_pin_hash(asset_name: str, actual_sha256: str) -> None:
    known = load_known_hashes()
    expected = known.get(asset_name)
    if expected is None:
        known[asset_name] = actual_sha256
        PBS_SHA256_FILE.write_text(
            json.dumps(known, indent=2, sort_keys=True) + "\n", encoding="utf-8"
        )
        print(
            f"[build_runtime] primer uso de {asset_name}: hash sha256 registrado en "
            f"{PBS_SHA256_FILE.name} (revisa y commitea este archivo)."
        )
        return
    if expected != actual_sha256:
        raise SystemExit(
            f"El sha256 de {asset_name} no coincide con el registrado en "
            f"{PBS_SHA256_FILE.name}: esperado {expected}, descargado {actual_sha256}. "
            "La descarga puede estar corrupta o el asset fue alterado; no se arma el "
            "runtime con un interprete sin verificar."
        )


def download(url: str, dest: Path, asset_name: str) -> None:
    if dest.exists():
        print(f"[build_runtime] ya descargado: {dest.name}")
        verify_or_pin_hash(asset_name, sha256_of(dest))
        return
    print(f"[build_runtime] descargando {url}")
    dest.parent.mkdir(parents=True, exist_ok=True)
    tmp = dest.with_suffix(dest.suffix + ".part")
    urllib.request.urlretrieve(url, tmp)
    verify_or_pin_hash(asset_name, sha256_of(tmp))
    tmp.rename(dest)


def extract_tar_gz(archive: Path, dest_dir: Path) -> None:
    print(f"[build_runtime] extrayendo {archive.name} -> {dest_dir}")
    if dest_dir.exists():
        shutil.rmtree(dest_dir)
    dest_dir.mkdir(parents=True)
    with tarfile.open(archive, "r:gz") as tf:
        tf.extractall(dest_dir)


def python_build_standalone_url(asset_name: str) -> str:
    return (
        f"https://github.com/astral-sh/python-build-standalone/releases/"
        f"download/{PBS_TAG}/{asset_name}"
    )


def find_python_exe(extracted_dir: Path) -> Path:
    # install_only(_stripped) builds extraen a <extracted_dir>/python/...
    candidates = list(extracted_dir.glob("python/**/python.exe"))
    if not candidates:
        candidates = list(extracted_dir.glob("python/**/python3"))
    if not candidates:
        raise SystemExit(f"No encontre el interprete extraido en {extracted_dir}")
    # Preferimos el ejecutable de nivel mas alto (install_only pone python.exe
    # directo en python/python.exe en Windows).
    candidates.sort(key=lambda p: len(p.parts))
    return candidates[0]


def pip_install(python_exe: Path, packages: list[str]) -> None:
    print(f"[build_runtime] pip install {' '.join(packages)}")
    subprocess.run(
        [str(python_exe), "-m", "pip", "install", "--no-cache-dir", *packages],
        check=True,
    )


def trim_runtime(python_dir: Path) -> None:
    """Recorte de tamano seguro y gratis (R2 en PLAN.md §10): borra bytecode
    cacheado (se regenera solo) y pip (solo hace falta en build-time, no en
    runtime del alumno). No toca numpy/pygame-ce: son dependencias reales de
    pgzero (tone.py/ptext.py), recortarlas es trabajo aparte, no de este spike.
    """
    removed_bytes = 0

    for cache_dir in python_dir.rglob("__pycache__"):
        removed_bytes += sum(f.stat().st_size for f in cache_dir.rglob("*") if f.is_file())
        shutil.rmtree(cache_dir, ignore_errors=True)

    site_packages = python_dir / "Lib" / "site-packages"
    for pattern in ("pip", "pip-*.dist-info"):
        for path in site_packages.glob(pattern):
            if path.is_dir():
                removed_bytes += sum(f.stat().st_size for f in path.rglob("*") if f.is_file())
                shutil.rmtree(path, ignore_errors=True)

    scripts_dir = python_dir / "Scripts"
    for pattern in ("pip*.exe",):
        for path in scripts_dir.glob(pattern):
            removed_bytes += path.stat().st_size
            path.unlink(missing_ok=True)

    print(f"[build_runtime] trim: liberados ~{removed_bytes / 1_000_000:.1f} MB")


# Submodulos de numpy que pgzero no toca (confirmado: tone.py/ptext.py solo
# usan numpy.clip/arange/arrays basicos). Son lazy-loaded via __getattr__ en
# numpy/__init__.py, asi que borrarlos no rompe `import numpy`. El resto del
# peso de numpy (numpy.libs + numpy/_core, ~30 MB de BLAS/extensiones
# compiladas) es la libreria matematica real: no se recorta aqui, ver nota en
# PLAN.md §10 (deuda tecnica).
NUMPY_DEAD_WEIGHT_DIRS = ["f2py", "polynomial"]


def trim_numpy(site_packages: Path) -> None:
    numpy_dir = site_packages / "numpy"
    if not numpy_dir.exists():
        return
    removed_bytes = 0

    for name in NUMPY_DEAD_WEIGHT_DIRS:
        path = numpy_dir / name
        if path.exists():
            removed_bytes += sum(f.stat().st_size for f in path.rglob("*") if f.is_file())
            shutil.rmtree(path, ignore_errors=True)

    # Directorios de tests (no se ejecutan en runtime) y stubs .pyi (solo
    # para type-checkers como mypy, no se leen al importar).
    for tests_dir in numpy_dir.rglob("tests"):
        if tests_dir.is_dir():
            removed_bytes += sum(f.stat().st_size for f in tests_dir.rglob("*") if f.is_file())
            shutil.rmtree(tests_dir, ignore_errors=True)
    for pyi_file in numpy_dir.rglob("*.pyi"):
        removed_bytes += pyi_file.stat().st_size
        pyi_file.unlink(missing_ok=True)

    print(f"[build_runtime] trim numpy: liberados ~{removed_bytes / 1_000_000:.1f} MB")


def copy_vendored_and_launcher(dist_dir: Path) -> None:
    vendored_src = RUNTIME_DIR / "vendored"
    vendored_dst = dist_dir / "vendored"
    if vendored_dst.exists():
        shutil.rmtree(vendored_dst)
    shutil.copytree(vendored_src, vendored_dst)

    launcher_src = RUNTIME_DIR / "launcher.py"
    shutil.copy2(launcher_src, dist_dir / "launcher.py")


def sha256_of(path: Path) -> str:
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--out",
        type=Path,
        default=RUNTIME_DIR / "dist",
        help="Directorio de salida (default: runtime/dist)",
    )
    parser.add_argument(
        "--skip-pip",
        action="store_true",
        help="No reinstalar pygame-ce/deps si el interprete ya existe (rapido para iterar en el launcher).",
    )
    args = parser.parse_args()

    key = current_platform_key()
    if key not in PLATFORM_ASSETS:
        raise SystemExit(
            f"Plataforma no soportada todavia: {key}. "
            f"Fase 0 solo cubre Windows x86_64 (ver PLAN.md §11)."
        )
    asset_name, platform_dir_name = PLATFORM_ASSETS[key]

    cache_dir = RUNTIME_DIR / ".cache"
    archive_path = cache_dir / asset_name
    download(python_build_standalone_url(asset_name), archive_path, asset_name)

    dist_dir = args.out / platform_dir_name
    python_dir = dist_dir / "python"

    if args.skip_pip and (python_dir / "python.exe").exists():
        print("[build_runtime] --skip-pip: reutilizando interprete existente")
    else:
        extract_tmp = cache_dir / f"extract-{platform_dir_name}"
        extract_tar_gz(archive_path, extract_tmp)
        python_exe = find_python_exe(extract_tmp)
        extracted_python_root = python_exe.parent

        if python_dir.exists():
            shutil.rmtree(python_dir)
        dist_dir.mkdir(parents=True, exist_ok=True)
        shutil.move(str(extracted_python_root), str(python_dir))
        shutil.rmtree(extract_tmp, ignore_errors=True)

        python_exe = python_dir / "python.exe"
        pip_install(python_exe, [f"pygame-ce=={PYGAME_CE_VERSION}", *PGZERO_EXTRA_DEPS])
        trim_runtime(python_dir)
        trim_numpy(python_dir / "Lib" / "site-packages")

    copy_vendored_and_launcher(dist_dir)

    print(f"[build_runtime] listo: {dist_dir}")
    print(f"[build_runtime] sha256 del release: {sha256_of(archive_path)}")


if __name__ == "__main__":
    main()
