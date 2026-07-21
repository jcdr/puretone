# Custom vertical slider paint contract

## Geometry

- Outer rect: `(slider_thickness × slider_length)`, centered in half-column
- Handle radius: `clamp(width/2.5, 18, 48)`
- Usable Y range: outer Y shrunk by handle radius (handle stays fully visible)
- Pointer → value: `normalized = 1 - clamp((y - usable.min) / usable.span)`
- Value → Y: `y = usable.min + (1 - normalized) * usable.span`
- Top of rail = high value; bottom = low value

## Colors

- Rail fill: `Color32::from_rgb(60, 60, 60)` — default-like dark gray
- Handle fill: `Color32::WHITE`
- Handle stroke: light gray ~`(220,220,230)`, width 1.5 idle / 2.0 active

## Why not stock egui::Slider

Rail uses `widgets.inactive.bg_fill`; handle uses interact `bg_fill`. At rest both are inactive → **one color**. White handle without custom paint forces white rail.
