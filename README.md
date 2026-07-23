# Temporal Video Analyzer (TVA)

[![CI](https://github.com/tayuLuc/tva/actions/workflows/ci.yml/badge.svg)](https://github.com/tayuLuc/tva/actions/workflows/ci.yml)
[![Release](https://github.com/tayuLuc/tva/actions/workflows/release.yml/badge.svg)](https://github.com/tayuLuc/tva/actions/workflows/release.yml)
[![Crates.io](https://img.shields.io/crates/v/tva-core)](https://crates.io/crates/tva-core)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue)](LICENSE)
[![MSRV](https://img.shields.io/badge/rustc-1.88%2B-lightgrey)](Cargo.toml)

Analyze video files for FPS drops, duplicate frames, screen tears, and frametime jitter. Headless core + CLI + WASM + C FFI.

## Install

```bash
cargo install tva-cli
```

Or download a pre-built binary from [releases](https://github.com/tayuLuc/tva/releases).

## Usage

```bash
# Full analysis → JSON
tva analyze input.mp4 --format json

# Specific metrics
tva analyze input.mp4 --metrics fps,tears,duplicates

# Overlay FPS graph on video
tva overlay input.mp4 -o output.mp4

# Batch processing → CSV
tva analyze ./clips/*.mp4 --format csv > report.csv
```

## Architecture

```
tva-core          core library (traits + pipeline)
├── traits        FrameComparator, Smoother, FrameDecoder
├── adapters      feature-gated (image-compare, dssim, savgol, video-rs)
├── pixel_buffer  RGB Vec<u8> — zero external deps
├── pipeline      generic over traits
└── report        additive serde schema

tva-cli           CLI binary (clap, JSON/CSV output)
tva-ffi           C shared lib (cdylib + staticlib)
tva-wasm          WASM bindings (wasm-bindgen)
tva-gui           Tauri desktop (separate repo)
tva-web           GitHub Pages demo (separate repo)
```

## Features

| Feature | Flag | Description |
|---------|------|-------------|
| SSIM/MSSIM/Hybrid compare | `compare-image` | image-compare adapter (default) |
| Multi-core DSSIM | `compare-dssim` | dssim-core adapter |
| Savitzky-Golay smoothing | `smooth-savgol` | staged-sg-filter, SIMD |
| FFmpeg decode | `decode-ffmpeg` | video-rs adapter |
| FFT resolution detect | `fft` | rustfft 2D FFT |

## Components

| Repo | Description |
|------|-------------|
| [tva](https://github.com/tayuLuc/tva) | Core + CLI + FFI + WASM (this repo) |
| [tva-gui](https://github.com/tayuLuc/tva-gui) | Tauri desktop GUI |
| [tva-web](https://github.com/tayuLuc/tva-web) | GitHub Pages demo |

## License

MIT — see [LICENSE](LICENSE).
