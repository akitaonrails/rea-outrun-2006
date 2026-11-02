#!/bin/sh
# Prints the CHANGELOG.md section of one version, for its GitHub release:
#
#   packaging/release-notes.sh <version or tag> [changelog]
#
# "v0.2.0" and "0.2.0" both find "## [0.2.0] - <date>". Fails when that
# section is missing or empty, so a release never goes out without notes.
set -eu
version=${1#v}
changelog=${2:-CHANGELOG.md}
# Plain string functions, no bracket expressions: the same in gawk, mawk
# and busybox awk. A "[x]: url" link line ends the last section.
notes=$(${AWK:-awk} -v v="$version" '
  substr($0, 1, 4) == "## [" {
    if (take) exit
    head = substr($0, 5); head = substr(head, 1, index(head, "]") - 1)
    if (head == v) { take = 1; next }
  }
  substr($0, 1, 1) == "[" && index($0, "]: ") > 1 { if (take) exit }
  take { print }
' "$changelog")
# Drop leading and trailing blank lines.
notes=$(printf '%s\n' "$notes" | sed -e '/./,$!d' | sed -e ':a' -e '/^\n*$/{$d;N;ba' -e '}')
[ -n "$notes" ] || { echo "no CHANGELOG section for $version" >&2; exit 1; }
printf '%s\n' "$notes"
