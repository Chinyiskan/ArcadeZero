# Cheatsheet de pgzero / pgzero cheatsheet

> Referencia rápida de la API de Pygame Zero que trae ArcadeZero (vendorizada en
> `runtime/vendored/pgzero/`, ver `PLAN.md` §2 y §3.4). Todo lo de este documento
> existe de verdad en ese código — es la fuente de datos para el autocompletado
> de `editor-ux` y el catálogo de errores de `friendly-errors`.
>
> No hace falta `import` nada de esto en `main.py`: pgzero inyecta estos nombres
> como si fueran variables globales del archivo (ver `runtime/launcher.py` →
> `runner.prepare_mod`).

---

## Variables de configuración del sketch

Se escriben sueltas, al principio de `main.py` (todas opcionales):

| Nombre | Tipo | Para qué sirve |
|---|---|---|
| `WIDTH` | `int` | Ancho de la ventana en píxeles. Por defecto `800`. |
| `HEIGHT` | `int` | Alto de la ventana en píxeles. Por defecto `600`. |
| `TITLE` | `str` | Título de la ventana. Por defecto `"Pygame Zero Game"`. |
| `ICON` | `str` | Nombre de un archivo en `images/` para el ícono de la ventana. |

---

## Las funciones que pgzero llama por ti

Defínelas en `main.py` con estos nombres exactos — pgzero las busca solas, no
hay que registrarlas en ningún lado.

| Función | Cuándo se llama | Firma |
|---|---|---|
| `draw()` | Una vez por cuadro, para dibujar. **No puede recibir parámetros.** | `def draw():` |
| `update()` / `update(dt)` | Una vez por cuadro, antes de `draw()`, para mover cosas y leer teclado/ratón. `dt` es el tiempo en segundos desde el cuadro anterior. | `def update(dt):` o `def update():` |
| `on_key_down(key)` | Al presionar una tecla. | `def on_key_down(key):` |
| `on_key_up(key)` | Al soltar una tecla. | `def on_key_up(key):` |
| `on_mouse_down(pos, button)` | Al hacer clic. | `def on_mouse_down(pos, button):` |
| `on_mouse_up(pos, button)` | Al soltar el clic. | `def on_mouse_up(pos, button):` |
| `on_mouse_move(pos, rel, buttons)` | Al mover el ratón. | `def on_mouse_move(pos):` |
| `on_music_end()` | Cuando termina la música de fondo. | `def on_music_end():` |

Todos los parámetros de los handlers son opcionales de declarar: pgzero solo
pasa los que tu función pide, por nombre (ej. `def on_key_down(key):` funciona,
`def on_key_down():` también).

---

## `Actor` — personajes/sprites

```python
alien = Actor("alien", (100, 50))          # imagen images/alien.png, en esa posición
alien = Actor("alien", topleft=(0, 0))     # posición con ancla simbólica
```

- La imagen va en `images/<nombre>.png` (o `.gif`, `.jpg`, `.jpeg`, `.bmp`, `.webp`) — **minúsculas, sin espacios ni acentos**.

| Miembro | Qué es |
|---|---|
| `.pos` | Tupla `(x, y)` del punto de anclaje (por defecto el centro). |
| `.x`, `.y` | Coordenadas por separado. |
| `.image` | Nombre de la imagen actual (se puede reasignar para "cambiar de sprite"). |
| `.angle` | Rotación en grados. |
| `.opacity` | Transparencia, de `0.0` (invisible) a `1.0` (opaco). |
| `.anchor` | Punto de la imagen usado como referencia (`("center", "center")` por defecto). |
| `.width`, `.height` | Tamaño en píxeles (delegado del rect interno). |
| `.top`, `.bottom`, `.left`, `.right` | Bordes del rectángulo del sprite. |
| `.topleft`, `.topright`, `.bottomleft`, `.bottomright`, `.midtop`, `.midbottom`, `.midleft`, `.midright`, `.center` | Posiciones simbólicas del rectángulo (igual que `pygame.Rect`). |
| `.draw()` | Dibuja el actor en `screen` (llamalo dentro de tu `draw()`). |
| `.angle_to(target)` | Ángulo en grados hacia otro Actor o punto `(x, y)`. |
| `.distance_to(target)` | Distancia en píxeles hacia otro Actor o punto. |
| `.colliderect(other)` | `True` si el rectángulo del actor se superpone con otro rect/Actor. |
| `.collidepoint(x, y)` | `True` si el punto cae dentro del actor. |
| `.unload_image()` | Libera la imagen de la caché (raro que haga falta). |

---

## `screen` — dibujar en la ventana

| Miembro | Qué hace |
|---|---|
| `screen.clear()` | Limpia la pantalla a negro. |
| `screen.fill(color)` | Rellena la pantalla con un color, ej. `screen.fill((30, 30, 40))`. |
| `screen.blit(imagen_o_nombre, pos)` | Dibuja una imagen suelta (sin usar Actor) en `pos`. |
| `screen.draw.text(texto, ...)` | Dibuja texto. Ej: `screen.draw.text("Hola", center=(WIDTH/2, HEIGHT/2))`. |
| `screen.draw.textbox(texto, rect, ...)` | Dibuja texto ajustado a una caja. |
| `screen.draw.line(inicio, fin, color, width=1)` | Dibuja una línea. |
| `screen.draw.circle(pos, radio, color, width=1)` | Dibuja el contorno de un círculo. |
| `screen.draw.filled_circle(pos, radio, color)` | Dibuja un círculo relleno. |
| `screen.draw.rect(rect, color, width=1)` | Dibuja el contorno de un rectángulo (`Rect`/`ZRect`). |
| `screen.draw.filled_rect(rect, color)` | Dibuja un rectángulo relleno. |
| `screen.draw.polygon(puntos, color)` | Contorno de un polígono. |
| `screen.draw.filled_polygon(puntos, color)` | Polígono relleno. |
| `screen.width`, `screen.height` | Tamaño actual de la pantalla. |
| `screen.bounds()` | Un `ZRect` con los límites de la pantalla. |

Colores: tuplas `(r, g, b)` o `(r, g, b, a)`, o nombres de pygame (`"white"`, `"cornflowerblue"`, etc).

---

## `keyboard` — teclado

```python
if keyboard.left:
    player.x -= 5
if keyboard.space:
    jump()
```

- Cualquier tecla se consulta como atributo en **minúsculas**: `keyboard.left`, `keyboard.right`, `keyboard.up`, `keyboard.down`, `keyboard.space`, `keyboard.a`...`keyboard.z`, `keyboard.k_0`...`keyboard.k_9`, `keyboard.enter` (alias de `return`), `keyboard.lshift`, `keyboard.rshift`, `keyboard.lctrl`, `keyboard.escape`, etc.
- Devuelve `True`/`False` según si está presionada en ese instante.
- Lista completa de nombres válidos: cualquier constante `pygame.K_*` sin el prefijo `K_` (en minúscula).

---

## `mouse`, `keys`, `keymods` — constantes con nombre

- `mouse.LEFT`, `mouse.MIDDLE`, `mouse.RIGHT`, `mouse.WHEEL_UP`, `mouse.WHEEL_DOWN` — se usan en `on_mouse_down(pos, button)`: `if button == mouse.LEFT:`.
- `keys.<NOMBRE>` — el mismo valor que recibe `on_key_down(key)`, ej. `if key == keys.SPACE:`.
- `keymods.<NOMBRE>` — modificadores (`CTRL`, `SHIFT`, `ALT`...), rara vez necesarios a este nivel.

---

## `images` y `sounds` — cargar recursos a mano

```python
img = images.nave              # carga images/nave.png (con caché)
sonido = sounds.explosion      # carga sounds/explosion.wav u .ogg
sonido.play()
```

- `images.<nombre>`: accede a `images/<nombre>.<ext>` (extensiones válidas: `png`, `gif`, `jpg`, `jpeg`, `bmp`, `webp`). Casi nunca hace falta si usas `Actor`.
- `sounds.<nombre>`: accede a `sounds/<nombre>.<ext>` (`wav`, `ogg`, `oga`) y devuelve un sonido reproducible con `.play()`, `.stop()`, `.set_volume(0.0–1.0)`.
- Los nombres de archivo deben ir en **minúsculas**, sin espacios — si no, pgzero tira un error explícito (ver `docs/friendly-errors.md`).

## `music` — música de fondo

| Función | Qué hace |
|---|---|
| `music.play(nombre)` | Reproduce `music/<nombre>.<ext>` en loop (`mp3`, `ogg`, `oga`). |
| `music.play_once(nombre)` | Igual, pero sin repetir. |
| `music.queue(nombre)` | Encola la siguiente canción. |
| `music.stop()` | Detiene la música. |
| `music.pause()` / `music.unpause()` | Pausa/reanuda. |
| `music.fadeout(segundos)` | Baja el volumen y para. |
| `music.set_volume(v)` / `music.get_volume()` | Volumen de `0.0` a `1.0`. |
| `music.is_playing(nombre)` | `True` si esa pista suena y no está pausada. |

---

## `clock` — temporizadores

```python
clock.schedule(explotar, 2.0)              # llama explotar() en 2 segundos
clock.schedule_interval(spawn_enemigo, 1.5)  # llama spawn_enemigo() cada 1.5s
clock.unschedule(explotar)                 # cancela
```

| Función | Qué hace |
|---|---|
| `clock.schedule(callback, delay)` | Llama `callback()` una vez, tras `delay` segundos. |
| `clock.schedule_unique(callback, delay)` | Como `schedule`, pero si ya estaba programado lo reemplaza. |
| `clock.schedule_interval(callback, delay)` | Llama `callback()` cada `delay` segundos, repetidamente. |
| `clock.unschedule(callback)` | Cancela una llamada programada. |
| `clock.each_tick(callback)` | Llama `callback(dt)` en cada cuadro (alternativa a `update`). |

---

## `animate()` — animar un atributo con el tiempo

```python
animate(alien, pos=(400, 300), duration=1.0, tween='accel_decel')
```

- Primer argumento: el objeto a animar (ej. un `Actor`).
- Argumentos con nombre: cada atributo a animar y su valor final (`pos=`, `x=`, `angle=`, `opacity=`...).
- `duration`: segundos que tarda la animación (por defecto `1`).
- `tween`: forma de la curva de animación. Valores válidos: `linear`, `accelerate`, `decelerate`, `accel_decel`, `in_elastic`, `out_elastic`, `in_out_elastic`, `bounce_end`, `bounce_start`, `bounce_start_end`.
- `on_finished`: función a llamar cuando termina.
- Devuelve un objeto con `.running` (bool) y `.stop(complete=True/False)`.

---

## `Rect` / `ZRect` — rectángulos

```python
zona = Rect((10, 10), (100, 50))
if zona.colliderect(alien):
    ...
```

- Mismo comportamiento que `pygame.Rect`, con propiedades de posición (`.topleft`, `.center`, `.top`, `.bottom`...) y `.colliderect(other)`, `.collidepoint(x, y)`.
- `screen.draw.rect(...)` / `filled_rect(...)` esperan uno de estos.

---

## `exit()`

- Termina el juego de forma prolija (espera a que terminen de sonar los sonidos, hasta 1 segundo, y cierra). Úsalo en vez de `sys.exit()` dentro de un sketch.

---

## Convenciones de assets (ver `PLAN.md` §5)

```
mi-juego/
├─ main.py
├─ images/   png, gif, jpg, jpeg, bmp, webp
├─ sounds/   wav, ogg, oga
└─ music/    mp3, ogg, oga
```

- **Todo en minúsculas, sin espacios ni acentos** — pgzero rechaza rutas mal
  capitalizadas incluso en Windows (donde el sistema de archivos no distingue
  mayúsculas) para evitar que el juego se rompa al llevarlo a otra plataforma.
- Las rutas son siempre relativas a esas carpetas: `Actor("nave")` busca
  `images/nave.png` (o las otras extensiones), nunca hace falta escribir
  `"images/nave.png"` entero.

---

## Lo que NO trae pgzero (para no buscarlo)

- No hay `pygame.init()` manual, ni loop de eventos propio — pgzero ya lo maneja (`launcher.py` arranca `PGZeroGame`, ver `PLAN.md` §3.2).
- No hay soporte multi-ventana ni "escenas" — un sketch, una ventana.
- `tone` (síntesis de sonido chiptune) existe en el vendor pero no forma parte de este cheatsheet: es una API avanzada de generación de audio, fuera del nivel de las plantillas de `templates/`.
