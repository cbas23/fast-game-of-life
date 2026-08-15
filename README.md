# fast-game-of-life

![fast-game-of-life icon](assets/icon.svg)

A Rust implementation of Conway's Game of Life focused on fast sparse-world updates.

The simulation stores the world as 32x32 chunks. Each row is packed into a `u32`, and each generation is computed with bitwise neighbor counting instead of per-cell object storage. Terminal and `minifb` renderers are included as examples.

## Features

- Sparse world storage with `HashMap`-backed chunks
- 32x32 chunk representation using bit-packed rows
- Bitwise generation updates across chunk boundaries
- Simple pattern loading from strings using `*` for live cells
- WebAssembly bindings for browser and JavaScript projects
- `minifb` window rendering with a decoupled simulation/render loop

## Requirements

- Rust toolchain with Cargo
- A desktop environment supported by `minifb` for the graphical example
- `wasm-pack` and the `wasm32-unknown-unknown` Rust target for WebAssembly builds

## Examples

Run the terminal animation:

```sh
cargo run --release --example cli
```

Run the graphical desktop viewer:

```sh
cargo run --release --example desktop
```

Press `Escape` to close the graphical viewer.

## WebAssembly

Install the WebAssembly target and build tool once:

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-pack
```

Build a browser-ready ES module:

```sh
wasm-pack build wasm --release --target web
```

The generated JavaScript, TypeScript declarations, and WebAssembly binary are
written to `wasm/pkg/`. They can be copied into a web project or installed as a
local package.

```js
import init, { WasmWorld } from "./wasm/pkg/fast_game_of_life_wasm.js";

await init();

const world = new WasmWorld();
world.load_pattern(0, 0, ".*.\n..*\n***");
world.step();

// Flat Float64Array: [x0, y0, x1, y1, ...]
const cells = world.live_cells();
```

The bindings use JavaScript numbers for coordinates and validate that they are
finite integers within the range supported by the internal chunk keys.

## Development

Check that the project builds:

```sh
cargo check
```

Check all library and example targets:

```sh
cargo check --all-targets
```

## Project Layout

```text
src/
  lib.rs    Public library exports
  world.rs  Sparse world storage and generation computation
  chunk.rs  32x32 bit-packed chunk representation
examples/
  cli.rs      Animated terminal renderer
  desktop.rs  Graphical minifb renderer
wasm/
  src/lib.rs  JavaScript-facing WebAssembly wrapper
lex/        Reference Game of Life lexicon files
```

## Pattern Format

Patterns are plain text grids loaded with `World::load_pattern`:

- `*` means a live cell
- Any other character means a dead cell
- The origin is expressed in global cell coordinates

Example:

```text
***
..*
.*.
```
