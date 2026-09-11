# Nuevo juego con pgzero / New pgzero game
# ArcadeZero llama a estas dos funciones muchas veces por segundo:
#   draw()    -> ES: dibuja lo que se ve en pantalla, cuadro a cuadro.
#                EN: draws what is on screen, frame by frame.
#   update()  -> ES: mueve cosas y revisa el teclado ANTES de dibujar.
#                EN: moves things and checks the keyboard BEFORE drawing.

WIDTH = 800
HEIGHT = 600


def draw():
    screen.clear()
    screen.draw.text("Hola, ArcadeZero!", center=(WIDTH / 2, HEIGHT / 2))


def update():
    pass
