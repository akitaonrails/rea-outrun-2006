#!/bin/sh
# Assembles the Windows package folder: OutRunLauncher.exe and the pinned
# third-party components in the folders it reads beside itself.
#
#   packaging/package.sh <output-folder>
#
# Build the launcher first (cargo build --release --target
# i686-pc-windows-gnu). Every download comes from packaging/<name>.txt and
# is checked by SHA-256; they are kept in $OR_CACHE (default
# ~/.cache/or-package). The community HD texture pack is not bundled: it
# is a separate download the launcher links to (docs/HD_TEXTURES.md).
#
# Needs curl, unzip, tar, cabextract and sha256sum.
set -eu
ROOT=$(dirname "$(dirname "$(readlink -f "$0")")")
CACHE=${OR_CACHE:-$HOME/.cache/or-package}
PINS=$ROOT/packaging
. "$PINS/pins.sh"

[ $# -eq 1 ] || { echo "usage: $0 <output-folder>"; exit 2; }
OUT=$1
EXE=$ROOT/target/i686-pc-windows-gnu/release/OutRunLauncher.exe
[ -f "$EXE" ] || { echo "build the launcher first: $EXE is missing"; exit 1; }
for tool in curl unzip tar cabextract sha256sum; do
  command -v "$tool" >/dev/null || { echo "$tool is needed"; exit 1; }
done

rm -rf "$OUT"
mkdir -p "$OUT/tweaks" "$OUT/vc" "$OUT/linux" "$OUT/soundtrack"
WORK=$(mktemp -d); trap 'rm -rf "$WORK"' EXIT

cp "$EXE" "$OUT/"
cp "$ROOT/README.md" "$ROOT/THIRD_PARTY.md" "$ROOT/LICENSE-MIT" "$ROOT/LICENSE-APACHE" "$OUT/"
# or-play and the shared play library it sources (rea-launcher-sdk).
cp "$ROOT/linux/or-play" "$ROOT/linux/README.md" "$ROOT/sdk/rea-play.sh" "$OUT/linux/"
chmod +x "$OUT/linux/or-play"

# OutRun2006Tweaks: the replacement EXE, dinput8.dll, its two INI files,
# and the licence and source notes of the build (docs/TWEAKS.md).
unzip -q -j -o "$(fetch tweaks)" dinput8.dll OR2006C2C.exe \
  OutRun2006Tweaks.ini OutRun2006Tweaks.lods.ini LICENSE.md SOURCE.txt -d "$OUT/tweaks"

# The soundtrack fix and the xdelta3 that applies it.
unzip -q -j -o "$(fetch soundtrack)" 14_Rush_a_Difficulty_1989.ogg.vcdiff \
  xdelta3-3.1.0-i686.exe -d "$OUT/soundtrack"
mv "$OUT/soundtrack/xdelta3-3.1.0-i686.exe" "$OUT/soundtrack/xdelta3.exe"

# DXVK's 32-bit d3d9.dll.
tar -xzf "$(fetch dxvk)" -C "$WORK" --wildcards '*/x32/d3d9.dll'
mv "$WORK"/*/x32/d3d9.dll "$OUT/linux/d3d9.dll"

# The VC++ runtime: the installer is a cab bundle; one inner cab (its name
# changes between versions) holds the DLLs as <name>_x86.
cabextract -q -d "$WORK/vc" "$(fetch vcredist)"
for cab in "$WORK"/vc/a*; do
  if cabextract -l "$cab" 2>/dev/null | grep -q 'vcruntime140.dll_x86'; then
    cabextract -q -d "$WORK/vcdll" -F 'msvcp140.dll_x86' -F 'vcruntime140.dll_x86' \
      -F 'vcomp140.dll_x86' "$cab"
    break
  fi
done
for dll in msvcp140 vcruntime140 vcomp140; do
  [ -f "$WORK/vcdll/$dll.dll_x86" ] || { echo "VC++ runtime: $dll.dll not found"; exit 1; }
  mv "$WORK/vcdll/$dll.dll_x86" "$OUT/vc/$dll.dll"
done

echo "packaged $OUT"
