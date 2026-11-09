# Solar System Shaders

Renderer de software escrito completamente en Rust. Cada cuerpo celeste se dibuja píxel por píxel y su apariencia se genera con shaders procedurales: no se utilizan texturas ni materiales.

## Ejecutar

```powershell
cargo run
```

Para generar o regenerar todas las capturas que aparecen abajo:

```powershell
cargo run -- --all-screenshots
```

## Controles

- `1` Sol
- `2` Venus
- `3` Neptuno
- `4` Tierra + Luna
- `5` Saturno + anillos
- `6` Urano
- `Left/Right Arrow` cambiar de cuerpo celeste
- `S` guardar la captura del cuerpo actual
- `Esc` cerrar

## Shaders y efectos

Todos los patrones visibles se calculan a partir de la posición, la normal, la iluminación y funciones matemáticas (`sin`, ruido procedural, `smoothstep`, gradientes y efectos de borde).

- **Sol:** gradiente de temperatura, células de convección, filamentos de plasma y brillo emisivo.
- **Venus:** gradiente dorado, bandas de nubes, streaks atmosféricos y borde cálido.
- **Neptuno:** base azul, bandas, variación de nubes y borde atmosférico frío.
- **Tierra:** océanos, continentes, variación de terreno, nubes y atmósfera.
- **Saturno:** bandas doradas, turbulencia, zona ecuatorial, borde cálido y anillos procedurales.
- **Urano:** gradiente cian, bandas sutiles, bruma polar y borde frío.
- **Luna:** segunda esfera junto a la Tierra con gradiente gris, mares lunares y cráteres procedurales.

Saturno se renderiza con un modelo de anillos generado por código y una prueba de profundidad. La Tierra se renderiza junto a una segunda esfera para demostrar la luna, también sin texturas.

## Evidencia de la rúbrica

| Criterio | Evidencia en el proyecto |
|---|---|
| 3 cuerpos requeridos | Sol, Tierra y Saturno |
| Complejidad de shaders | Cada cuerpo tiene al menos 4 capas que afectan el color; la Tierra tiene 5 |
| 3 planetas extra | Venus, Neptuno y Urano |
| Sistema de anillos | Anillos procedurales de Saturno con bandas y depth test |
| Luna | `MoonShader` renderizado junto a la Tierra |

La implementación cubre todos los criterios técnicos de la rúbrica; la parte de creatividad queda sujeta a la evaluación subjetiva del diseño.

## Capturas renderizadas

### Sol, Venus y Neptuno

| Sol | Venus | Neptuno |
|---|---|---|
| ![Sol renderizado](screenshots/sun.png) | ![Venus renderizado](screenshots/venus.png) | ![Neptuno renderizado](screenshots/neptune.png) |

### Tierra con luna, Saturno con anillos y Urano

| Tierra + Luna | Saturno + anillos | Urano |
|---|---|---|
| ![Tierra con luna renderizada](screenshots/earth.png) | ![Saturno con anillos renderizado](screenshots/saturn.png) | ![Urano renderizado](screenshots/uranus.png) |
