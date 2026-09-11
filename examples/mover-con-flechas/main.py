# Mover con flechas / Move with arrow keys
#
# ES: Este es el juego que se abre la primera vez que usas ArcadeZero.
#     Pulsa Jugar (o F5) y mueve al alien con las flechas del teclado.
#     Despues cambia algo de este codigo (por ejemplo "speed") y vuelve
#     a jugar para ver el cambio.
# EN: This is the game that opens the first time you use ArcadeZero.
#     Press Play (or F5) and move the alien with the arrow keys. Then
#     change something in this code (like "speed") and play again to
#     see your change.

WIDTH = 480
HEIGHT = 360
TITLE = "Mueve al alien"

alien = Actor("alien", (WIDTH // 2, HEIGHT // 2))
speed = 200


def update(dt):
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
