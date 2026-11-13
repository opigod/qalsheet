#!/usr/bin/env bash
# Build native QalSheet APKs with cargo + aapt2 + apksigner.
# Usage: scripts/build-android.sh [phone|wear|all] [abi ...]
#   ABIs: arm64-v8a, armeabi-v7a, x86_64
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

FLAVOR="${1:-all}"
shift || true
if [ "$#" -gt 0 ]; then
  ABIS=("$@")
else
  ABIS=(arm64-v8a armeabi-v7a x86_64)
fi

SDK="${ANDROID_HOME:-${ANDROID_SDK_ROOT:-}}"
if [ -z "$SDK" ]; then
  if [ -f local.properties ]; then
    SDK="$(sed -n 's/^sdk\.dir=//p' local.properties)"
  fi
fi
if [ -z "$SDK" ]; then
  SDK="$HOME/.cache/openchamber-btw-builder/android-sdk"
fi
NDK="${ANDROID_NDK_ROOT:-$(ls -d "$SDK"/ndk/* 2>/dev/null | sort -V | tail -1)}"
if [ -z "${BUILD_TOOLS:-}" ] && [ -d "$SDK/build-tools/36.0.0" ]; then
  BUILD_TOOLS="$SDK/build-tools/36.0.0"
else
  BUILD_TOOLS="${BUILD_TOOLS:-$(ls -d "$SDK"/build-tools/* 2>/dev/null | sort -V | tail -1)}"
fi
PLATFORM="$SDK/platforms/android-35/android.jar"
TOOLCHAIN="$NDK/toolchains/llvm/prebuilt/linux-x86_64"

if [ ! -f "$PLATFORM" ]; then
  echo "Android platform 35 not found at $PLATFORM" >&2
  exit 1
fi
if [ ! -d "$TOOLCHAIN" ]; then
  echo "NDK toolchain not found at $TOOLCHAIN" >&2
  exit 1
fi

CARGO="${CARGO:-$HOME/.cargo/bin/cargo}"
if [ ! -x "$CARGO" ]; then CARGO="$(command -v cargo)"; fi

DIST="$ROOT/dist"
mkdir -p "$DIST"

# Arch ships a linker-script libc++.so; some Android build-tools binaries try
# to load it at runtime. Point them at the real shared object.
COMPAT="$ROOT/target/android-compat"
mkdir -p "$COMPAT"
if [ -f /usr/lib64/x86_64-pc-linux-gnu/libc++.so.1.0 ]; then
  ln -sf /usr/lib64/x86_64-pc-linux-gnu/libc++.so.1.0 "$COMPAT/libc++.so"
elif [ -f /usr/lib/x86_64-linux-gnu/libc++.so.1.0 ]; then
  ln -sf /usr/lib/x86_64-linux-gnu/libc++.so.1.0 "$COMPAT/libc++.so"
fi
export LD_LIBRARY_PATH="$COMPAT:${LD_LIBRARY_PATH:-}"

abi_target() {
  case "$1" in
    arm64-v8a) echo "aarch64-linux-android" ;;
    armeabi-v7a) echo "armv7-linux-androideabi" ;;
    x86_64) echo "x86_64-linux-android" ;;
    *) echo "Unknown ABI $1" >&2; exit 1 ;;
  esac
}

abi_clang() {
  case "$1" in
    arm64-v8a) echo "aarch64-linux-android30-clang" ;;
    armeabi-v7a) echo "armv7a-linux-androideabi30-clang" ;;
    x86_64) echo "x86_64-linux-android30-clang" ;;
  esac
}

build_crate() {
  local crate="$1" target="$2" abi="$3"
  local var
  var="$(echo "$target" | tr '[:lower:]-' '[:upper:]_')"
  local clang="$TOOLCHAIN/bin/$(abi_clang "$abi")"
  export "CARGO_TARGET_${var}_LINKER=$clang"
  export "CC_${target//-/_}=$clang"
  export "AR_${target//-/_}=$TOOLCHAIN/bin/llvm-ar"
  echo "==> $crate for $abi"
  "$CARGO" build --release --target "$target" -p "$crate"
}

package_apk() {
  local flavor="$1"
  local crate="$2"
  local pkg="$3"
  local lib="$4"
  local manifest="$ROOT/packaging/android/AndroidManifest-$flavor.xml"
  local work="$ROOT/target/apk-$flavor"
  local compiled="$work/resources.zip"
  local unsigned="$work/unsigned.apk"
  local aligned="$work/aligned.apk"
  local apk="$DIST/$pkg-2.0.0.apk"

  rm -rf "$work"
  mkdir -p "$work/res"
  cp -R "$ROOT/packaging/android/res/." "$work/res/"

  echo "==> compiling resources for $flavor"
  "$BUILD_TOOLS/aapt2" compile --dir "$work/res" -o "$compiled"
  "$BUILD_TOOLS/aapt2" link \
    -o "$unsigned" \
    -I "$PLATFORM" \
    --manifest "$manifest" \
    --min-sdk-version 30 \
    --target-sdk-version 35 \
    --auto-add-overlay \
    "$compiled"

  for abi in "${ABIS[@]}"; do
    local target libpath dest
    target="$(abi_target "$abi")"
    libpath="$ROOT/target/$target/release/lib${lib}.so"
    if [ ! -f "$libpath" ]; then
      echo "Missing $libpath" >&2
      exit 1
    fi
    dest="$work/lib/$abi"
    mkdir -p "$dest"
    cp "$libpath" "$dest/"
  done
  (cd "$work" && zip -q -g "$unsigned" lib/*/*.so)

  "$BUILD_TOOLS/zipalign" -f 4 "$unsigned" "$aligned"
  local keystore="${ANDROID_DEBUG_KEYSTORE:-$HOME/.android/debug.keystore}"
  if [ ! -f "$keystore" ]; then
    keytool -genkeypair -v -keystore "$keystore" -storepass android \
      -alias androiddebugkey -keypass android -keyalg RSA -keysize 2048 \
      -validity 10000 -dname "CN=Android Debug,O=Android,C=US" >/dev/null
  fi
  "$BUILD_TOOLS/apksigner" sign \
    --ks "$keystore" \
    --ks-pass pass:android \
    --key-pass pass:android \
    --v1-signing-enabled true \
    --v2-signing-enabled true \
    --out "$apk" \
    "$aligned"
  echo "==> $apk"
}

if [[ "$FLAVOR" == "phone" || "$FLAVOR" == "all" ]]; then
  for abi in "${ABIS[@]}"; do build_crate qalsheet-android "$(abi_target "$abi")" "$abi"; done
  package_apk phone qalsheet-android com.qalsheet.pro qalsheet_android
fi

if [[ "$FLAVOR" == "wear" || "$FLAVOR" == "all" ]]; then
  for abi in "${ABIS[@]}"; do build_crate qalsheet-wear "$(abi_target "$abi")" "$abi"; done
  package_apk wear qalsheet-wear com.qalsheet qalsheet_wear
fi
