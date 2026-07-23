# tva-web

WASM demo for GitHub Pages — analyze short video clips in the browser.

## How it works

1. Browser decodes video via **WebCodecs** (H.264/VP9/AV1)
2. Raw `ImageData` frames passed to **WASM** (tva-core via wasm-pack)
3. **Chart.js** renders FPS graph, timeline, and metrics

## Limits

- Clips ≤30s (memory + decode time)
- Browser-supported codecs only (H.264 baseline, VP9, AV1)

## Deploy

```bash
wasm-pack build crates/tva-wasm --target web
cp -r crates/tva-wasm/pkg/ web/
# push to gh-pages branch
```
