# Mi primer sprite / My first sprite
#
# ES: Un "Actor" es un personaje que se puede mover y dibujar solo.
#     Este alien se mueve con las flechas del teclado. Prueba a cambiar
#     "alien" por otro nombre de imagen en la carpeta images/, o cambia
#     "speed" para que vaya mas rapido o mas lento.
# EN: An "Actor" is a character you can move and draw easily.
#     This alien moves with the arrow keys. Try changing "alien" for
#     another image name in the images/ folder, or change "speed" to
#     make it move faster or slower.

WIDTH = 480
HEIGHT = 360
TITLE = "Mi primer sprite"

# Actor(nombre_de_imagen, posicion_inicial)
# La imagen debe existir en images/alien.png (minusculas, sin espacios).
alien = Actor("alien", (WIDTH // 2, HEIGHT // 2))
speed = 200  # pixeles por segundo / pixels per second


def update(dt):
    # ES: "dt" es el tiempo (en segundos) desde el cuadro anterior. Se usa
    #     para que el movimiento sea igual de rapido en cualquier compu.
    # EN: "dt" is the time (in seconds) since the last frame. It's used so
    #     movement feels the same speed on any computer.
    if keyboard.left:
        alien.x -= speed * dt
    if keyboard.right:
        alien.x += speed * dt
    if keyboard.up:
        alien.y -= speed * dt
    if keyboard.down:
        alien.y += speed * dt


def draw():
    screen.fill((30, 30, 40))
    alien.draw()
