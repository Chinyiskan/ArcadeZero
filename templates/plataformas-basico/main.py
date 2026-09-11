# Plataformas basico / Basic platformer
#
# ES: Un personaje que cae por gravedad, camina con las flechas izquierda
#     y derecha, y salta con la barra espaciadora sobre una plataforma fija.
#     No es un juego de plataformas completo (eso queda para ti): es lo
#     minimo para que veas gravedad + salto + una colision funcionando.
# EN: A character that falls due to gravity, walks with left/right arrows,
#     and jumps with spacebar onto a fixed platform. This is not a full
#     platformer (that part is up to you): it's the minimum to see
#     gravity + jumping + one collision working together.

WIDTH = 480
HEIGHT = 360
TITLE = "Plataformas basico"

GRAVITY = 900          # pixeles/seg^2 que tira hacia abajo / pulls down
JUMP_SPEED = -420       # negativo = hacia arriba / negative = upward
MOVE_SPEED = 180

player = Actor("player", (WIDTH // 2, HEIGHT - 100))
player_vy = 0           # velocidad vertical actual / current vertical speed
on_ground = False

platform = Actor("platform", (WIDTH // 2, HEIGHT - 40))


def update(dt):
    global player_vy, on_ground

    # Movimiento horizontal / horizontal movement
    if keyboard.left:
        player.x -= MOVE_SPEED * dt
    if keyboard.right:
        player.x += MOVE_SPEED * dt

    # Saltar solo si esta en el suelo o la plataforma / jump only if grounded
    if keyboard.space and on_ground:
        player_vy = JUMP_SPEED
        on_ground = False

    # Gravedad: la velocidad vertical crece cada cuadro / gravity each frame
    player_vy += GRAVITY * dt
    player.y += player_vy * dt

    # Colision simple con la plataforma: solo si cae sobre ella
    # Simple collision with the platform: only while falling onto it
    on_ground = False
    if player_vy >= 0 and player.colliderect(platform):
        if player.bottom <= platform.top + 20:
            player.bottom = platform.top
            player_vy = 0
            on_ground = True

    # Si se cae fuera de la pantalla, vuelve arriba / respawn if it falls off
    if player.top > HEIGHT:
        player.pos = (WIDTH // 2, 0)
        player_vy = 0


def draw():
    screen.fill((20, 24, 40))
    platform.draw()
    player.draw()
