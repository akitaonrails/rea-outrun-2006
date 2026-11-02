#!/bin/sh
# Builds the Linux AppImage: the launcher with its bundled components,
# or-play and a portable Wine.
#
#   packaging/appimage/build.sh <package-folder> <output-folder>
#
# <package-folder> is an unpacked OutRunLauncher zip: OutRunLauncher.exe
# and the folders it reads beside itself (tweaks, vc, linux, soundtrack).
# Downloads (checked by SHA-256) are kept in $OR_CACHE
# (default ~/.cache/or-package, shared with package.sh).
set -eu
PKG=$(readlink -f "$1"); OUT=$(readlink -f "$2")
HERE=$(dirname "$(readlink -f "$0")")
CACHE=${OR_CACHE:-$HOME/.cache/or-package}
PINS=$(dirname "$HERE")
. "$PINS/pins.sh"

# Wine (packaging/wine.txt), Kron4ek's portable WoW64 build, so the 32-bit
# game needs no 32-bit system libraries; appimagetool packs the AppDir
# with a pinned, dated runtime (otherwise it downloads the latest one).
WINE=$(fetch wine)
TOOL=$(fetch appimagetool)
RUNTIME=$(fetch appimage-runtime)
chmod +x "$TOOL"

APPDIR=$(mktemp -d)/OutRunLauncher.AppDir
trap 'rm -rf "$(dirname "$APPDIR")"' EXIT HUP INT TERM
mkdir -p "$APPDIR/usr/lib/or" "$APPDIR/usr/lib/wine"
tar -xJf "$WINE" -C "$APPDIR/usr/lib/wine" --strip-components=1
# Development files aren't needed to run.
rm -rf "$APPDIR/usr/lib/wine/include" "$APPDIR/usr/lib/wine/share/man"
for tool in function_grep.pl widl winebuild winecpp winedump wineg++ winegcc winemaker wmc wrc msidb; do
  rm -f "$APPDIR/usr/lib/wine/bin/$tool"
done
# The launcher reads its components from the folders beside it.
cp "$PKG/OutRunLauncher.exe" "$APPDIR/usr/lib/or/"
for dir in tweaks vc linux soundtrack; do
  if [ -d "$PKG/$dir" ]; then cp -r "$PKG/$dir" "$APPDIR/usr/lib/or/$dir"; fi
done
mv "$APPDIR/usr/lib/or/linux/or-play" "$APPDIR/usr/lib/or/linux/rea-play.sh" "$APPDIR/usr/lib/or/"
mkdir -p "$APPDIR/usr/share/doc/wine" "$APPDIR/usr/share/doc/or/linux"
cp "$HERE"/wine/* "$APPDIR/usr/share/doc/wine/"
# Our licences, the third-party list and the guides, as in the zip; the
# Tweaks build's licence and source notes stay in usr/lib/or/tweaks.
cp "$PKG/README.md" "$PKG/THIRD_PARTY.md" "$PKG/LICENSE-MIT" "$PKG/LICENSE-APACHE" \
  "$APPDIR/usr/share/doc/or/"
cp "$PKG/linux/README.md" "$APPDIR/usr/share/doc/or/linux/"
cp "$HERE/AppRun" "$APPDIR/AppRun"
cp "$HERE/outrun-launcher.desktop" "$HERE/outrun-launcher.png" "$APPDIR/"
chmod +x "$APPDIR/AppRun" "$APPDIR/usr/lib/or/or-play"
mkdir -p "$OUT"
ARCH=x86_64 "$TOOL" --appimage-extract-and-run --no-appstream \
  --runtime-file "$RUNTIME" "$APPDIR" "$OUT/OutRunLauncher-x86_64.AppImage"
[ -s "$OUT/OutRunLauncher-x86_64.AppImage" ]
echo "built $OUT/OutRunLauncher-x86_64.AppImage"
