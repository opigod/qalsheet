# AGENTS.md — QalSheet 2

Working guide for anyone (human or agent) changing QalSheet.

## What this is

QalSheet is a native scientific calculator for **Linux**, **Android** and **Wear OS**. The entire
application — UI and math — is Rust. It uses egui/eframe with the glow (OpenGL) renderer.

Hard constraints:

- **No JVM in the app.** No Kotlin/Java source, no Compose runtime, no DEX code, no WebView,
  no Electron.
- **No network use.** No permissions for internet, no telemetry.
- **Same visual language as the original Wear app:** ink `#07060B`, Nunito, violet `#8B5CF6`,
  teal `#2DD4BF`, round keys, fixed dark theme.
- The watch keeps the original three tabs and the original 5 × 4 calculator grid. Do not add
  graph/tools to the watch.
- Do not add a fourth watch tab without checking the round-screen safe area at 466 × 466 / 320 dpi.

## Workspace

```
Cargo.toml                    Rust workspace (edition 2024, resolver 3)
crates/qalsheet-core          pure math/state, no egui dependency
crates/qalsheet-ui            shared egui screens for all targets
crates/qalsheet-desktop       Linux binary
crates/qalsheet-android       Android phone NativeActivity cdylib
crates/qalsheet-wear          Wear OS NativeActivity cdylib
packaging/android             manifests + PNG icons
packaging/linux               desktop entry, icon, AppRun
scripts/build-android.sh      cargo + NDK + aapt2 + apksigner packaging
scripts/build-appimage.sh     AppImage packaging
PKGBUILD                      Arch package
dist/                         built APKs and AppImage (signed debug)
docs/screenshots              current screenshots
```

## Build and test

```bash
cargo test --workspace
cargo build --release -p qalsheet-desktop
./target/release/qalsheet
```

Android requires an SDK, NDK r27+ and the Rust targets:

```bash
rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android
export ANDROID_HOME="$HOME/Android/Sdk"
export ANDROID_NDK_ROOT="$ANDROID_HOME/ndk/27.2.12479018"
./scripts/build-android.sh all
```

The APKs install with `adb install -r`. Both are debug-signed:
`dist/com.qalsheet.pro-2.0.0.apk` (phone) and `dist/com.qalsheet-2.0.0.apk` (watch).

AppImage:

```bash
./scripts/build-appimage.sh
```

## Core behavior

### Expression engine (`expr.rs`)

Recursive-descent parser with an AST. Supports `+ - * / ^`, Unicode `× ÷ −`, parentheses,
implicit multiplication (`2π`, `2(3+4)`, `2sin(30)`), postfix `!`/`%`, `π e τ φ ans`, functions
`sin cos tan asin acos atan sinh cosh tanh asinh acosh atanh ln log log2 exp sqrt cbrt abs floor
ceil round sign min max mod`.

- Powers are right-associative.
- Trig respects `AngleMode`; inverse trig returns the active mode.
- `tan(90)` in degrees, roots of negatives, logs of non-positives, factorial of non-integers and
  overflow return an error.
- `evaluable()` repairs trailing operators/unmatched `(` for interactive input.
- `Expression` is compiled once and reused by graph/calculus.

### Calculator (`calculator.rs`)

`CalculatorState` owns expression, result, angle mode, scientific-page flag, 100-entry history,
memory and `ans`. `press()` is the keypad API; `apply_function()` wraps the last operand like the
Wear app (`12+30` + sin → `12+sin(30)`). Function insertion returns to the numeric page.

### Algebra (`algebra.rs`)

Sparse multivariate polynomial parser and rational solver. Powers expand at parse time. One-letter
equations are solved exactly through degree 2 and numerically through 12; multi-letter equations
return a symbolic expression for the selected letter when linear. Every denominator is kept as a
nonzero constraint and extraneous roots are rejected.

### Graph (`graph.rs`)

`find_roots` samples the window, refines sign changes with bisection, verifies the residual and
rejects vertical asymptotes. `find_intersections` is the root of a−b. `nice_step` drives the grid.
The UI compiles each function once and reuses the AST while sampling pixels.

### Units (`units.rs`)

Every category converts through a base unit with `base = (value + offset) * factor`. Temperature is
the only affine category. The current categories are length, mass, temperature, volume, area,
speed, time, data, angle, pressure and energy.

### Tools

- `stats.rs`: quartiles, sample/population variance and SD, mode, paired regression.
- `programmer.rs`: 32/64-bit width, decimal/hex/octal/binary, AND OR XOR SHL SHR NOT.
- `calculus.rs`: 5-point derivative, adaptive Simpson integration.
- `constants.rs`: physical constants; tap copies, `+` inserts into the calculator.

## UI rules

- `theme.rs` is the source of truth for colors, fonts and metric scaling.
- Use the custom widgets in `widgets.rs`; do not reach for Material widgets.
- Pills/keys are rounded pills with a 1 dp white-alpha border, press animation and no ripple.
- All screens use absolute-region painting for the custom look. If a widget needs standard text
  editing or scrolling, create a child `Ui` with `new_child(UiBuilder…)`.
- Responsive breakpoints in `Metrics`:
  - watch if `Wear` flavor or width < 320;
  - wide/desktop when width ≥ 780;
  - otherwise compact bottom-nav layout.
- Watch safe area:
  - three 30 dp tabs centred at the top (not a full-width bar);
  - content side padding 18 dp, bottom inset 22 dp;
  - calculator and science keypads are **5 columns × 4 rows**; bottom corners stay blank;
  - the arithmetic bottom row is `blank, 0, ., +, blank`;
  - the science bottom row is `!, %, ±, 1/x` centred with half-key margins;
  - do not replace this with the narrow trapezoid keypad. The user explicitly prefers the large
    Kotlin-style keys.
- Modals use `egui::Modal` with a card frame. On watch use `common::modal_width` /
  `modal_max_height` and a narrow variable-picker grid.
- Text that has no glyph in Nunito/SF fallback appears as a box. Prefer ASCII labels (`x^y`,
  `cbrt`, `exp`, `log2`, `!`) over uncommon Unicode.

## Android packaging

- `hasCode=false`; only `NativeActivity` and native `.so` files.
- Manifests are in `packaging/android/`.
- `scripts/build-android.sh` builds each ABI with the NDK clang wrappers and packages with
  `aapt2`, `zipalign` and `apksigner`.
- A user-local `libc++.so` shim is created for Arch's linker-script issue.
- Keep `extractNativeLibs=true` unless you verify 16 KB page alignment.
- Update `versionCode`/`versionName` in both manifests and `Cargo.toml` for every release.

## Release checklist

- [ ] `cargo test --workspace`
- [ ] `cargo build --release -p qalsheet-desktop`
- [ ] `./scripts/build-android.sh all`
- [ ] `./scripts/build-appimage.sh`
- [ ] Install phone APK on an Android emulator/device and watch APK on Wear.
- [ ] Check calculator, fx, graph, algebra, converter and every tools tab.
- [ ] Check text fields open the system keyboard and results update live.
- [ ] Check the round watch screen: all keys visible, no text outside its bubble, bottom row large.
- [ ] Check no ANRs on cold start.
- [ ] Update screenshots in `docs/screenshots/`.
- [ ] Update version in `Cargo.toml` (workspace package), manifests and PKGBUILD.
- [ ] Publish debug-signed APKs/AppImage with SHA-256 checksums.

## Known limitations

- History/memory/tab are in-memory only.
- Graph is real-valued only; no polar/parametric/3D.
- Algebra returns only real roots and no full worked steps.
- Watch intentionally has no graph/tools.
- AppImage relies on the host X11/Wayland/OpenGL stack (standard on Arch).
