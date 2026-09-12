# Matriz de pruebas manuales — ArcadeZero

> Checklist manual ejecutable a mano antes de publicar cada release. No hay
> framework E2E automatizado (Playwright/WebDriver) a propósito: la ventana
> del juego es SDL externa al webview, y el costo de mantener ese harness es
> alto para lo que aporta en v1. Ver `PLAN.md` §9-11.
>
> **Cómo usar este archivo:** cada ronda de QA agrega filas a la tabla de
> "Resultados" al final de cada escenario, con fecha, plataforma (versión de
> Windows/macOS/Linux exacta), resultado (pasa/falla) y el issue vinculado si
> falló algo. No se borran rondas viejas — se acumulan para ver tendencia.
>
> **Plataforma prioritaria: Windows**, hasta Fase 5 (§9 PLAN.md). Las
> secciones de macOS/Linux quedan como placeholder de una fila hasta que el
> proyecto llegue ahí (Fase 5+, fuera de alcance actual — ver PLAN.md §11).

## Contexto del bug que originó esta ronda

Release anterior: el instalador NSIS no incluía el runtime de Python embebido
(faltaba `bundle.resources` en `tauri.conf.json`) y el Rust que localiza esa
carpeta usaba una ruta de compilación válida solo en el runner de CI. Síntoma
real: instalación rápida y aparentemente exitosa, pero el botón Jugar no
hacía nada — sin ningún error visible (un `try/catch` en el frontend se
tragaba el fallo). Los tres fixes ya están en `master`. Esta matriz existe
para que ese modo de falla (y sus primos) no vuelva a pasar sin que QA lo
atrape antes del release.

---

## 0. Setup de entorno de prueba

- VM limpia de Windows 10/11 **sin** Visual C++ Redistributable, **sin**
  Python de sistema, **sin** herramientas de desarrollador. Snapshot antes de
  instalar, para poder repetir "instalación limpia" varias veces sin rearmar
  la VM a mano.
- Una segunda VM/cuenta con antivirus de terceros instalado (Avast Free o
  AVG Free bastan, son los más agresivos con instaladores sin firmar).
- Cuenta de usuario de Windows con nombre en unicode (ej. `José 🎮`) para el
  escenario 2.
- Conexión a internet que se pueda cortar a demanda (perfil de red / apagar
  el adaptador, no basta con "modo avión" si la VM está en NAT).

---

## 1. Instalación limpia sin Python ni VC++ Redistributable

**Pasos:**
1. En la VM limpia (snapshot del §0), descargar el `.exe` del release desde
   GitHub Releases.
2. Doble-click, seguir el instalador con las opciones por defecto.
3. Abrir ArcadeZero desde el acceso directo del menú Inicio.
4. Esperar a que cargue el ejemplo del primer arranque (§6.6 PLAN.md).
5. Pulsar **Jugar**.

**Resultado esperado:** el instalador termina sin pedir instalar nada más
(ni VC++ Redist, ni Python). La app abre en menos de 2 segundos. Al pulsar
Jugar se abre una ventana nueva con el sprite de ejemplo moviéndose con las
flechas, en menos de 1-2 segundos. Ningún diálogo de "falta un DLL" ni de
Windows pidiendo instalar un runtime.

**Qué revisar si falla:** que el runtime completo (`python.exe`, DLLs,
`pygame-ce`, `pgzero` vendorizado) esté físicamente en la carpeta de
instalación (`%LOCALAPPDATA%\Programs\ArcadeZero\` o donde apunte el NSIS) —
este es exactamente el bug que ya arreglamos, así que es el candado #1 a
verificar en cada release nuevo.

**Severidad si falla:** Crítica (bloquea el 100% de usuarios sin Python
preinstalado — que es el 100% del público objetivo).

---

## 2. Ruta de instalación con espacios, tildes o usuario unicode/emoji

**Pasos:**
1. Instalar en una cuenta de Windows cuyo nombre de usuario tenga tildes,
   espacios o un emoji (ej. `C:\Users\José 🎮\AppData\Local\Programs\...`).
2. Alternativamente, si el instalador NSIS permite elegir carpeta, instalar
   a mano en `C:\Program Files\Arcade Zero (v1)\`.
3. Crear un sketch nuevo, importar un asset con nombre en español con tilde
   (ej. `avión.png`), correr el juego.

**Resultado esperado:** todo funciona igual que en el escenario 1. Ninguna
ruta con espacios/unicode rompe el spawn del subproceso `launcher.py` ni la
resolución de `images/`, `sounds/`, `music/`.

**Qué revisar si falla:** cómo Rust arma el `Command` para spawnear Python
(comillas, encoding de argumentos en Windows es UTF-16, no UTF-8 crudo) y
cómo `launcher.py` hace `chdir` a la carpeta del sketch.

**Severidad si falla:** Alta (las cuentas familiares de Windows con
nombre real de la persona, con tilde o emoji, son comunes en el público de
10-17 años).

---

## 3. Falsos positivos de antivirus (R3 del PLAN)

**Pasos:**
1. En una VM con Windows Defender solo (default), descargar e instalar.
   Observar si SmartScreen bloquea ("Windows protegió su PC").
2. En una VM con Avast Free o AVG Free instalado y actualizado, repetir la
   descarga e instalación. Observar si pone en cuarentena el instalador o
   el `python.exe` embebido después de instalado.
3. Si el AV pone algo en cuarentena, intentar ejecutar la app igual y ver
   qué pasa (falla silenciosa vs. mensaje claro).

**Resultado esperado (realista, instalador sin firmar por decisión de
Fase 5 recortada, ver PLAN.md §11):** SmartScreen probablemente muestre "Más
información" → "Ejecutar de todas formas" — aceptable, documentado. Un AV de
terceros **no debería** poner en cuarentena `python.exe` embebido después de
instalado sin que el usuario pueda desbloquearlo con un click. Si lo hace,
la app debe mostrar un error entendible al pulsar Jugar (no quedarse muda),
y el README/release notes deben tener un párrafo de "si tu antivirus
bloquea la app" con la excepción a agregar.

**Qué revisar si falla:** que este sea justo el mismo modo de falla que el
bug original (Jugar no hace nada, sin error) pero causado por el AV borrando
`python.exe` en vez de por el instalador. El fix del `try/catch` faltante en
el frontend debería, como mínimo, mostrar "no se encontró el runtime" en vez
de silencio — confirmar que ese mensaje aparece también en este caso.

**Severidad si falla silenciosamente:** Alta. Si falla con mensaje claro:
Baja/aceptada (documentar y seguir, ver PLAN.md §9/§10 R3).

---

## 4. Sin conexión a internet

### 4a. Primer arranque completamente offline

**Pasos:** desconectar la red de la VM antes de abrir la app por primera
vez. Abrir ArcadeZero, crear un sketch, correr un juego.

**Resultado esperado:** todo funciona igual que online. El chequeo de
updates (`tauri-plugin-updater`) falla silenciosamente en segundo plano (sin
diálogo bloqueante, sin banner de error alarmante) — como mucho, ausencia
del banner de "hay una actualización".

### 4b. Corte de red a mitad de una descarga de update

**Pasos:** con una versión vieja instalada, provocar que aparezca el banner
de actualización disponible, iniciar la descarga, y cortar la red a mitad
de camino (desactivar el adaptador de red).

**Resultado esperado:** la app no crashea ni queda en un estado roto (ej.
instalador a medio escribir bloqueando el próximo intento). Debe poder
reintentar la actualización más tarde sin reinstalar desde cero. El editor
y Jugar/Detener del sketch actual siguen funcionando mientras tanto (la
descarga de update nunca debe bloquear el hilo principal ni el juego).

**Severidad si falla:** Media (4a), Media-Alta (4b: si deja el binario
corrupto o bloquea el próximo intento de update).

---

## 5. Doble-click rápido en Jugar / Detener

**Pasos:**
1. Abrir un sketch que corre bien. Hacer doble-click rápido y repetido en
   **Jugar** (5-10 veces en 2 segundos).
2. Con el juego corriendo, hacer doble-click rápido y repetido en
   **Detener**.
3. Alternar Jugar/Detener/Jugar/Detener sin pausas, 10 veces seguidas.
4. Cerrar la ventana del juego con la X del sistema operativo mientras el
   IDE cree que sigue corriendo, y luego pulsar Jugar de nuevo.

**Resultado esperado:** nunca se abren dos ventanas de juego simultáneas
para el mismo sketch (o si se permite multi-instancia, es intencional y
documentado, no un bug de doble-spawn). El botón Detener nunca crashea la
app ni dispara error si ya no hay proceso corriendo (proceso zombie o ya
muerto se maneja con gracia). Cerrar la ventana del juego con la X del SO
actualiza el estado del IDE (`run_exit`) y el botón Jugar vuelve a estar
disponible sin tener que reiniciar el IDE.

**Qué revisar si falla:** condiciones de carrera en `RunController` (§3.1,
§10 R4 PLAN.md) — el spawn/kill debe ser idempotente y el estado
"corriendo/detenido" debe reflejar la realidad del proceso hijo, no solo la
última acción del usuario.

**Severidad si falla:** Media-Alta (puede dejar el IDE en un estado
confuso donde el chico cree que el juego no corre pero sí, o viceversa).

---

## 6. Nombres de proyecto inválidos / rutas sin permiso

**Pasos:**
1. Crear un proyecto nuevo con un nombre que Windows prohíbe en nombres de
   archivo/carpeta: `CON`, `PRN`, `AUX`, `NUL`, o con caracteres `<>:"/\|?*`.
2. Crear un proyecto con el mismo nombre de uno que ya existe en la carpeta
   de sketches configurada.
3. Navegar manualmente (si el diálogo de "Nuevo"/"Abrir" lo permite) hasta
   `C:\Program Files\` y tratar de crear el proyecto ahí (carpeta sin
   permiso de escritura para un usuario estándar).
4. Crear un proyecto con un nombre extremadamente largo (250+ caracteres),
   acercándose al límite de ruta de Windows (260 caracteres sin
   `\\?\` prefix).

**Resultado esperado:** en todos los casos, un mensaje de error claro en
español simple ("Ese nombre no se puede usar en Windows", "Ya existe un
proyecto con ese nombre", "No tenés permiso para crear archivos ahí" —
lenguaje del chico, no `Error: EACCES`/`ERROR_ACCESS_DENIED`). Nunca un
crash del IDE, nunca una carpeta a medio crear que después confunda al
"Abrir" (carpeta sin `main.py` pero con subcarpetas huérfanas).

**Severidad si falla:** Media (nombre inválido/Program Files), Alta si deja
al IDE en estado inconsistente o crashea.

---

## 7. `main.py` roto o mal comportado

**Pasos — probar cada uno como un sketch separado:**
1. `main.py` con error de sintaxis obvio (falta `:` en un `if`).
2. `main.py` con `while True: pass` sin ningún `import pgzero` (bucle
   infinito de CPU puro, sin ventana).
3. `main.py` que hace `import pygame; pygame.init()` y abre su propia
   ventana con un loop `while True` que nunca llama a `pygame.quit()` ni
   revisa eventos de cierre (ventana pygame "zombie" que no responde a la X).
4. `main.py` vacío (0 bytes).
5. `main.py` que solo tiene `print("hola")` sin ningún hook de pgzero
   (`draw`/`update`).

Para cada uno: pulsar Jugar, esperar, luego pulsar Detener (o cerrar la
ventana del juego a mano si el caso 3 no responde).

**Resultado esperado:**
- Caso 1: tarjeta de error amigable con la línea del error, sin abrir
  ninguna ventana de juego (ya cubierto por Fase 3, confirmar que sigue
  funcionando en el instalador real, no solo en dev).
- Caso 2 y 3 (bucles infinitos/ventana zombie): el botón **Detener** debe
  matar el proceso limpio (kill forzado del subproceso, no solo pedirle que
  cierre) en menos de 2-3 segundos, sin que el IDE se congele mientras tanto
  (§10 R4/§6.3 PLAN.md: "el IDE nunca se congela mientras el juego corre").
  Esto es el caso más importante de este bloque: un chico que mete un loop
  infinito por accidente (muy común) no debe tener que matar el proceso
  desde el Administrador de tareas.
- Caso 4 y 5: mensaje claro de "tu archivo no tiene código de pgzero" o
  similar, no un traceback críptico ni un crash silencioso.

**Severidad si falla el Detener en casos 2/3:** Crítica (un chico de 10 años
no sabe abrir el Administrador de Tareas; si Detener no mata el proceso,
tiene que reiniciar la PC o pedir ayuda a un adulto).

---

## 8. Assets problemáticos

**Pasos:**
1. Importar una imagen corrupta (renombrar un `.txt` a `.png`, o truncar un
   PNG real a la mitad con un editor hex).
2. Importar un sonido corrupto de la misma forma (`.wav` truncado).
3. Importar un archivo con nombre extremadamente largo (100+ caracteres) y
   con caracteres especiales/emoji en el nombre original, antes de que el
   renombrado automático (§5 PLAN.md) lo normalice.
4. Copiar manualmente (fuera del panel de assets, directo en Explorador de
   Windows) 2000+ archivos pequeños dentro de `images/` de un sketch, y
   luego abrir ese sketch en ArcadeZero.
5. Referenciar en `main.py` una imagen corrupta con `Actor("nombre")` y
   correr el juego.

**Resultado esperado:** importar un archivo corrupto no crashea el IDE (el
renombrado/copia es solo manejo de bytes, no necesita decodificar la
imagen). Correr un juego que usa una imagen corrupta produce un error
amigable tipo pygame ("No se pudo abrir la imagen...", ya hay un caso
similar en el catálogo de `docs/friendly-errors.md`), no un crash mudo. El
panel de assets con 2000+ archivos no congela la UI al abrir el sketch (el
watcher con `notify` no debe iterar de forma bloqueante en el hilo
principal) — puede ser lento, pero no debe trabarse indefinidamente.

**Severidad si falla:** Media (assets corruptos son un catch ya cubierto
en parte por Fase 3), Media-Alta (freeze de UI con carpetas grandes, porque
"miles de archivos" ocurre fácil si un chico arrastra una carpeta de
sprites descargada de internet completa).

---

## 9. Desinstalar y reinstalar

**Pasos:**
1. Con ArcadeZero instalado y con al menos un sketch guardado y ajustes
   cambiados (tema, idioma), desinstalar desde "Aplicaciones y
   características" de Windows.
2. Revisar a mano qué queda en `%APPDATA%\` y `%LOCALAPPDATA%\` (config dir
   TOML, cualquier caché, el propio directorio de instalación).
3. Reinstalar la **misma** versión. Verificar que abre limpio, sin arrastrar
   configuración corrupta de la instalación anterior si el desinstalador no
   la limpió (o que si la mantiene, es intencional: los sketches del chico
   no se deben perder por reinstalar).
4. Instalar una versión vieja (release anterior), confirmar que el updater
   ofrece actualizar a la más nueva, actualizar, y verificar que el updater
   de la app resultante sigue apuntando al mismo endpoint/repo (no a una
   URL vieja hardcodeada de un build de prueba).

**Resultado esperado:** el desinstalador de NSIS quita el binario y accesos
directos. Es aceptable (y preferible) que **no** borre la carpeta de
sketches del usuario ni el config TOML sin preguntar — perder el trabajo del
chico al desinstalar sería peor que dejar basura. Documentar cuál es el
comportamiento real observado (si no pregunta y borra igual, o si nunca
borra nada, dejando basura acumulable). El updater después de reinstalar
sigue funcionando y apunta al repo real de GitHub Releases (confirmar que
`tauri.conf.json` ya no tiene el placeholder `TODO_OWNER/TODO_REPO`
mencionado en PLAN.md §11 Fase 5 antes de este release).

**Severidad si falla:** Media (basura en disco), Alta si borra los sketches
del usuario sin avisar, Alta si el updater quedó con endpoint roto.

---

## 10. Dos instancias simultáneas

**Pasos:**
1. Abrir ArcadeZero. Sin cerrarlo, abrir el acceso directo de nuevo (o
   doble-click al `.exe` de instalación otra vez).
2. Si abre una segunda ventana: abrir el mismo sketch en ambas instancias,
   editar y guardar desde la instancia A, luego editar y guardar desde la
   instancia B, y ver qué pasa (¿se pisan los cambios sin avisar?).
3. Pulsar Jugar en ambas instancias para el mismo sketch al mismo tiempo.

**Resultado esperado (mínimo aceptable para v1):** no debe crashear ninguna
de las dos instancias. Es aceptable que no haya single-instance lock (no es
un requisito documentado en PLAN.md), pero si dos instancias escriben el
mismo `main.py` sin coordinarse, el chico puede perder cambios — como
mínimo no debe corromper el archivo (dejarlo a medio escribir). Ideal:
detectar el guardado externo y avisar "el archivo cambió, ¿querés
recargar?" — si no está implementado, documentarlo como gap conocido, no
como bug bloqueante de este release.

**Severidad si falla:** Baja-Media (pérdida de cambios por sobrescritura es
molesta pero no bloquea el caso de uso principal de un chico con una sola
ventana abierta); Alta si corrompe el archivo en disco (bytes mezclados) o
si crashea la app.

---

## Resultados

Formato de fila: `Fecha | Plataforma (versión exacta) | Escenario | Resultado | Issue`.
Agregar una fila por cada corrida de un escenario. No borrar filas viejas.

| Fecha | Plataforma | Escenario | Resultado | Issue |
|---|---|---|---|---|
| _(pendiente — primera ronda posterior al fix de `bundle.resources`)_ | | | | |

---

## Resumen de riesgo para el próximo release

Los **3 riesgos más probables** de que el próximo release también falle en
producción, en orden:

1. **El fix del runtime embebido no cubre todas las formas en que se
   resuelve la ruta.** El bug original era justo esto (ruta de compilación
   hardcodeada de CI). Si `runtime.rs` todavía tiene algún `cfg!(debug)` o
   ruta relativa que funcione en la máquina del desarrollador pero no en la
   del usuario, el síntoma va a ser idéntico: instala rápido, Jugar no hace
   nada. El escenario 1 de esta matriz es el candado obligatorio antes de
   cualquier release — probarlo en una VM limpia, nunca en la máquina de
   desarrollo.
2. **Errores silenciosos que quedan en otro `try/catch` sin manejar.** Ya
   apareció uno en el frontend; es probable que haya hermanos (en Rust, al
   spawnear el proceso; en el updater; al leer settings corruptos). El
   patrón "falla y no dice nada" es el más peligroso porque un chico de 10
   años no reporta un bug con logs, solo dice "no funciona" y abandona.
3. **Falsos positivos de antivirus sobre el instalador o `python.exe`
   sin firmar** (Fase 5 se recortó a propósito sin firma Authenticode, ver
   PLAN.md §11). Esto no es un bug de código pero produce el mismo síntoma
   de usuario ("lo instalé y no anda") y es el que menos controla el
   equipo — la mitigación es documentación clara en el README/release
   notes, no código.
