#!/bin/sh
# packaging/build-dlls.sh: builds the SDK's wheel layer DLL (`rea_dinput.dll`)
# for tweaks/, into target/i686-pc-windows-gnu/dll/.
set -eu
ROOT=$(cd "$(dirname "$0")/.." && pwd)
cd "$ROOT"
exec cargo build --locked --profile dll --target "${RUST_TARGET:-i686-pc-windows-gnu}" --lib \
  -p or-launcher -p rea-launcher-dinput \
  --features or-launcher/layer-dlls "$@"
