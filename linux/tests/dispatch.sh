#!/bin/sh
# Tests or-play's commands in both layouts: the library next to it (zip,
# AppImage) and in ../sdk (this repository): sh linux/tests/dispatch.sh
set -u
HERE=$(dirname "$(readlink -f "$0")")
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
export T HOME=$T OR_PREFIX=$T/prefix
cat > "$T/fake-run" <<'FAKE'
#!/bin/sh
case "$*" in *"wineserver -k"*) echo killed >> "$T/wineserver" ;; esac
exit 0
FAKE
chmod +x "$T/fake-run"
export OR_RUN=$T/fake-run
mkdir -p "$T/pkg/linux" "$T/bare"
cp "$HERE/../or-play" "$HERE/../../sdk/rea-play.sh" "$T/pkg/linux/"
cp "$HERE/../or-play" "$T/bare/"
: > "$T/pkg/OutRunLauncher.exe"
fails=0
check() { # name got want
  if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2', want '$3'"; fails=$((fails + 1)); fi
}
for play in "$T/pkg/linux/or-play" "$HERE/../or-play"; do
  check "status ($play)" "$(sh "$play" status)" idle
  check "stop ($play)" "$(sh "$play" stop)" stopped
  sh "$play" nope > /dev/null 2>&1
  check "an unknown command exits 2 ($play)" "$?" 2
done
[ -f "$T/wineserver" ] && echo "ok   stop ends our prefix's wineserver" || { echo "FAIL no wineserver -k"; fails=$((fails + 1)); }
out=$(OR_LAUNCHER= sh "$HERE/../or-play" --play 2>&1); rc=$?
check "no launcher: exit 1" "$rc" 1
case $out in *"OutRunLauncher.exe not found next to or-play (or set OR_LAUNCHER)"*) echo "ok   no launcher: names OR_LAUNCHER" ;;
  *) echo "FAIL message: $out"; fails=$((fails + 1)) ;; esac
out=$(sh "$T/bare/or-play" status 2>&1); rc=$?
check "no library: exit 1" "$rc" 1
case $out in *"rea-play.sh not found"*) echo "ok   no library: a clear message" ;;
  *) echo "FAIL message: $out"; fails=$((fails + 1)) ;; esac
[ "$fails" = 0 ] && echo "all passed" || { echo "$fails failed"; exit 1; }
