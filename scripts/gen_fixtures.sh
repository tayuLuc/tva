#!/usr/bin/env bash
set -euo pipefail
DIR="$(cd "$(dirname "$0")/../crates/tva-core/tests/fixtures/frames" && pwd)"
python3 << PYEOF
import struct, zlib, os
def png(path, w, h, r, g, b):
    ihdr = struct.pack('>IIBBBBB', w, h, 8, 2, 0, 0, 0)
    raw = b''
    for _ in range(h):
        raw += b'\x00' + bytes([r, g, b]) * w
    z = zlib.compress(raw)
    crc = lambda s: struct.pack('>I', zlib.crc32(s) & 0xFFFFFFFF)
    with open(path, 'wb') as f:
        f.write(b'\x89PNG\r\n\x1a\n')
        f.write(struct.pack('>I', 13) + b'IHDR' + ihdr + crc(b'IHDR' + ihdr))
        f.write(struct.pack('>I', len(z)) + b'IDAT' + z + crc(b'IDAT' + z))
        f.write(b'\x00\x00\x00\x00IEND' + crc(b'IEND'))
d = '$DIR'
os.makedirs(d, exist_ok=True)
png(f'{d}/001.png', 64, 64, 200, 50, 50)
png(f'{d}/002.png', 64, 64, 200, 50, 50)
png(f'{d}/003.png', 64, 64, 50, 50, 200)
png(f'{d}/004.png', 64, 64, 50, 200, 50)
png(f'{d}/005.png', 64, 64, 50, 200, 50)
print(f'Generated 5 frames in {d}')
PYEOF
