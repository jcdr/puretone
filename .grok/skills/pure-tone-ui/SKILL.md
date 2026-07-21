---
name: pure-tone-ui
description: >
  Fullscreen dark dual vertical slider UI for pure-tone (custom painted
  sliders, white handles, gray rails, live labels, keep-screen-on). Use when
  changing layout, slider look/feel, labels, theme, fullscreen, or when the
  user runs /pure-tone-ui.
metadata:
  short-description: "egui dual vertical sliders + keep-screen-on"
---

# pure-tone UI

## Layout contract

- Fullscreen dark central panel (`Color32` ~ RGB 18,18,20)
- **Two equal columns** (`ui.columns(2, …)`): left frequency, right amplitude
- Each column: label on top (name + live value), then tall vertical slider centered in the half
- Label font ~28 pt, strong, light gray text
- Defaults: 440 Hz, −20 dB

Frequency slider stores **log-normalized** position in `[0,1]` and maps via `frequency_hertz_from_log_normalized`.  
Amplitude slider stores **dB** in `[-80, 0]`.

## Why custom sliders (not stock `egui::Slider`)

Stock egui paints **rail and handle** with the same `widgets.inactive.bg_fill`. Setting the handle white also whites the rail.

**Required look:**

| Part | Color |
|---|---|
| Rail (range bar) | Default dark gray `RGB(60,60,60)` |
| Handle (button) | Solid white + light stroke |

Implementation: `PureToneApp::vertical_value_slider` in `app_ui.rs`

- `allocate_exact_size` with click+drag
- Vertical mapping: top = max, bottom = min (`normalized = 1 - y_norm`)
- Rail: thin centered rect; width from `slider_rail_height`
- Handle: circle radius ≈ `rect.width() / 2.5` (clamped); white fill
- Thickness ≈ 50% of half-screen width; length fills most remaining height under labels

Do **not** reintroduce stock `Slider` for these two controls unless rail/handle colors can be separated.

## Sizing knobs (constants)

```text
SLIDER_THICKNESS_FRACTION_OF_HALF = 0.50
SLIDER_RAIL_FRACTION_OF_THICKNESS = 0.22
LABEL_FONT_SIZE = 28.0
```

## Keep screen on

Call **from `android_main`** with activity/vm pointers (UI-thread-safe early path):

1. `getWindow().addFlags(FLAG_KEEP_SCREEN_ON)` where flag = 128
2. `getDecorView().setKeepScreenOn(true)`

Calling only later from the egui frame often fails with `JavaException` (wrong thread / window not ready). Store handles via `android_context` for audio; keep-screen-on can use the same pointers.

Manifest theme already uses `Theme.DeviceDefault.NoActionBar.Fullscreen`.

## Fullscreen options

```rust
viewport: ViewportBuilder::default()
  .with_fullscreen(true)
  .with_decorations(false)
  .with_maximized(true)
```

Plus cargo-apk application theme fullscreen.

## Visual verification

After UI changes:

```bash
adb install -r target/debug/apk/PureTone.apk
adb shell am start -n com.jcdr.puretone/android.app.NativeActivity
adb shell screencap -p /sdcard/ui.png && adb pull /sdcard/ui.png
```

Inspect: two halves, gray rails, white knobs, readable labels, values update while dragging.

## Failure modes

| Symptom | Fix |
|---|---|
| Tiny vertical sliders | Stock egui uses `spacing.slider_width` as **length** for vertical; set large length or use custom slider size |
| White rails + white knobs | Do not style via shared `inactive.bg_fill`; keep custom paint |
| Handles left-biased | Center in half-column; full column width + center layout |
| Labels wrap awkwardly | Explicit `\n` between name and value |
| Screen dims while app focused | Re-check keep-screen-on from android_main; log success line |
| Tone updates lag UI | Ensure audio atomics updated every frame / on change |

## Do not

- Add play/stop chrome unless product requirements change
- Shrink touch targets below comfortable finger size on phones
- Commit screenshots into the repo
