# Sketch minimo para probar runtime/launcher.py (ver PLAN.md §11, Fase 0).
# Un Actor que se mueve con las flechas. Se cierra solo tras unos frames
# cuando corre desde smoke.py (ver ARCADEZERO_AUTOCLOSE_FRAMES).

import os

WIDTH = 400
HEIGHT = 300
TITLE = "ArcadeZero — sketch de prueba"

alien = Actor("alien", (WIDTH // 2, HEIGHT // 2))
speed = 120
_frame_count = 0

# Si ARCADEZERO_CRASH_AFTER=N esta seteada, revienta a proposito en el
# frame N para probar el excepthook de punta a punta (ver README del spike).
_crash_after = os.environ.get("ARCADEZERO_CRASH_AFTER")
_autoclose_after = os.environ.get("ARCADEZERO_AUTOCLOSE_FRAMES")


def update(dt):
    global _frame_count
    _frame_count += 1
    print(f"frame {_frame_count}")

    if keyboard.left:
        alien.x -= speed * dt
    if keyboard.right:
        alien.x += speed * dt
    if keyboard.up:
        alien.y -= speed * dt
    if keyboard.down:
        alien.y += speed * dt

    if _crash_after and _frame_count == int(_crash_after):
        raise RuntimeError("crash de prueba a proposito (ARCADEZERO_CRASH_AFTER)")

    if _autoclose_after and _frame_count >= int(_autoclose_after):
        exit()


def draw():
    screen.fill((30, 30, 40))
    alien.draw()
