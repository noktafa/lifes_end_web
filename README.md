# Life's End - Web Version

A nihilistic virus, destined to destroy all possible life. Now playable in your browser!

**Original:** https://github.com/noktafa/lifes_end

## About

Life's End fuses three classic game mechanics:
- **Asteroids** — inertia-based thrust, drifting through space
- **Snake** — a growing tail that trails behind you  
- **Conway's Game of Life** — the Monad, your enemy, obeying B3/S23 rules

You are the virus. Life is the disease. There is no cure — only you.

## Controls

| Key | Action |
|-----|--------|
| `W` / `↑` | Thrust forward |
| `A` / `D` / `←` / `→` | Rotate |
| `S` / `↓` | Hard brake |
| `Shift` | Boost (burns fuel) |
| `Space` / `Click` | Shoot projectile |
| `Q` | **NUKE** (requires 10+ tail segments) |

## Play Online

[Play Life's End](https://noktafa.github.io/lifes_end_web/)

## Building Locally

### Prerequisites

- [Rust](https://rustup.rs/)
- wasm-bindgen-cli: `cargo install wasm-bindgen-cli`
- wasm32 target: `rustup target add wasm32-unknown-unknown`

### Build

```bash
./build_web.sh
```

Or manually:

```bash
cargo build --release --target wasm32-unknown-unknown
wasm-bindgen --out-dir ./out --target web ./target/wasm32-unknown-unknown/release/lifes_end.wasm
cp -r assets out/
cp index.html out/
```

### Serve

```bash
cd out
python3 -m http.server 8080
```

Then open http://localhost:8080 in your browser.

## Tech Stack

- **Language:** Rust
- **Engine:** Bevy 0.15
- **Target:** WebAssembly (WASM)
- **Rendering:** WebGL2

## License

MIT - Same as the original project.
