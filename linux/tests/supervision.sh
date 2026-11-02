#!/bin/sh
# Tests or-play's own-session lock, stop and inner script with fakes; only
# harmless sleeps run: sh linux/tests/supervision.sh
set -u
unset GAMESCOPE_WAYLAND_DISPLAY
HERE=$(dirname "$(readlink -f "$0")")
T=$(mktemp -d)
export T HOME=$T OR_PREFIX=$T/prefix OR_SYSFS=$T/nosys
STALE_PID= STALE_START= OTHER_PID= OTHER_START= GAME_PID=
stop_helper() {
  [ -n "$1" ] || return 0
  stats=$(proc_stat "$1") || return 0
  if [ "${stats#* }" = "$2" ]; then kill "$1" 2>/dev/null || true; fi
  wait "$1" 2>/dev/null || true
}
cleanup() {
  stop_helper "$STALE_PID" "$STALE_START"
  stop_helper "$OTHER_PID" "$OTHER_START"
  [ -n "$GAME_PID" ] && kill "$GAME_PID" 2>/dev/null
  rm -rf "$T"
}
trap cleanup EXIT HUP INT TERM
mkdir -p "$OR_PREFIX/drive_c/users/me/AppData/Local/OutRunLauncher"
OR_PLAY_FUNCTIONS_ONLY=1 . "$HERE/../or-play"
fails=0
ok() { echo "ok   $1"; }
fail() { echo "FAIL $1"; fails=$((fails + 1)); }

# The wrapper: records wineserver -k, runs everything else as given.
cat > "$T/fake-run" <<'FAKE'
#!/bin/sh
case "$*" in
  *"command -v gamescope"*) exit 1 ;;
  *"wineserver -k"*) echo killed >> "$T/wineserver"; exit 0 ;;
esac
exec "$@"
FAKE
chmod +x "$T/fake-run"
RUN=$T/fake-run GPU=

# A live pid with the wrong start time: a stale lock, or a reused pid.
sleep 30 & STALE_PID=$!
stats=$(proc_stat "$STALE_PID"); STALE_START=${stats#* }
printf '%s %s\n' "$STALE_PID" "$((STALE_START + 1))" > "$LOCK"
if running; then fail "stale identity reported running"; else ok "stale/reused pid refused"; fi
stop_session
if kill -0 "$STALE_PID" 2>/dev/null; then ok "stop left the stale/reused pid alone"; else fail "stale pid was signalled"; fi
[ -f "$T/wineserver" ] && ok "stop ends our prefix's wineserver" || fail "stop did not run wineserver -k"
stop_helper "$STALE_PID" "$STALE_START"; STALE_PID= STALE_START=

# A lock holding or-play's own shell (the old lock) is no session of ours.
echo $$ > "$LOCK"
if running; then fail "a pid-only lock counted"; else ok "pid-only lock refused"; fi
stop_session
ok "stop did not kill the helper's own shell"

# Another OR2006C2C.exe (Steam's, say) is not ours.
ln -s "$(command -v sleep)" "$T/OR2006C2C.exe"
"$T/OR2006C2C.exe" 30 & OTHER_PID=$!
stats=$(proc_stat "$OTHER_PID"); OTHER_START=${stats#* }
rm -f "$LOCK"
if running; then fail "unrelated OR2006C2C.exe blocked a launch"; else ok "unrelated OR2006C2C.exe doesn't block"; fi
stop_session
if kill -0 "$OTHER_PID" 2>/dev/null; then ok "unrelated OR2006C2C.exe not signalled"; else fail "unrelated game was signalled"; fi
stop_helper "$OTHER_PID" "$OTHER_START"; OTHER_PID= OTHER_START=

# A real launch: the "game" is a sleep in its own session. While it runs
# or-play reports running; stop ends the whole session and not us.
mkdir -p "$T/bin"
cat > "$T/bin/wine" <<'FAKE'
#!/bin/sh
printf '<%s>\n' "$@" > "$T/args"
sleep 30 &
wait
FAKE
cat > "$T/bin/wineserver" <<'FAKE'
#!/bin/sh
exit 0
FAKE
chmod +x "$T/bin/wine" "$T/bin/wineserver"
PATH=$T/bin:$PATH
mkdir -p "$T/launcher it's here"
LAUNCHER=$T/"launcher it's here"/OutRunLauncher.exe; : > "$LAUNCHER"
start > /dev/null & GAME_PID=$!
i=0; while [ ! -f "$T/args" ] && [ $i -lt 50 ]; do sleep 0.1; i=$((i + 1)); done
if running; then ok "a launched session is running"; else fail "launched session not seen"; fi
if [ "$(head -1 "$T/args")" = "<$LAUNCHER>" ]; then ok "apostrophe+space launcher is one argument"; else fail "launcher argument: $(cat "$T/args")"; fi
read_lock; session=$LOCK_PID
[ "$session" != $$ ] && ok "the lock holds the game's session, not or-play" || fail "lock holds or-play's pid"
stop_session
i=0; while kill -0 "$GAME_PID" 2>/dev/null && [ $i -lt 50 ]; do sleep 0.1; i=$((i + 1)); done
if kill -0 "$GAME_PID" 2>/dev/null; then fail "or-play still waiting after stop"; else ok "stop ended the game session"; fi
if pgrep -s "$session" >/dev/null 2>&1; then fail "processes left in the session"; else ok "nothing left in the session"; fi
GAME_PID=
[ -f "$LOCK" ] && fail "stop left the lock" || ok "stop removed the lock"

# A natural exit removes the lock.
cat > "$T/bin/wine" <<'FAKE'
#!/bin/sh
exit 0
FAKE
start > /dev/null
[ -f "$LOCK" ] && fail "natural exit left the lock" || ok "natural exit removes the lock"

case "$(cat "$STATE/inner.sh")" in
  *'wine "$1" --no-window'*) ok "inner script takes the launcher as \$1" ;;
  *) fail "inner script does not use wine \"\$1\"" ;;
esac
case "$(cat "$STATE/inner.sh")" in
  *"$LAUNCHER"*) fail "launcher path written into the inner script" ;;
  *) ok "launcher path absent from the inner script" ;;
esac

# The settings window: Play (0) and no window (4) go on to the game;
# closing it (3) or anything else doesn't.
cat > "$T/fake-run" <<'FAKE'
#!/bin/sh
case "$*" in *"--settings --linux"*) echo "$*" > "$T/settings"; exit "$(cat "$T/rc")" ;; esac
exit 0
FAKE
for case in 0:0 4:0 3:1 1:1; do
  echo "${case%:*}" > "$T/rc"
  if settings; then got=0; else got=1; fi
  [ "$got" = "${case#*:}" ] && ok "settings exit ${case%:*} -> $got" || fail "settings exit ${case%:*} gave $got"
done
case "$(cat "$T/settings")" in
  *"WINEPREFIX=$OR_PREFIX"*"SDL_JOYSTICK_ALLOW_BACKGROUND_EVENTS=1"*"wine $LAUNCHER --settings --linux") ok "settings window: our prefix, SDL, helper flags" ;;
  *) fail "settings command: $(cat "$T/settings")" ;;
esac

[ "$fails" = 0 ] && echo "all passed" || { echo "$fails failed"; exit 1; }
