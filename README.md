# demostrante

Un programa chico en Rust que busca un pedazo mejor de su propia fuente y **sólo escribe un hijo si puede probar que el cambio es mejor**.

No es un modelo de lenguaje y no se copia solo. Le señalás una carpeta; sólo escribe ahí.

Es hermano de [mejorante](https://github.com/PascualMacana/mejorante). Darwin sigue proponiendo (mutantes al azar, puntaje congelado). Gödel acepta: el hijo nace porque un certificado cierra, no porque el buscador tuvo suerte.

Lo que evoluciona es una expresión matemática chica, el **cerebro**. El programa intenta encajar esta función:

```
f(x) = x² + 3x + 5
```

en `x = -5 … 5`. El puntaje es la suma de errores al cuadrado (**sse**). Más bajo es mejor. Cero es un encaje perfecto.

Una prueba es esa misma cuenta, escrita:

```
padre     cerebro  0                              sse  4323
  │ evolve (búsqueda)
  │ prove  (reevalúa los 11 puntos)
  ▼
hijo      cerebro  (+ (+ (* 3 x) (* x x)) 5)      sse  0
          padre  0
          prueba sse:4323.0000->0.0000
```

Cualquiera puede volver a correr los 11 puntos. Si los números no cierran, el certificado es mentira y no se escribe nada.

![Una célula que sólo sella un hijo cuando la prueba cierra](cell.svg)

Míralo en la terminal. El cuerpo se llena mientras baja el error. Un sello debajo de la célula se tinta sólo si el cerebro actual le ganaría al padre. `dish` usa por defecto la seed 7.

```bash
cargo run -- dish
```

## Cómo correrlo

Hace falta [Rust](https://rustup.rs/).

```bash
cargo build --release
./target/release/demostrante identity
./target/release/demostrante prove
./target/release/demostrante evolve --steps 120 --spawn ./hijo --build
./hijo/target/debug/demostrante identity
./hijo/target/debug/demostrante prove
```

`identity` imprime generación, cerebro, padre, y si la prueba verifica.  
`evolve --spawn ./hijo --build` busca, **se niega a escribir** si no hay mejora, y si hay escribe un hijo que lleva el certificado.

`spawn ./hijo` sin `evolve` copia este genoma y **no** afirma una mejora.

## Comandos

```
demostrante identity              generación, linaje, padre, prueba, puntaje
demostrante eval [x]              cerebro vs la función objetivo
demostrante prove [expr]          verifica la prueba de este individuo, o de un candidato
demostrante evolve                busca un cerebro mejor
                 --steps N      pasos de búsqueda (default 120)
                 --lambda L     mutantes por paso (default 30)
                 --seed S       rng reproducible
                 --spawn <dir>  hijo con el campeón, si la prueba cierra
                 --build        compila a ese hijo
                 --force        pisa un hijo anterior
                 --write        pisa src/main.rs, si la prueba cierra
demostrante dish                  anima una célula y un sello
                 --steps N      pasos de búsqueda (default 120)
                 --lambda L     mutantes por paso (default 30)
                 --seed S       default 7 (la demo fiable)
                 --delay MS     ms por cuadro (default 80)
demostrante spawn <dir>           copia el genoma actual (sin afirmar mejora)
demostrante genome                imprime las fuentes embebidas
```

## Cómo funciona

El cerebro vive en una constante de `src/main.rs`. También el cerebro padre y el string de la prueba.

1. Parsea el cerebro actual a un árbol.
2. Cada paso arma varios mutantes al azar. Se queda con el menor `sse + 0.01 × tamaño`.
3. Después de un encaje perfecto, las identidades algebraicas pueden achicar la expresión.
4. Una prueba de `A → B` vale si, en `x = -5 … 5`, `sse(B) < sse(A)`, o el sse es igual y `B` es más chico.
5. `--spawn` / `--write` reevalúan esa afirmación. Recién ahí parchean `BRAIN`, `PARENT_BRAIN` y `PROOF`.

La función de puntaje no cambia nunca. Esto no es Reina Roja: el mundo se queda quieto. No es la Gödel Machine de Schmidhuber: no hay un demostrador general. El certificado son los 11 puntos.

## Seguridad

- Un hijo por corrida. No hay bucles en segundo plano ni red.
- No escribe sobre el directorio home, `/`, `/usr`, `/etc`, ni el directorio en el que estás parado.
- `--force` sólo borra una carpeta que ya parece un proyecto `demostrante`.
- Una copia (`spawn` sin búsqueda) no pide prueba. Una mejora afirmada sí.

## Relacionados

[replicante](https://github.com/PascualMacana/replicante) se copia.  
[mejorante](https://github.com/PascualMacana/mejorante) se copia y además intenta mejorar, sin pedir prueba.  
[reinante](https://github.com/PascualMacana/reinante) sigue buscando porque la función de puntaje misma se mueve.  
[cruzante](https://github.com/PascualMacana/cruzante) se queda con los cruces del río que todavía eran legales.
