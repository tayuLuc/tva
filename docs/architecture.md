# TVA Architecture

## Layers

```
┌─────────────────────────────────────────────────┐
│                  Interfaces                      │
│  ┌─────┐  ┌──────┐  ┌─────┐  ┌──────────────┐  │
│  │ CLI │  │ Tauri│  │ WASM│  │ C FFI (lib)  │  │
│  └──┬──┘  └──┬───┘  └──┬──┘  └──────┬───────┘  │
│     │        │         │            │           │
├─────┴────────┴─────────┴────────────┴───────────┤
│              tva-core (Rust lib)                 │
│  ┌──────────┐ ┌──────────┐ ┌─────────────────┐  │
│  │ decoder  │ │ analyzer │ │ exporter        │  │
│  │ (FFmpeg) │ │ (diff,   │ │ (JSON, CSV,     │  │
│  │          │ │  CIELAB, │ │  overlay video) │  │
│  │          │ │  tears)  │ │                 │  │
│  └──────────┘ └──────────┘ └─────────────────┘  │
├─────────────────────────────────────────────────┤
│  FFmpeg (libavcodec/libavformat) via ffmpeg-next │
│  HW accel: NVDEC / VAAPI / VideoToolbox         │
└─────────────────────────────────────────────────┘
```

## Data flow

```
[Video file] → FFmpeg → Vec<Frame> → Analyzer → Report → JSON/CSV/Overlay
```

## Key decisions

- **Headless core**: tva-core knows nothing about UI
- **JSON as contract**: AI agent parses stdout, not logs
- **SIMD diff**: pixel loops via Rayon, not OpenCV
- **WASM demo**: WebCodecs → WASM → Chart.js (≤30s clips)
- **HW decode**: NVDEC/VAAPI/VideoToolbox via ffmpeg-next
