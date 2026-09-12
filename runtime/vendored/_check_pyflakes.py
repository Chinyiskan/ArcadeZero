"""Chequeo estatico soft para el boton "Revisar" (PLAN.md §8).

Corrido por el Tauri command `check_syntax` (src-tauri/src/check.rs) via
`python.exe _check_pyflakes.py <main.py>`. Imprime un JSON de issues a
stdout; Rust lo deserializa tal cual, sin parsear texto de pyflakes.
"""
import ast
import json
import sys

sys.path.insert(0, str(__import__("pathlib").Path(__file__).resolve().parent))

from pyflakes.checker import Checker  # noqa: E402

# Nombres que pgzero inyecta como globals magicos al correr main.py (ver
# runner.py/game.py de pgzero) sin que el alumno los importe. Sin esto,
# pyflakes marcaria "screen"/"keyboard"/etc. como "undefined name" en
# practicamente cualquier sketch valido — puro ruido, va contra el tono no
# punitivo de PLAN.md §8. Misma lista que KNOWN_NAMES en
# src-tauri/src/error_parse.rs (ahi se usa para sugerir typos).
PGZERO_GLOBALS = [
    "screen", "keyboard", "keys", "keymods", "mouse", "images", "sounds",
    "music", "clock", "animate", "Actor", "Rect", "ZRect", "WIDTH", "HEIGHT",
    "TITLE", "ICON", "exit",
]


def check(path: str) -> list[dict]:
    with open(path, encoding="utf-8") as f:
        source = f.read()

    try:
        tree = ast.parse(source, filename=path)
    except SyntaxError as e:
        return [{"line": e.lineno or 1, "col": e.offset or 1, "message": str(e.msg)}]

    checker = Checker(tree, filename=path, builtins=PGZERO_GLOBALS)
    checker.messages.sort(key=lambda m: m.lineno)
    return [
        {"line": m.lineno, "col": m.col + 1, "message": m.message % m.message_args}
        for m in checker.messages
    ]


if __name__ == "__main__":
    print(json.dumps(check(sys.argv[1])))
