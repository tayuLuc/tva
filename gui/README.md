# tva-gui

Desktop GUI for Temporal Video Analyzer — built with [Tauri](https://tauri.app) + Svelte.

## Structure

```
gui/
├── src-tauri/     # Rust shell (calls tva-core)
├── src/           # Svelte frontend
└── package.json
```

## Features

- Video player with frame-by-frame navigation
- FPS graph overlay on timeline
- Duplicate (red) / tear (yellow) markers
- Frametime histogram
- CSV export

## Dev

```bash
cd gui
npm install
npm run tauri dev
```
