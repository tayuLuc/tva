# Temporal Video Analyzer (TVA)

Analyze video files for FPS drops, duplicate frames, screen tears, and frametime jitter.

## Components

| Component | Description |
|-----------|-------------|
| `tva-core` | Rust library: decode, compare, detect, export |
| `tva-cli` | CLI binary for scripts and AI agents |
| `tva-ffi` | C shared library for embedding |
| `tva-wasm` | WASM bindings for browser demo |
| `gui/` | Tauri desktop GUI (Svelte) |
| `web/` | GitHub Pages demo |

## Quick start

```bash
cargo build --release
./target/release/tva analyze input.mp4 --format json
```

## Project structure

```
tva/
├── Cargo.toml          # workspace
├── crates/
│   ├── tva-core/       # core analysis engine
│   ├── tva-cli/        # CLI binary
│   ├── tva-ffi/        # C FFI
│   └── tva-wasm/       # WASM bindings
├── gui/                # Tauri desktop app
├── web/                # GitHub Pages demo
├── tests/              # integration tests
└── docs/               # documentation
```
