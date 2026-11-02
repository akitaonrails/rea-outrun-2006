#!/bin/sh
# Tests that or-play hands launcher.cfg's scaler to gamescope (-F), for
# every scaler, the race size as both the nested and the output size
# (-w/-h, -W/-H), and the SDL pad settings: sh linux/tests/scaler.sh
set -u
unset GAMESCOPE_WAYLAND_DISPLAY DRI_PRIME __NV_PRIME_RENDER_OFFLOAD
unset SDL_GAMECONTROLLER_ALLOW_BACKGROUND_EVENTS SDL_JOYSTICK_ALLOW_BACKGROUND_EVENTS
HERE=$(dirname "$(readlink -f "$0")")
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
export T HOME=$T OR_PREFIX=$T/prefix OR_SYSFS=$T/nosys
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
OR_PLAY_FUNCTIONS_ONLY=1 . "$HERE/../or-play"
RUN=$T/fake-run LAUNCHER=$T/OutRunLauncher.exe GPU=
fails=0
check() { # name expected-substring
  case "$(cat "$T/ran")" in
    *"$2"*) echo "ok   $1" ;;
    *) echo "FAIL $1: $(cat "$T/ran")"; fails=$((fails + 1)) ;;
  esac
}
CFG=$CFGDIR/launcher.cfg
for scaler in nearest pixel fsr linear; do
  printf 'resolution = 1920x1080\nscaler = %s\n' "$scaler" > "$CFG"
  start > /dev/null
  check "$scaler" "gamescope -w 1920 -h 1080 -W 1920 -H 1080 -f -S fit -F $scaler --"
done
printf 'resolution = 1920x1080\n' > "$CFG"
start > /dev/null
check "default is linear" "-F linear --"
# The race size from the resolution line.
size_of() {
  printf '%b' "$1" > "$CFG"
  start > /dev/null
  check "$4" "gamescope -w $2 -h $3 -W $2 -H $3 -f"
}
size_of 'resolution = original\n' 640 480 "original is 640x480"
size_of 'resolution = 1920X1080\n' 1920 1080 "uppercase X"
size_of 'resolution = 1280x800\r\n' 1280 800 "CRLF line"
size_of 'resolution = big\n' 1920 1080 "garbage falls back"
size_of 'resolution = 12x\n' 1920 1080 "half a size falls back"
size_of 'resolution = 3840x2160\nfullscreen = true\n' 3840 2160 "exclusive fullscreen also in gamescope"

# SDL keeps reading pads in the background unless the caller chose.
check "SDL controller background events default on" "SDL_GAMECONTROLLER_ALLOW_BACKGROUND_EVENTS=1"
check "SDL joystick background events default on" "SDL_JOYSTICK_ALLOW_BACKGROUND_EVENTS=1"
SDL_GAMECONTROLLER_ALLOW_BACKGROUND_EVENTS=0 SDL_JOYSTICK_ALLOW_BACKGROUND_EVENTS=0 start > /dev/null
check "the caller's SDL choices survive" "SDL_GAMECONTROLLER_ALLOW_BACKGROUND_EVENTS=0 SDL_JOYSTICK_ALLOW_BACKGROUND_EVENTS=0"

# No gamescope of our own inside one, or without one installed.
GAMESCOPE_WAYLAND_DISPLAY=fake start > /dev/null
check "already in gamescope runs directly" "SDL_JOYSTICK_ALLOW_BACKGROUND_EVENTS=1 sh $HOME/.config/or-launcher/inner.sh $LAUNCHER"
: > "$T/no-gamescope"
start > /dev/null
check "no gamescope installed runs directly" "SDL_JOYSTICK_ALLOW_BACKGROUND_EVENTS=1 sh $HOME/.config/or-launcher/inner.sh $LAUNCHER"
[ "$fails" = 0 ] && echo "all passed" || { echo "$fails failed"; exit 1; }
