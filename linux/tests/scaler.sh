#!/bin/sh
# Tests that or-play hands launcher.cfg's scaler to gamescope (-F), for
# every scaler, the race size as both the output and the nested size
# (-W/-H, -w/-h), and the SDL pad settings: sh linux/tests/scaler.sh
set -u
# or-play is sourced the SDK way (shell/README.md "Tests"): REA_FUNCTIONS_ONLY
# loads it without running, REA_PLAY_HERE finds the vendored ../sdk library.
PLAY_DIR=$(dirname "$(dirname "$(readlink -f "$0")")")
unset GAMESCOPE_WAYLAND_DISPLAY DRI_PRIME __NV_PRIME_RENDER_OFFLOAD WINEDLLOVERRIDES
unset SDL_GAMECONTROLLER_ALLOW_BACKGROUND_EVENTS SDL_JOYSTICK_ALLOW_BACKGROUND_EVENTS
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
export T HOME=$T OR_PREFIX=$T/prefix
CFGDIR=$OR_PREFIX/drive_c/users/me/AppData/Local/OutRunLauncher
mkdir -p "$CFGDIR"; : > "$T/OutRunLauncher.exe"
# The wrapper (a program: or-play starts it through setsid): "gamescope is
# installed" unless $T/no-gamescope exists, and records the command it runs.
cat > "$T/fake-run" <<'FAKE'
#!/bin/sh
case "$*" in *"command -v gamescope"*) [ ! -e "$T/no-gamescope" ]; exit ;; esac
echo "$*" > "$T/ran"
FAKE
chmod +x "$T/fake-run"
export OR_RUN=$T/fake-run OR_LAUNCHER=$T/OutRunLauncher.exe
REA_FUNCTIONS_ONLY=1 REA_PLAY_HERE=$PLAY_DIR . "$PLAY_DIR/or-play"
REA_GPU= REA_TEST=1 REA_SYSFS=$T/nosys
fails=0
check() { # name expected-substring
  case "$(cat "$T/ran")" in
    *"$2"*) echo "ok   $1" ;;
    *) echo "FAIL $1: $(cat "$T/ran")"; fails=$((fails + 1)) ;;
  esac
}
[ "$REA_CFG" = "$CFGDIR/launcher.cfg" ] && echo "ok   launcher.cfg in the prefix" ||
  { echo "FAIL cfg: $REA_CFG"; fails=$((fails + 1)); }
[ "$REA_EXE" = "$T/OutRunLauncher.exe" ] && echo "ok   OR_LAUNCHER is the launcher" ||
  { echo "FAIL launcher: $REA_EXE"; fails=$((fails + 1)); }
CFG=$REA_CFG
for scaler in nearest pixel fsr linear; do
  printf 'resolution = 1920x1080\nscaler = %s\n' "$scaler" > "$CFG"
  rea_start > /dev/null
  check "$scaler" "gamescope -W 1920 -H 1080 -w 1920 -h 1080 -f -S fit -F $scaler -- sh $HOME/.config/or-launcher/inner.sh $T/OutRunLauncher.exe --no-window"
done
# Values match in any case, as the launcher reads them (SDK v0.3.2).
printf 'resolution = 1920x1080\nscaler = FSR\n' > "$CFG"
rea_start > /dev/null
check "scaler in capitals" "-F fsr --"
rea_start --scaler=Pixel > /dev/null
check "--scaler=Value for one run" "-F pixel --"
printf 'resolution = 1920x1080\n' > "$CFG"
rea_start > /dev/null
check "default is linear" "-F linear --"
check "our prefix" "env WINEPREFIX=$OR_PREFIX "
# One-run --scaler and --resolution win over launcher.cfg for that run
# (and reach the launcher too); the next run is launcher.cfg's again.
printf 'resolution = 1920x1080\nscaler = nearest\n' > "$CFG"
rea_start --scaler fsr > /dev/null
check "--scaler for one run" "-F fsr -- sh $HOME/.config/or-launcher/inner.sh $T/OutRunLauncher.exe --no-window --scaler fsr"
rea_start --resolution=1280x720 --no-window --scaler pixel > /dev/null
check "--resolution for one run" "gamescope -W 1280 -H 720 -w 1280 -h 720 -f -S fit -F pixel --"
rea_start --download-hd-pack --scaler fsr > /dev/null
check "a switch is not a value" "-F fsr --"
rea_start > /dev/null
check "a plain run is launcher.cfg's again" "gamescope -W 1920 -H 1080 -w 1920 -h 1080 -f -S fit -F nearest --"
# The race size from the resolution line.
size_of() {
  printf '%b' "$1" > "$CFG"
  rea_start > /dev/null
  check "$4" "gamescope -W $2 -H $3 -w $2 -h $3 -f"
}
size_of 'resolution = original\n' 640 480 "original is 640x480"
size_of 'resolution = OFF\n' 640 480 "off is 640x480 too"
size_of 'resolution = Original\n' 640 480 "Original in any case"
size_of 'resolution = 1920X1080\n' 1920 1080 "uppercase X"
size_of 'resolution = 1280x800\r\n' 1280 800 "CRLF line"
size_of 'resolution = big\n' 1920 1080 "garbage falls back"
size_of 'resolution = 12x\n' 1920 1080 "half a size falls back"
size_of 'resolution = 3840x2160\nfullscreen = true\n' 3840 2160 "exclusive fullscreen also in gamescope"
size_of 'resolution = 3840x2160\nfullscreen = false\n' 3840 2160 "the borderless window also in gamescope"

# SDL keeps reading pads in the background unless the caller chose.
check "SDL controller background events default on" "SDL_GAMECONTROLLER_ALLOW_BACKGROUND_EVENTS=1"
check "SDL joystick background events default on" "SDL_JOYSTICK_ALLOW_BACKGROUND_EVENTS=1"
SDL_GAMECONTROLLER_ALLOW_BACKGROUND_EVENTS=0 SDL_JOYSTICK_ALLOW_BACKGROUND_EVENTS=0 rea_start > /dev/null
check "the caller's SDL choices survive" "SDL_GAMECONTROLLER_ALLOW_BACKGROUND_EVENTS=0 SDL_JOYSTICK_ALLOW_BACKGROUND_EVENTS=0"

# No gamescope of our own inside one, or without one installed.
direct() {
  case "$(cat "$T/ran")" in
    *gamescope*) echo "FAIL $1: $(cat "$T/ran")"; fails=$((fails + 1)) ;;
    *"sh $HOME/.config/or-launcher/inner.sh $T/OutRunLauncher.exe --no-window") echo "ok   $1" ;;
    *) echo "FAIL $1: $(cat "$T/ran")"; fails=$((fails + 1)) ;;
  esac
}
GAMESCOPE_WAYLAND_DISPLAY=fake rea_start > /dev/null
direct "already in gamescope runs directly"
: > "$T/no-gamescope"
rea_start > /dev/null
direct "no gamescope installed runs directly"
[ "$fails" = 0 ] && echo "all passed" || { echo "$fails failed"; exit 1; }
