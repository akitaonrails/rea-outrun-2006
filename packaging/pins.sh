# Shared by package.sh and appimage/build.sh (sourced, not run): reads a
# pinned download from packaging/<name>.txt (one "URL SHA-256" line, #
# comments) and fetches it into $CACHE, checked by SHA-256. The caller sets
# PINS (the packaging folder) and CACHE.

# pin <name>: prints "URL SHA-256".
pin() {
  line=$(grep -v '^#' "$PINS/$1.txt" | grep -m1 .) || { echo "no pin in packaging/$1.txt" >&2; exit 1; }
  echo "$line"
}

# fetch <name>: downloads packaging/<name>.txt's URL once into $CACHE and
# prints the cached file's path. A file that fails its digest is deleted.
fetch() {
  set -- "$1" $(pin "$1")
  mkdir -p "$CACHE"
  file=$CACHE/$1-$(printf %.12s "$3")
  if [ ! -f "$file" ]; then
    echo "downloading $1: $2" >&2
    curl -fsSL "$2" -o "$file.part" || { echo "download failed: $2" >&2; exit 1; }
    mv "$file.part" "$file"
  fi
  if ! echo "$3  $file" | sha256sum -c --quiet >&2; then
    echo "checksum mismatch for $1 ($2)" >&2
    rm -f "$file"
    exit 1
  fi
  echo "$file"
}
