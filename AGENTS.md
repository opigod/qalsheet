# AGENTS.md — QalSheet

This file is the working guide for anyone (human or agent) changing QalSheet.

## Project

QalSheet is a stand-alone Wear OS calculator for round watches. The reference device is the **Xiaomi Watch 2 Pro**: 1.43" AMOLED, 466 × 466 px, ~320 dpi (≈233 dp), round, rotating crown. It is a native Kotlin/Jetpack Compose app — no WebView, no React Native, no network use.

The three tabs are:

1. Calculator (numeric keypad, `fx` scientific page, history)
2. Converter (8 categories, unit pickers, amount keypad)
3. Algebra (polynomial-equation editor and solver, any letters)

The Chemistry tab was intentionally removed in 1.1.0. Do not add a fourth top-level tab without checking the round-screen tab spacing and product scope.

## Toolchain

- Kotlin 2.1.20
- AGP 8.9.3
- Gradle 8.13 (wrapper committed)
- compileSdk 35, targetSdk 35, minSdk 30
- JDK 17+ (built with Temurin 21)
- Jetpack Compose 1.9.0 + Wear Compose Foundation 1.6.2

Build:

```bash
./gradlew :app:assembleDebug
./gradlew :app:testDebugUnitTest
```

Install:

```bash
adb install -r app/build/outputs/apk/debug/app-debug.apk
adb shell am start -n com.qalsheet/.MainActivity
```

`local.properties` is gitignored. Set `sdk.dir` locally. The app uses no phone module and declares the Wear standalone metadata.

## Layout constraints (important)

The screen is a circle, not a rounded rectangle. Fixed layouts that look fine in a square preview will be clipped.

- Usable width at the very top/bottom is tiny. Do not put a close button in a literal screen corner. Full-screen dialogs use `QalDialog` with 26 dp top padding (18 dp for the amount keypad) and `DialogHeader` with 34 dp side padding.
- A 180 dp-wide card is safe in the middle of the screen but cannot extend below ~195 dp without clipping; keep wide fixed cards in the upper/middle area or make them narrower.
- The calculator page is the reference example:
  - root: 12 dp top, 30 dp tab row, 2 dp gap, 14 dp bottom
  - display and keypad both use an extra 14 dp horizontal margin (20 dp total)
  - 5-column grid; the bottom row is `blank, 0, ., +, blank` on purpose. Do not fill the bottom corners with buttons.
  - the display header carries a ~54 dp `123`/`fx` switch, a 30 dp DEG/RAD tag and an optional 20 dp history icon; keep the row to 20 dp and let the expression preview ellipsize
- The `fx` scientific page is also a 5-column, 4-row grid. Its bottom row contains only four centered function keys (`! % ± 1/x`); the top three rows hold 5 keys each. Do not add a sixth column or a fifth row; both would clip at 466 × 466.
- The Algebra page uses the same 5-column, 4-row geometry:
  - top three rows: `7 8 9 ÷ (` / `4 5 6 × )` / `1 2 3 − =`
  - bottom row: four centered keys `0 . variable +`
  - the two grouping keys stay in those positions but their labels are dynamic: they insert `(`, `[`, `{` by nesting depth and the matching closer; do not add extra bracket keys or columns
  - the display header holds the C key, backspace, variable/letter pill, a 24 dp `x^` power key and Solve; keep the row 24 dp and do not make the Solve pill wider than ~50 dp
  - the power key inserts `^` after a digit, letter or closing group; it never adds a fifth keypad row or a sixth column
  - the display is weighted 0.34 and the keypad 0.66; this is intentionally a little tighter than the calculator so the equation/result and hint fit above the keypad
- Changes to key sizes or root padding must be checked on a 466 × 466 / 320 dpi round emulator, not just in Preview.
- The converter uses a 170 × 98 dp card with a 24 dp corner radius, centered in a vertically centered page. Its category strip is a `LazyRow`; do not move it into the bottom half of the circle.
- The list pages use `ScalingLazyColumn`. For normal top-to-bottom fixed content, set `autoCentering = null` and disable the fisheye with `ScalingLazyColumnDefaults.scalingParams(edgeScale = 1f, edgeAlpha = 1f, ...)`. Beware: `rememberScalingLazyListState()` defaults to `initialCenterItemIndex = 1`, which starts a list scrolled past its first item. Set `initialCenterItemIndex = 0` for top-of-list dialogs; set it to the selected row for pickers.
- The algebra letter picker is a 4-column `ScalingLazyColumn`; selecting a letter near the bottom may require scrolling. Do not put the picker's full grid in a fixed Column.

## Architecture

```
com.qalsheet
  MainActivity.kt                 ComponentActivity + edge-to-edge
  QalSheetApp.kt                  root screen, 3-tab Wear pager, hoisted state
  ui/QalTheme.kt                  colors, Nunito family, typography, background
  ui/Components.kt                custom Canvas icons, tabs, page switch,
                                  mode tag, dialog shell, pill, fitted text
  calc/CalculatorEngine.kt        pure parser + CalculatorState
  calc/CalculatorPage.kt          calculator UI + history dialog
  convert/Units.kt                MeasureUnit, UnitCategory, conversion data
  convert/ConverterState.kt       category/unit/input state
  convert/ConverterPage.kt        converter, amount keypad, unit picker
  algebra/AlgebraEngine.kt        polynomial/rational solver + AlgebraState
  algebra/AlgebraPage.kt          equation editor, keypad, letter picker
```

State objects (`CalculatorState`, `ConverterState`, `AlgebraState`) are created in `QalSheetApp` and passed down. This keeps values alive when the pager disposes a neighbour page. Do not move long-lived state back inside a page without checking pager disposal.

## Design system

Palette (inspired by QuickCalc's fallback dark Material theme):

- ink `#07060B`, lifted ink `#100C18`
- card `#17141F`, key `#24212E`, operator `#35304A`
- violet `#8B5CF6`, soft violet `#C4B5FD`
- teal `#2DD4BF`, pink `#FF9EC4`
- text `#F7F3FF`, muted `#A39BB7`

Rules:

- Nunito only (5 static weights in `res/font`). OFL license in `licenses/nunito-OFL.txt`.
- Custom components, not phone Material3. Do not add `material-icons-extended` for one icon — draw it in `GlyphIcon` with Compose Canvas.
- Pills/keys use `RoundedCornerShape(50)` (or 12–22 dp for cards), a 1 dp white-alpha border, press scale and haptic feedback. Ripple is intentionally disabled.
- Keys: digits `QalType.Button`, operators `QalType.Operator`, equals uses the violet gradient and dark ink text.
- Text in the UI is English.

## Calculator

`CalculatorEngine` is a small recursive-descent parser:

- supports `+ − × ÷`, parentheses, decimals, unary minus, right-associative `^`
- supports `sin cos tan asin acos atan ln log/log10 sqrt`, constants `π`/`e`, postfix `!` and `%`
- angle-aware trig (`AngleMode.Degrees` / `Radians`); inverse trig returns the active mode; trig near-zero is clamped to 0
- sensible implicit multiplication (`2π`, `2(3+4)`, `2sin(30)`, `(2)(3)`, `2e`)
- `evaluable()` repairs trailing operators/unclosed parentheses and rejects unbalanced closing parentheses
- division by zero, trig/log/root domain errors and overflow return `null` → UI shows `Error`
- formatting: BigDecimal, 12 significant digits; scientific notation outside `1e-8..1e12`; leading minus becomes `−`; converter grouping uses NBSP

### Scientific mode

Scientific functions live in the Calculator tab, not as a fourth top-level tab.

- The display header contains `QalPageSwitch` (`123` / `fx`) and `QalModeTag` (`DEG` / `RAD`). Both are always visible.
- The `fx` page shows 19 full-size function keys:
  `sin cos tan asin acos atan ln log √ x² x^y π e ( ) ! % ± 1/x`.
- Tapping a function applies it to the current expression and automatically returns to the `123` page. The user then enters the argument or presses `=`.
- Prefix functions (`sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `ln`, `log`, `√`) wrap the last complete operand if there is one: `12+30` + sin → `12+sin(30)`. If there is no operand, they append `sin(`.
- Postfix/utility keys:
  - `x²` appends `^2`
  - `x^y` appends `^`
  - `1/x` wraps the last operand as `1/(…)`
  - `!` and `%` append postfix tokens; `%` means divide by 100 (`200+10%` = 200.1)
  - `±` negates the whole expression
  - `π`/`e` append constants; implicit multiplication handles `2π`
  - `(`/`)` insert/close parentheses; equals also closes unmatched parentheses through `evaluable()`
- `applyScientific()` first closes an incomplete parenthesis group if the expression ends with a complete value, so `sin(30` + `x²` becomes `sin(30)^2` rather than an invalid expression.
- Angle mode is part of `CalculatorState` and persists across tab switches. Trig history entries record the mode; the history row displays `DEG`/`RAD`. Loading a history entry restores its angle mode.
- The fx page is not a second numeric pad; there is no `=`, `C` or backspace there. The state auto-returns to the numeric page after a function insertion.
- `log` is base 10 and is labelled `log` on the key; the engine also accepts `log10(...)`.

The standard UI intentionally exposes only the numeric keypad; parentheses, powers and scientific functions are reached through the fx page. The equals key is in the right column of row 3, matching the QuickCalc layout. History is in memory, newest first, capped at 40, and is not persisted.

When changing parser or function-application logic, update `CalculatorEngineTest` and `ScientificCalculatorTest`, and check both keypad pages on the round emulator.

## Algebra

`AlgebraEngine` is a separate polynomial-equation solver. It does not use the scientific calculator’s numeric parser.

- The parser builds multivariate polynomials and rational quotients while parsing. Integer powers are expanded, so `(x+1)^2` becomes `x² + 2x + 1`; `x²`/`x³` Unicode input is normalized to `^`.
- Supports `+ − × ÷ ^`, parentheses, nested `[ ]` and `{ }` groups, decimals (dot or comma), unary signs and implicit multiplication (`3x`, `2(x+y)`, `2[3x+1]`).
- Powers must have a whole-number exponent from 0 to 12. `x^y`, `x^0.5` and `x^-1` are rejected with a clear message.
- A single-letter equation is solved by degree:
  - degree 1 → one exact root;
  - degree 2 → exact quadratic formula, both real roots, `No real solution` when the discriminant is negative;
  - degree 3–12 → numeric real roots (Durand–Kerner), distinct roots only.
- Quadratic roots are returned as `SolvedRoots` when there are two; higher-degree equations do the same for multiple real roots. The display joins them, e.g. `x = −2, 2`.
- Linear equations with several letters return a symbolic expression for the selected letter. Powers with more than one letter are rejected (`Use one letter with powers`).
- Division may be by a constant or a polynomial. The parser keeps every denominator as a nonzero constraint; roots that make a denominator zero are rejected as `No solution`. Nested fractions keep all constraints.
- `3x + 5 = 20` → `x = 5`.
- `3t + 5 = 20` → `t = 5`; single-variable equations auto-detect the letter even if another letter is selected.
- `2t + 3q = 12` solved for `t` → `t = 6 − 1.5q`.
- If multiple letters remain and the selected letter is absent, the result is an error asking the user to choose a letter from the equation.
- If all terms cancel: zero → infinite solutions; nonzero constant → no solution.
- Answers are rounded to six decimals (scientific notation outside a reasonable range); symbolic coefficients use the same formatter. Numeric higher-degree roots are approximations rounded for display.

UI/state rules:

- `AlgebraState` holds `equation`, `result` and the selected `variable`.
- The equation keypad inserts the selected variable through `insertVariable()`. To enter multiple letters, the user selects another letter via the variable pill, then taps the variable key.
- The header power key calls `insertPower()`; it appends `^` only after a digit, letter or closing group. The exponent digits are typed normally.
- Selecting a letter clears the result but keeps the equation.
- `=` inserts the equation separator. Pressing `=` again, or tapping **Solve**, evaluates.
- Editing any key after a solution clears the result and continues editing the equation.
- The letter picker is a full-screen dialog; it uses a 4-column scaling list and centers the currently selected row.
- `formatExpression()` formats symbolic right-hand sides, e.g. `6 − 1.5y`; signs, unit coefficients and sorted letters matter.
- Multiple roots use `SolvedRoots` and display as `x = −2, 2`. Complex-only equations use `NoRealSolution` → `No real solution`.

When changing the solver, extend `AlgebraEngineTest` with numeric, quadratic, higher-power, multi-letter, no-solution, infinite-solution, no-real-solution and malformed-input cases.

## Converter

`MeasureUnit` uses an affine conversion to a category base:

```
base = (value + offset) * factor
value = base / factor - offset
```

Base units: metre, gram, Celsius, litre, m², m/s, second, byte. Temperature is the only affine case.

- Defaults: cm → mm, kg → g, °C → °F, l → ml, m² → cm², km/h → m/s, h → min, MB → GB.
- Data uses decimal (kB = 1000 B) and binary (KiB = 1024 B) prefixes; keep labels explicit.
- Swap sets the input to the previous result so the relationship stays valid (1 cm = 10 mm → 10 mm = 1 cm).
- The amount keypad is custom; there is no system keyboard. It supports sign toggle, decimal, clear and backspace, with a 12-character input cap.
- Unit pickers use `ScalingLazyColumn` and center the currently selected unit.

When adding a unit, keep the symbol unique inside its category and update `UnitsTest`.

## Testing / QA checklist

Emulator used during development:

```bash
avdmanager create avd -n qalsheet_wear -k "system-images;android-33;android-wear;x86_64" -d wearos_large_round
# then set hw.lcd.width/height=466, density=320, circular=true
emulator -avd qalsheet_wear -no-window -gpu swiftshader_indirect -skin 466x466 ...
```

Before shipping a layout change:

- Calculator: all 18 keys of the numeric page and all 19 function keys of the fx page fully visible on the 466 × 466 round screen; `123`/`fx` switch and DEG/RAD tag fit the 20 dp header; long results shrink/ellipsize; history opens from the clock icon.
- Scientific mode: DEG/RAD changes trig results; `sin(30)` in DEG is 0.5; applying a function returns to the numeric page; an out-of-domain expression (`asin(2)` or `tan(90)`) shows a red Error; history shows the mode.
- Algebra: `3x+5=20` solves to `x = 5`; `3t+5=20` solves to `t = 5`; `2t+3q=12` solved for `t` shows `t = 6 − 1.5q`; `4[y−3(2y−1)]=2[y−2(3y+2)]` solves to `y = 2`; `5/x=2/3` solves to `x = 7.5`; `x^2=4` shows `x = −2, 2`; `x^3−6x^2+11x−6=0` shows `x = 1, 2, 3`; `x^2+1=0` shows `No real solution`; the power key inserts `^`; the letter picker can choose a visible letter; the C/backspace/power/Solve row and all 19 equation keys are fully visible; the hint/result text fits above the keypad.
- Converter: all 8 categories reachable, category row scrolls, unit picker selected row centered, amount keypad keys not clipped, sign toggle and swap work.
- Round-screen mask: no interactive control in the outer corners.
- Run `./gradlew :app:testDebugUnitTest :app:assembleDebug`.

Reference screenshots live in `docs/screenshots/`: basic keypad, fx page, RAD state, error result, algebra page, algebra solution, multi-letter algebra, converter and amount entry.

## Versioning and release

- Current app version: `1.3.0` (`versionCode 4`), adding brackets, rational equations, and powers to Algebra.
- Build the APK, then create a GitHub release with the debug-signed APK. Update `versionCode`/`versionName` in `app/build.gradle.kts` for every published build.
- The GitHub repository is `opigod/qalsheet`; releases use tags such as `v1.3.0`.
- Keep release notes honest: the APK is debug-signed for sideloading, not a Play Store build. Include a SHA-256 checksum.

## Known limitations / good next steps

- No ambient/AOD, tile or watch-face complication yet.
- Calculator history, active keypad page, angle mode and algebra state are not persisted across process death.
- No cursor/selection editing: expressions and equations are append/backspace only.
- Algebra solves single-letter polynomial equations up to degree 12. Degree 1–2 are exact; degrees 3–12 use numeric real roots. Complex roots are not shown (only `No real solution`), powers with multiple letters are rejected, and systems/inequalities are not implemented.
- Adding a settings screen for haptics/theme would require a fourth tab or a dialog; measure the round safe area first.
