#!/usr/bin/env bash
# Build a self-contained AppImage from the Linux release binary.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

CARGO="${CARGO:-$HOME/.cargo/bin/cargo}"
if [ ! -x "$CARGO" ]; then CARGO="$(command -v cargo)"; fi

TOOLS="${APPIMAGE_TOOLS:-/tmp/opencode/appimage-tools}"
mkdir -p "$TOOLS"
LINUXDEPLOY="$TOOLS/linuxdeploy-x86_64.AppImage"
APPIMAGETOOL="$TOOLS/appimagetool-x86_64.AppImage"
if [ ! -x "$LINUXDEPLOY" ]; then
  curl -sSfL -o "$LINUXDEPLOY" \
    https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-x86_64.AppImage
  chmod +x "$LINUXDEPLOY"
fi
if [ ! -x "$APPIMAGETOOL" ]; then
  curl -sSfL -o "$APPIMAGETOOL" \
    https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-x86_64.AppImage
  chmod +x "$APPIMAGETOOL"
fi

echo "==> building release binary"
"$CARGO" build --release -p qalsheet-desktop

APPDIR="$ROOT/target/QalSheet.AppDir"
rm -rf "$APPDIR"
mkdir -p "$APPDIR/usr/bin" "$APPDIR/usr/share/applications" \
  "$APPDIR/usr/share/icons/hicolor/512x512/apps"
cp "$ROOT/target/release/qalsheet" "$APPDIR/usr/bin/qalsheet"
cp "$ROOT/packaging/linux/qalsheet.desktop" "$APPDIR/usr/share/applications/"
cp "$ROOT/packaging/linux/qalsheet.png" "$APPDIR/usr/share/icons/hicolor/512x512/apps/qalsheet.png"
cp "$ROOT/packaging/linux/qalsheet.png" "$APPDIR/qalsheet.png"
cp "$ROOT/packaging/linux/qalsheet.desktop" "$APPDIR/qalsheet.desktop"
cp "$ROOT/packaging/linux/AppRun" "$APPDIR/AppRun"
chmod +x "$APPDIR/AppRun" "$APPDIR/usr/bin/qalsheet"

echo "==> bundling Linux libraries"
ARCH=x86_64 "$LINUXDEPLOY" --appimage-extract-and-run \
  --appdir "$APPDIR" \
  --executable "$APPDIR/usr/bin/qalsheet" \
  --desktop-file "$APPDIR/qalsheet.desktop" \
  --icon-file "$APPDIR/qalsheet.png" \
  --no-appstream 2>&1 | tail -20 || true

# linuxdeploy can replace AppRun; always use our tiny launcher.
cp "$ROOT/packaging/linux/AppRun" "$APPDIR/AppRun"
chmod +x "$APPDIR/AppRun"

mkdir -p "$ROOT/dist"
OUT="$ROOT/dist/QalSheet-2.0.0-x86_64.AppImage"
rm -f "$OUT"
ARCH=x86_64 VERSION=2.0.0 "$APPIMAGETOOL" --appimage-extract-and-run \
  --no-appstream "$APPDIR" "$OUT"
chmod +x "$OUT"
echo "==> $OUT"
