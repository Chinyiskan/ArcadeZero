# ArcadeZero

ArcadeZero es un editor de escritorio para que estudiantes escriban y ejecuten
sus propios juegos en Python usando [Pygame Zero](https://pygame-zero.readthedocs.io/)
(pgzero). Nació como herramienta de clase para enseñar programación: trae todo
lo necesario para escribir código y ver el juego correr, sin que el estudiante
tenga que instalar Python, pip ni ninguna dependencia por su cuenta.

Windows es la única plataforma soportada por ahora.

## Qué incluye

- Editor de código con resaltado de sintaxis para Python y autocompletado de
  la API de pgzero (`screen`, `keyboard`, `Actor`, `clock`, etc.).
- Botón "Jugar"/"Detener" que corre el juego como proceso aparte, sin
  congelar el editor.
- Errores de Python traducidos a mensajes que un principiante puede entender
  (catálogo en español e inglés, con sugerencias tipo "¿quisiste decir...?"
  para errores de tipeo comunes).
- Botón "Revisar": chequeo estático con [pyflakes](https://github.com/PyCQA/pyflakes)
  antes de ejecutar (variables sin usar, nombres no definidos), con el aviso
  subrayado directamente en el editor.
- Panel de assets (imágenes, sonidos, música) con arrastrar y soltar.
- Plantillas de proyecto para empezar rápido (`en-blanco`, `mi-primer-sprite`,
  `plataformas-basico`).
- Temas visuales (Día, Dracula, One Dark Pro, Alto contraste) y accesibilidad
  básica: fuente OpenDyslexic opcional, escalado de la interfaz.
- Auto-actualización: al publicar una versión nueva en GitHub Releases, la
  app instalada avisa sola y se actualiza con un clic.

## Tecnologías

| Capa | Tecnología |
|---|---|
| Interfaz | [Svelte 5](https://svelte.dev/) + SvelteKit, TypeScript, [Vite](https://vite.dev/) |
| Editor de código | [CodeMirror 6](https://codemirror.net/) |
| Aplicación de escritorio | [Tauri 2](https://tauri.app/) (Rust) |
| Motor del juego | [pygame-ce](https://pyga.me/) + [Pygame Zero](https://pygame-zero.readthedocs.io/) (vendorizado y parcheado en `runtime/vendored/pgzero/`) |
| Runtime de Python | [python-build-standalone](https://github.com/astral-sh/python-build-standalone) embebido — no depende de un Python instalado en la máquina |
| Chequeo estático | [pyflakes](https://github.com/PyCQA/pyflakes) (vendorizado, puro Python) |
| Empaquetado / instalador | Tauri bundler (NSIS), `tauri-plugin-updater` |
| CI/CD | GitHub Actions |

## Arquitectura

ArcadeZero es una app de Tauri: la interfaz es una página web (Svelte) que
corre dentro de una ventana nativa, y toda la lógica que toca el sistema
(procesos, archivos, subprocesos) vive en Rust. Los dos lados se hablan por
comandos e eventos de Tauri, no por una API HTTP.

```
┌─────────────────────────┐        comandos Tauri        ┌──────────────────────────┐
│   src/  (Svelte, UI)     │ ────────────────────────────>│  src-tauri/  (Rust)      │
│  editor, toolbar,        │<──────────────────────────── │  ProjectManager,         │
│  consola, panel de       │        eventos (run_stdout,   │  RunController,          │
│  assets, ayuda           │        run_error, run_exit)   │  AssetService, Settings  │
└─────────────────────────┘                                └───────────┬──────────────┘
                                                                        │ spawnea
                                                                        v
                                                    ┌────────────────────────────────────┐
                                                    │  runtime/ (Python embebido)          │
                                                    │  launcher.py -> pgzero -> pygame-ce  │
                                                    │  vendored/pgzero, vendored/pyflakes  │
                                                    └────────────────────────────────────┘
```

Flujo típico al apretar "Jugar":

1. El frontend llama al comando `run_project` con la carpeta del sketch.
2. Rust (`src-tauri/src/run.rs`) guarda el `main.py` y lanza el Python
   embebido corriendo `runtime/launcher.py`, que a su vez prepara y arranca
   el juego con la API de pgzero.
3. `stdout`/`stderr` del proceso hijo se transmiten al frontend como eventos
   (`run_stdout`, `run_stderr`) y se muestran en la consola en vivo.
4. Si el juego revienta, `launcher.py` intercepta la excepción con un
   `sys.excepthook`, la serializa a JSON y la manda por stderr con un
   prefijo especial. Rust (`error_parse.rs`) la parsea, la cruza contra un
   catálogo de errores comunes de principiante y la manda al frontend como
   un evento `run_error` ya traducido a un mensaje amigable.

El botón "Revisar" sigue un camino más corto: Rust corre el Python embebido
contra `_check_pyflakes.py` (que llama a pyflakes con una lista de nombres
mágicos de pgzero para no marcar `screen`/`keyboard`/etc. como error), y el
resultado se muestra como subrayado en el editor (reusando el sistema de
lint de CodeMirror) y como lista en la consola.

### Por qué un runtime de Python embebido

La idea es que un estudiante baje el instalador, lo corra, y ya pueda
programar — sin instalar Python, sin `pip install`, sin variables de
entorno. `runtime/build_runtime.py` arma ese runtime: descarga un build
standalone de CPython, le instala `pygame-ce` (que reemplaza a `pygame`
clásico, ver `PYGAME_CE_VERSION` en ese script) y sus dependencias, y copia
`pgzero` y `pyflakes` ya vendorizados. Ese runtime armado (`runtime/dist/`)
no se commitea al repo — lo arma cada quien localmente o el propio CI antes
de compilar el instalador.

## Estructura del repositorio

```
src/                 Interfaz (Svelte). Componentes en src/lib/, ruta
                      principal en src/routes/+page.svelte.
src-tauri/            Backend Rust: comandos, spawn del proceso del juego,
                      manejo de assets/settings, parser de errores.
runtime/              Constructor del runtime embebido (build_runtime.py),
                      el shim de arranque (launcher.py), y el código
                      vendorizado de pgzero/pyflakes.
templates/            Plantillas de proyecto que ve el estudiante al crear
                      un sketch nuevo.
examples/             Proyectos de ejemplo.
docs/                 Cheatsheet de la API de pgzero y catálogo de errores
                      amigables, pensados como referencia legible (no solo
                      para desarrolladores).
.github/workflows/    CI (tests en cada push/PR) y Release (arma el
                      instalador y publica un borrador de Release al
                      taggear una versión).
```

## Desarrollo local

Requisitos: [pnpm](https://pnpm.io/), un toolchain de [Rust](https://rustup.rs/)
estable, y Python 3 en el sistema (solo para armar el runtime embebido, no
hace falta para el resto del desarrollo).

```bash
pnpm install
python runtime/build_runtime.py   # arma runtime/dist/, hace falta una vez
                                   # (y de nuevo si cambia algo en runtime/vendored/)
pnpm tauri dev
```

Para correr los tests:

```bash
pnpm run check   # type-check de Svelte/TypeScript
pnpm test        # tests del frontend (Vitest)
cd src-tauri && cargo test   # tests del backend (Rust)
```

## Cómo contribuir

Cualquiera puede usar, modificar y proponer cambios a ArcadeZero (ver
[LICENSE](LICENSE), MIT).

1. Forkeá el repo y creá una rama para tu cambio.
2. Seguí la estructura de arriba: cambios de interfaz van en `src/`, de
   backend en `src-tauri/src/`, de runtime/empaquetado Python en `runtime/`.
3. Si tocás lógica no trivial, agregá o actualizá los tests correspondientes
   (Vitest para TypeScript, `cargo test` para Rust, o un script de asserts
   simple al estilo de `runtime/tests/smoke.py` para Python).
4. Corré `pnpm run check`, `pnpm test` y `cargo test` antes de abrir el PR —
   es lo mismo que corre `ci.yml` en cada Pull Request.
5. Abrí el Pull Request describiendo qué cambia y por qué.

Ideas de contribución bienvenidas: soporte para macOS/Linux, firma de
código para el instalador de Windows, más plantillas de proyecto, más casos
en el catálogo de errores amigables, traducciones adicionales de la
interfaz.

## Publicar una versión nueva

Le pega a cualquiera con permisos de escritura en el repo:

```bash
git tag vX.Y.Z
git push origin vX.Y.Z
```

Esto dispara el workflow de Release: arma el runtime, compila el instalador
NSIS, y deja un borrador de Release en GitHub con el instalador adjunto. Se
revisa y se publica a mano — nunca sale público sin esa confirmación.

## Licencia

[MIT](LICENSE).
