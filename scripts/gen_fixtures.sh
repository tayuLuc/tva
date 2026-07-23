#!/usr/bin/env bash
set -euo pipefail
DIR="$(cd "$(dirname "$0")/../crates/tva-core/tests/fixtures/frames" && pwd)"
mkdir -p "$DIR"
if command -v python3 &>/dev/null; then
python3 << 'PYEOF'
import struct, zlib, os
def p(w,h,r,g,b,pth):
    ih=struct.pack('>IIBBBBB',w,h,8,2,0,0,0)
    raw=b''
    for _ in range(h): raw+=b'\x00'+bytes([r,g,b])*w
    z=zlib.compress(raw)
    crc=lambda s: struct.pack('>I',zlib.crc32(s)&0xFFFFFFFF)
    with open(pth,'wb') as f:
        f.write(b'\x89PNG\r\n\x1a\n')
        f.write(struct.pack('>I',13)+b'IHDR'+ih+crc(b'IHDR'+ih))
        f.write(struct.pack('>I',len(z))+b'IDAT'+z+crc(b'IDAT'+z))
        f.write(b'\x00\x00\x00\x00IEND'+crc(b'IEND'))
d=os.path.join(os.path.dirname(__file__) or '.', '..', 'crates/tva-core/tests/fixtures/frames')
os.makedirs(d, exist_ok=True)
p(64,64,200,50,50,os.path.join(d,'001.png'))
p(64,64,200,50,50,os.path.join(d,'002.png'))
p(64,64,50,50,200,os.path.join(d,'003.png'))
p(64,64,50,200,50,os.path.join(d,'004.png'))
p(64,64,50,200,50,os.path.join(d,'005.png'))
print(f'fixtures in {d}')
PYEOF
else echo "python3 not found, skipping"; fi
