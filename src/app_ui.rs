use crate::audio_engine::SineAudioEngine;
use crate::audio_math::{
    frequency_hertz_from_log_normalized, log_normalized_from_frequency_hertz,
    MAXIMUM_AMPLITUDE_DECIBELS, MINIMUM_AMPLITUDE_DECIBELS,
};
use eframe::egui::{
    self, pos2, Align, Align2, Color32, FontFamily, FontId, Layout, Rect, RichText, Sense, Shape,
    Stroke, Vec2,
};

const DEFAULT_FREQUENCY_HERTZ: f32 = 440.0;
const DEFAULT_AMPLITUDE_DECIBELS: f32 = -20.0;
const LABEL_FONT_SIZE: f32 = 28.0;
const VALUE_FONT_SIZE: f32 = 28.0;
const FREQUENCY_WIDEST_NUMBER_TEMPLATE: &str = "00000";
const AMPLITUDE_WIDEST_NUMBER_TEMPLATE: &str = "-000.0";
const SLIDER_THICKNESS_FRACTION_OF_HALF: f32 = 0.50;
const SLIDER_RAIL_FRACTION_OF_THICKNESS: f32 = 0.22;
const SLIDER_RAIL_COLOR: Color32 = Color32::from_rgb(60, 60, 60);
const SLIDER_HANDLE_FILL: Color32 = Color32::WHITE;
const SLIDER_HANDLE_STROKE: Color32 = Color32::from_rgb(220, 220, 230);
const VALUE_TEXT_COLOR: Color32 = Color32::from_rgb(230, 230, 235);
const BASE_PADDING: f32 = 16.0;
const BASE_MIN_SLIDER_LENGTH: f32 = 80.0;
const MIN_LAYOUT_SCALE: f32 = 0.50;
const NARROW_LAYOUT_SCALE: f32 = 0.35;

pub struct PureToneApp {
    frequency_log_normalized: f32,
    amplitude_decibels: f32,
    audio_engine: SineAudioEngine,
    dark_style_applied: bool,
}

struct ControlLayout {
    label_font_size: f32,
    value_font_size: f32,
    padding: f32,
    slider_thickness: f32,
    slider_rail_height: f32,
    handle_radius_min: f32,
    handle_radius_max: f32,
    /// Set when the column must be taller than the viewport so a scroll area can show it.
    column_height: Option<f32>,
    needs_scroll: bool,
}

impl Default for PureToneApp {
    fn default() -> Self {
        Self::new()
    }
}

impl PureToneApp {
    pub fn new() -> Self {
        let frequency_log_normalized = log_normalized_from_frequency_hertz(DEFAULT_FREQUENCY_HERTZ);
        let amplitude_decibels = DEFAULT_AMPLITUDE_DECIBELS;
        let audio_engine = SineAudioEngine::start(DEFAULT_FREQUENCY_HERTZ, amplitude_decibels);
        Self {
            frequency_log_normalized,
            amplitude_decibels,
            audio_engine,
            dark_style_applied: false,
        }
    }

    fn frequency_hertz(&self) -> f32 {
        frequency_hertz_from_log_normalized(self.frequency_log_normalized)
    }

    fn value_font_id(font_size: f32) -> FontId {
        FontId::new(font_size, FontFamily::Proportional)
    }

    fn format_frequency_number(frequency_hertz: f32) -> String {
        format!("{:.0}", frequency_hertz)
    }

    fn format_amplitude_number(amplitude_decibels: f32) -> String {
        format!("{:.1}", amplitude_decibels)
    }

    /// Width of the fixed number box for `widest_number_template`.
    /// Each `'0'` is one digit position (the widest of `'0'..='9'`); every other
    /// character, such as `'-'` or `'.'`, uses its own glyph width.
    fn number_box_width(ui: &egui::Ui, font_size: f32, widest_number_template: &str) -> f32 {
        let font_id = Self::value_font_id(font_size);
        ui.fonts(|fonts| {
            let digit_width = ('0'..='9')
                .map(|digit| fonts.glyph_width(&font_id, digit))
                .fold(0.0_f32, f32::max);
            widest_number_template
                .chars()
                .map(|character| {
                    if character == '0' {
                        digit_width
                    } else {
                        fonts.glyph_width(&font_id, character)
                    }
                })
                .sum()
        })
    }

    fn laid_out_text_width(ui: &egui::Ui, font_size: f32, text: &str) -> f32 {
        let font_id = Self::value_font_id(font_size);
        ui.fonts(|fonts| {
            fonts
                .layout_no_wrap(text.to_owned(), font_id, VALUE_TEXT_COLOR)
                .rect
                .width()
        })
    }

    /// `(group_width, number_box_width)` for the centered value label.
    /// Group width is the widest number box plus the laid-out unit, so the unit
    /// stays at one x for every value.
    fn value_group_geometry(
        ui: &egui::Ui,
        font_size: f32,
        widest_number_template: &str,
        unit_text: &str,
    ) -> (f32, f32) {
        let number_box_width = Self::number_box_width(ui, font_size, widest_number_template);
        let unit_width = Self::laid_out_text_width(ui, font_size, unit_text);
        (number_box_width + unit_width, number_box_width)
    }

    fn paint_fixed_value_with_unit(
        ui: &mut egui::Ui,
        number_text: &str,
        unit_text: &str,
        widest_number_template: &str,
        value_font_size: f32,
    ) -> Rect {
        let font_id = Self::value_font_id(value_font_size);
        let (group_width, number_box_width) =
            Self::value_group_geometry(ui, value_font_size, widest_number_template, unit_text);
        let row_height = ui.fonts(|fonts| fonts.row_height(&font_id));
        let (rect, _response) =
            ui.allocate_exact_size(Vec2::new(group_width, row_height), Sense::hover());

        let number_galley = ui.fonts(|fonts| {
            fonts.layout_no_wrap(number_text.to_owned(), font_id.clone(), VALUE_TEXT_COLOR)
        });
        let unit_galley =
            ui.fonts(|fonts| fonts.layout_no_wrap(unit_text.to_owned(), font_id, VALUE_TEXT_COLOR));

        let unit_anchor = pos2(rect.left() + number_box_width, rect.center().y);
        let number_origin = Align2::RIGHT_CENTER
            .anchor_size(unit_anchor, number_galley.size())
            .min;
        let unit_origin = Align2::LEFT_CENTER
            .anchor_size(unit_anchor, unit_galley.size())
            .min;
        ui.painter()
            .galley(number_origin, number_galley, VALUE_TEXT_COLOR);
        ui.painter()
            .galley(unit_origin, unit_galley, VALUE_TEXT_COLOR);
        rect
    }

    fn apply_dark_style(context: &egui::Context) {
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = Color32::from_rgb(18, 18, 20);
        visuals.extreme_bg_color = Color32::from_rgb(28, 28, 32);
        context.set_visuals(visuals);
        context.style_mut(|style| {
            style.spacing.item_spacing = Vec2::new(16.0, 16.0);
        });
    }

    fn text_row_height(ui: &egui::Ui, font_size: f32) -> f32 {
        let font_id = Self::value_font_id(font_size);
        ui.fonts(|fonts| fonts.row_height(&font_id)).max(font_size)
    }

    fn text_width(ui: &egui::Ui, text: &str, font_size: f32) -> f32 {
        let font_id = Self::value_font_id(font_size);
        ui.fonts(|fonts| {
            text.chars()
                .map(|character| fonts.glyph_width(&font_id, character))
                .sum::<f32>()
        })
    }

    fn fixed_value_width(
        ui: &egui::Ui,
        font_size: f32,
        widest_number_template: &str,
        unit_text: &str,
    ) -> f32 {
        Self::value_group_geometry(ui, font_size, widest_number_template, unit_text).0
    }

    fn packed_column_height(ui: &egui::Ui, scale: f32) -> f32 {
        let padding = BASE_PADDING * scale;
        let label_row = Self::text_row_height(ui, LABEL_FONT_SIZE * scale);
        let value_row = Self::text_row_height(ui, VALUE_FONT_SIZE * scale);
        label_row + value_row + padding * 3.0 + BASE_MIN_SLIDER_LENGTH * scale
    }

    fn widest_control(ui: &egui::Ui, scale: f32) -> f32 {
        let label_size = LABEL_FONT_SIZE * scale;
        let value_size = VALUE_FONT_SIZE * scale;
        let labels = Self::text_width(ui, "Frequency", label_size).max(Self::text_width(
            ui,
            "Amplitude",
            label_size,
        ));
        let frequency =
            Self::fixed_value_width(ui, value_size, FREQUENCY_WIDEST_NUMBER_TEMPLATE, " Hz");
        let amplitude =
            Self::fixed_value_width(ui, value_size, AMPLITUDE_WIDEST_NUMBER_TEMPLATE, " dB");
        labels.max(frequency).max(amplitude)
    }

    /// Scale 1 keeps the portrait metrics (28px type, 16px padding, 80px minimum slider).
    /// Shorter or narrower windows scale those down. Scrolling is only the fallback when
    /// the floor scale still cannot fit the height.
    fn control_layout(ui: &egui::Ui, available: Vec2) -> ControlLayout {
        let column_gap = ui.spacing().item_spacing.x;
        let column_width = ((available.x - column_gap) / 2.0).max(1.0);
        let half_width = available.x * 0.5;
        let mut slider_thickness =
            (half_width * SLIDER_THICKNESS_FRACTION_OF_HALF).clamp(96.0, 200.0);
        if slider_thickness > column_width {
            slider_thickness = column_width;
        }
        let rail_upper = 36.0_f32.min(slider_thickness);
        let rail_lower = 12.0_f32.min(rail_upper);
        let slider_rail_height =
            (slider_thickness * SLIDER_RAIL_FRACTION_OF_THICKNESS).clamp(rail_lower, rail_upper);

        let height_ok = |scale: f32| Self::packed_column_height(ui, scale) <= available.y - 1.0;
        let width_ok = |scale: f32| Self::widest_control(ui, scale) <= column_width - 2.0;
        let fits = |scale: f32| height_ok(scale) && width_ok(scale);

        let (scale, needs_scroll) = if fits(1.0) {
            (1.0, false)
        } else if fits(MIN_LAYOUT_SCALE) {
            let mut low = MIN_LAYOUT_SCALE;
            let mut high = 1.0;
            for _ in 0..16 {
                let mid = (low + high) * 0.5;
                if fits(mid) {
                    low = mid;
                } else {
                    high = mid;
                }
            }
            (low, false)
        } else {
            let mut scale = MIN_LAYOUT_SCALE;
            if !width_ok(scale) {
                if width_ok(NARROW_LAYOUT_SCALE) {
                    let mut low = NARROW_LAYOUT_SCALE;
                    let mut high = MIN_LAYOUT_SCALE;
                    for _ in 0..12 {
                        let mid = (low + high) * 0.5;
                        if width_ok(mid) {
                            low = mid;
                        } else {
                            high = mid;
                        }
                    }
                    scale = low;
                } else {
                    scale = NARROW_LAYOUT_SCALE;
                }
            }
            (scale, !height_ok(scale))
        };

        let column_height = if needs_scroll {
            Some(Self::packed_column_height(ui, scale))
        } else {
            None
        };

        ControlLayout {
            label_font_size: LABEL_FONT_SIZE * scale,
            value_font_size: VALUE_FONT_SIZE * scale,
            padding: BASE_PADDING * scale,
            slider_thickness,
            slider_rail_height,
            handle_radius_min: 18.0 * scale,
            handle_radius_max: 48.0 * scale,
            column_height,
            needs_scroll,
        }
    }

    fn with_control_layout(
        ui: &mut egui::Ui,
        add_contents: impl FnOnce(&mut egui::Ui, &ControlLayout),
    ) {
        let available = ui.available_size();
        if available.x < 1.0 || available.y < 1.0 {
            return;
        }
        let layout = Self::control_layout(ui, available);
        if layout.needs_scroll {
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    let layout = Self::control_layout(ui, ui.available_size());
                    add_contents(ui, &layout);
                });
        } else {
            add_contents(ui, &layout);
        }
    }

    fn vertical_value_slider(
        ui: &mut egui::Ui,
        value: &mut f32,
        range_start: f32,
        range_end: f32,
        slider_thickness: f32,
        slider_length: f32,
        slider_rail_height: f32,
        handle_radius_min: f32,
        handle_radius_max: f32,
    ) -> egui::Response {
        let desired = Vec2::new(slider_thickness, slider_length);
        let (rect, mut response) = ui.allocate_exact_size(desired, Sense::click_and_drag());

        let max_radius_inside = (rect.height() * 0.5 - 1.0).max(1.0);
        let handle_min = handle_radius_min.min(max_radius_inside);
        let handle_max = handle_radius_max.min(max_radius_inside).max(handle_min);
        let handle_radius = (rect.width() / 2.5).clamp(handle_min, handle_max);
        let usable = rect.y_range().shrink(handle_radius);
        let span = (range_end - range_start).abs().max(f32::EPSILON);

        if let Some(pointer) = response.interact_pointer_pos() {
            let usable_span = usable.span().max(f32::EPSILON);
            let normalized_from_top = ((pointer.y - usable.min) / usable_span).clamp(0.0, 1.0);
            let normalized = 1.0 - normalized_from_top;
            *value = range_start + normalized * (range_end - range_start);
            response.mark_changed();
        }

        let normalized = ((*value - range_start) / span).clamp(0.0, 1.0);
        let handle_y = if usable.span() > 0.0 {
            usable.min + (1.0 - normalized) * usable.span()
        } else {
            rect.center().y
        };
        let handle_center = pos2(rect.center().x, handle_y);

        if ui.is_rect_visible(rect) {
            let rail_half = slider_rail_height * 0.5;
            let rail_rect = Rect::from_min_max(
                pos2(rect.center().x - rail_half, rect.top()),
                pos2(rect.center().x + rail_half, rect.bottom()),
            );
            ui.painter()
                .rect_filled(rail_rect, rail_half, SLIDER_RAIL_COLOR);

            let expansion = if response.dragged() {
                2.0
            } else if response.hovered() {
                1.0
            } else {
                0.0
            };
            let stroke_width = if response.dragged() || response.hovered() {
                2.0_f32
            } else {
                1.5_f32
            };
            ui.painter().add(Shape::circle_filled(
                handle_center,
                handle_radius + expansion,
                SLIDER_HANDLE_FILL,
            ));
            ui.painter().add(Shape::circle_stroke(
                handle_center,
                handle_radius + expansion,
                Stroke::new(stroke_width, SLIDER_HANDLE_STROKE),
            ));
        }

        response
    }

    fn paint_half_column(
        ui: &mut egui::Ui,
        layout: &ControlLayout,
        title_text: &str,
        number_text: String,
        unit_text: &str,
        widest_number_template: &str,
        value: &mut f32,
        range_start: f32,
        range_end: f32,
    ) -> egui::Response {
        let column_size = match layout.column_height {
            Some(height) => Vec2::new(ui.available_width(), height),
            None => ui.available_size(),
        };
        let mut slider_thickness = layout.slider_thickness.min(column_size.x.max(1.0));
        let rail_upper = 36.0_f32.min(slider_thickness);
        let rail_lower = 12.0_f32.min(rail_upper);
        let slider_rail_height = layout
            .slider_rail_height
            .clamp(rail_lower, rail_upper.max(rail_lower));
        ui.allocate_ui_with_layout(column_size, Layout::top_down(Align::Center), |ui| {
            ui.set_width(column_size.x);
            ui.set_min_height(column_size.y);
            ui.set_max_height(column_size.y);
            ui.add_space(layout.padding);
            ui.spacing_mut().item_spacing.y = 0.0;
            ui.label(
                RichText::new(title_text)
                    .size(layout.label_font_size)
                    .color(VALUE_TEXT_COLOR)
                    .strong(),
            );
            Self::paint_fixed_value_with_unit(
                ui,
                &number_text,
                unit_text,
                widest_number_template,
                layout.value_font_size,
            );
            ui.add_space(layout.padding);
            let bottom_padding = layout.padding;
            let slider_length = (ui.available_height() - bottom_padding).max(0.0);
            slider_thickness = slider_thickness.min(ui.available_width().max(1.0));
            Self::vertical_value_slider(
                ui,
                value,
                range_start,
                range_end,
                slider_thickness,
                slider_length,
                slider_rail_height,
                layout.handle_radius_min,
                layout.handle_radius_max,
            )
        })
        .inner
    }

    fn show_controls(&mut self, ui: &mut egui::Ui) {
        let frequency_hertz = self.frequency_hertz();
        Self::with_control_layout(ui, |ui, layout| {
            ui.columns(2, |columns| {
                let frequency_response = Self::paint_half_column(
                    &mut columns[0],
                    layout,
                    "Frequency",
                    Self::format_frequency_number(frequency_hertz),
                    " Hz",
                    FREQUENCY_WIDEST_NUMBER_TEMPLATE,
                    &mut self.frequency_log_normalized,
                    0.0,
                    1.0,
                );
                if frequency_response.changed() {
                    self.audio_engine
                        .set_frequency_hertz(self.frequency_hertz());
                }

                let amplitude_response = Self::paint_half_column(
                    &mut columns[1],
                    layout,
                    "Amplitude",
                    Self::format_amplitude_number(self.amplitude_decibels),
                    " dB",
                    AMPLITUDE_WIDEST_NUMBER_TEMPLATE,
                    &mut self.amplitude_decibels,
                    MINIMUM_AMPLITUDE_DECIBELS,
                    MAXIMUM_AMPLITUDE_DECIBELS,
                );
                if amplitude_response.changed() {
                    self.audio_engine
                        .set_amplitude_decibels(self.amplitude_decibels);
                }
            });
        });
    }
}

impl eframe::App for PureToneApp {
    fn update(&mut self, context: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.dark_style_applied {
            Self::apply_dark_style(context);
            self.dark_style_applied = true;
        }

        let frequency_hertz = self.frequency_hertz();
        self.audio_engine.set_frequency_hertz(frequency_hertz);
        self.audio_engine
            .set_amplitude_decibels(self.amplitude_decibels);

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(Color32::from_rgb(18, 18, 20)))
            .show(context, |ui| {
                self.show_controls(ui);
            });
    }
}

#[cfg(test)]
mod layout_tests {
    use super::{
        PureToneApp, AMPLITUDE_WIDEST_NUMBER_TEMPLATE, FREQUENCY_WIDEST_NUMBER_TEMPLATE,
        VALUE_FONT_SIZE,
    };
    use crate::audio_math::{
        frequency_hertz_from_log_normalized, MAXIMUM_AMPLITUDE_DECIBELS, MINIMUM_AMPLITUDE_DECIBELS,
    };
    use eframe::egui::{self, pos2, vec2, Color32, Rect};

    fn run_at(width: f32, height: f32, check: impl FnOnce(&mut egui::Ui)) {
        let context = egui::Context::default();
        let raw_input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(width, height))),
            ..Default::default()
        };
        let mut check = Some(check);
        let _ = context.run(raw_input, |context| {
            PureToneApp::apply_dark_style(context);
            egui::CentralPanel::default()
                .frame(egui::Frame::NONE.fill(Color32::from_rgb(18, 18, 20)))
                .show(context, |ui| {
                    if let Some(check) = check.take() {
                        check(ui);
                    }
                });
        });
    }

    #[test]
    fn portrait_keeps_full_size_metrics() {
        run_at(400.0, 800.0, |ui| {
            let available = ui.available_size();
            let layout = PureToneApp::control_layout(ui, available);
            assert!(!layout.needs_scroll);
            assert!(layout.column_height.is_none());
            assert!((layout.label_font_size - 28.0).abs() < 0.01);
            assert!((layout.value_font_size - 28.0).abs() < 0.01);
            assert!((layout.padding - 16.0).abs() < 0.01);
            assert!((layout.handle_radius_min - 18.0).abs() < 0.01);
            assert!((layout.handle_radius_max - 48.0).abs() < 0.01);
            let expected_thickness = (available.x * 0.5 * 0.50).clamp(96.0, 200.0);
            assert!((layout.slider_thickness - expected_thickness).abs() < 0.01);

            let mut frequency = 0.5_f32;
            let mut amplitude = -20.0_f32;
            let mut frequency_rect = Rect::NOTHING;
            let mut amplitude_rect = Rect::NOTHING;
            ui.columns(2, |columns| {
                frequency_rect = PureToneApp::paint_half_column(
                    &mut columns[0],
                    &layout,
                    "Frequency",
                    "440".to_string(),
                    " Hz",
                    FREQUENCY_WIDEST_NUMBER_TEMPLATE,
                    &mut frequency,
                    0.0,
                    1.0,
                )
                .rect;
                amplitude_rect = PureToneApp::paint_half_column(
                    &mut columns[1],
                    &layout,
                    "Amplitude",
                    "-20.0".to_string(),
                    " dB",
                    AMPLITUDE_WIDEST_NUMBER_TEMPLATE,
                    &mut amplitude,
                    MINIMUM_AMPLITUDE_DECIBELS,
                    MAXIMUM_AMPLITUDE_DECIBELS,
                )
                .rect;
            });
            assert!(
                frequency_rect.height() > 400.0,
                "{}",
                frequency_rect.height()
            );
            assert!(amplitude_rect.height() > 400.0);
            assert!(frequency_rect.center().x + 40.0 < amplitude_rect.center().x);
            assert!(frequency_rect.bottom() <= available.y + 1.0);
            assert!(amplitude_rect.bottom() <= available.y + 1.0);
        });
    }

    #[test]
    fn landscape_phone_keeps_both_sliders_side_by_side() {
        run_at(800.0, 360.0, |ui| {
            let available = ui.available_size();
            let layout = PureToneApp::control_layout(ui, available);
            assert!(
                !layout.needs_scroll,
                "360px-tall landscape should fit without scrolling"
            );
            assert!((layout.label_font_size - 28.0).abs() < 0.01);

            let mut frequency = 0.5_f32;
            let mut amplitude = -20.0_f32;
            let mut frequency_rect = Rect::NOTHING;
            let mut amplitude_rect = Rect::NOTHING;
            ui.columns(2, |columns| {
                frequency_rect = PureToneApp::paint_half_column(
                    &mut columns[0],
                    &layout,
                    "Frequency",
                    "440".to_string(),
                    " Hz",
                    FREQUENCY_WIDEST_NUMBER_TEMPLATE,
                    &mut frequency,
                    0.0,
                    1.0,
                )
                .rect;
                amplitude_rect = PureToneApp::paint_half_column(
                    &mut columns[1],
                    &layout,
                    "Amplitude",
                    "-20.0".to_string(),
                    " dB",
                    AMPLITUDE_WIDEST_NUMBER_TEMPLATE,
                    &mut amplitude,
                    MINIMUM_AMPLITUDE_DECIBELS,
                    MAXIMUM_AMPLITUDE_DECIBELS,
                )
                .rect;
            });
            assert!(
                frequency_rect.height() > 80.0,
                "{}",
                frequency_rect.height()
            );
            assert!(amplitude_rect.height() > 80.0);
            assert!(frequency_rect.center().x + 80.0 < amplitude_rect.center().x);
            assert!((frequency_rect.center().y - amplitude_rect.center().y).abs() < 2.0);
            assert!(frequency_rect.bottom() <= available.y + 1.0);
            assert!(amplitude_rect.right() <= available.x + 1.0);
        });
    }

    #[test]
    fn very_short_landscape_scrolls_instead_of_clipping() {
        run_at(900.0, 80.0, |ui| {
            let available = ui.available_size();
            let layout = PureToneApp::control_layout(ui, available);
            assert!(layout.needs_scroll);
            let packed = layout.column_height.expect("scroll content height");
            assert!(packed > available.y);
            assert!(layout.label_font_size < 28.0);
            assert!(layout.label_font_size + 0.01 >= 28.0 * 0.35);

            let mut frequency = 0.5_f32;
            let mut amplitude = -20.0_f32;
            let mut frequency_rect = Rect::NOTHING;
            let mut amplitude_rect = Rect::NOTHING;
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    let layout = PureToneApp::control_layout(ui, ui.available_size());
                    ui.columns(2, |columns| {
                        frequency_rect = PureToneApp::paint_half_column(
                            &mut columns[0],
                            &layout,
                            "Frequency",
                            "440".to_string(),
                            " Hz",
                            FREQUENCY_WIDEST_NUMBER_TEMPLATE,
                            &mut frequency,
                            0.0,
                            1.0,
                        )
                        .rect;
                        amplitude_rect = PureToneApp::paint_half_column(
                            &mut columns[1],
                            &layout,
                            "Amplitude",
                            "-20.0".to_string(),
                            " dB",
                            AMPLITUDE_WIDEST_NUMBER_TEMPLATE,
                            &mut amplitude,
                            MINIMUM_AMPLITUDE_DECIBELS,
                            MAXIMUM_AMPLITUDE_DECIBELS,
                        )
                        .rect;
                    });
                });
            assert!(
                frequency_rect.height() > 20.0,
                "{}",
                frequency_rect.height()
            );
            assert!(amplitude_rect.height() > 20.0);
            assert!(frequency_rect.center().x + 80.0 < amplitude_rect.center().x);
        });
    }

    #[test]
    fn extreme_values_fit_their_digit_slots() {
        let widest_frequency =
            PureToneApp::format_frequency_number(frequency_hertz_from_log_normalized(1.0));
        let lowest_frequency =
            PureToneApp::format_frequency_number(frequency_hertz_from_log_normalized(0.0));
        let widest_amplitude = PureToneApp::format_amplitude_number(MINIMUM_AMPLITUDE_DECIBELS);
        assert_eq!(widest_frequency, "20000");
        assert_eq!(lowest_frequency, "20");
        assert_eq!(widest_amplitude, "-100.0");
        assert_formatted_number_fits_template(&widest_frequency, FREQUENCY_WIDEST_NUMBER_TEMPLATE);
        assert_formatted_number_fits_template(&lowest_frequency, FREQUENCY_WIDEST_NUMBER_TEMPLATE);
        assert_formatted_number_fits_template(&widest_amplitude, AMPLITUDE_WIDEST_NUMBER_TEMPLATE);
        assert_eq!(
            widest_frequency.chars().count(),
            FREQUENCY_WIDEST_NUMBER_TEMPLATE.chars().count()
        );
        assert_eq!(
            widest_amplitude.chars().count(),
            AMPLITUDE_WIDEST_NUMBER_TEMPLATE.chars().count()
        );
        assert!(
            lowest_frequency.chars().count() < FREQUENCY_WIDEST_NUMBER_TEMPLATE.chars().count()
        );

        run_at(400.0, 800.0, |ui| {
            let frequency_box_width = PureToneApp::number_box_width(
                ui,
                VALUE_FONT_SIZE,
                FREQUENCY_WIDEST_NUMBER_TEMPLATE,
            );
            let amplitude_box_width = PureToneApp::number_box_width(
                ui,
                VALUE_FONT_SIZE,
                AMPLITUDE_WIDEST_NUMBER_TEMPLATE,
            );
            for number_text in ["20", "440", "19999", "20000"] {
                let text_width = PureToneApp::text_width(ui, number_text, VALUE_FONT_SIZE);
                assert!(
                    text_width <= frequency_box_width,
                    "{number_text} width {text_width} exceeds frequency box {frequency_box_width}"
                );
            }
            for number_text in ["-100.0", "-99.9", "-0.0", "0.0"] {
                let text_width = PureToneApp::text_width(ui, number_text, VALUE_FONT_SIZE);
                assert!(
                    text_width <= amplitude_box_width,
                    "{number_text} width {text_width} exceeds amplitude box {amplitude_box_width}"
                );
            }
        });
    }

    #[test]
    fn value_group_stays_fixed_and_centered_for_every_value() {
        run_at(400.0, 800.0, |ui| {
            let layout = PureToneApp::control_layout(ui, ui.available_size());
            ui.columns(2, |columns| {
                place_values_and_assert_fixed_group(
                    &mut columns[0],
                    layout.value_font_size,
                    FREQUENCY_WIDEST_NUMBER_TEMPLATE,
                    " Hz",
                    &["20000", "440", "20"],
                );
                place_values_and_assert_fixed_group(
                    &mut columns[1],
                    layout.value_font_size,
                    AMPLITUDE_WIDEST_NUMBER_TEMPLATE,
                    " dB",
                    &["-100.0", "-5.0", "0.0"],
                );
            });
        });
    }

    /// Right-align `number_text` into `template`. A `'0'` slot must hold a digit.
    /// Any other template character must appear in the formatted text.
    fn assert_formatted_number_fits_template(number_text: &str, template: &str) {
        let number_characters: Vec<char> = number_text.chars().collect();
        let template_characters: Vec<char> = template.chars().collect();
        assert!(
            number_characters.len() <= template_characters.len(),
            "{number_text} is longer than template {template}"
        );
        let leading_slots = template_characters.len() - number_characters.len();
        for (offset, number_character) in number_characters.iter().enumerate() {
            let template_character = template_characters[leading_slots + offset];
            if template_character == '0' {
                assert!(
                    number_character.is_ascii_digit(),
                    "{number_text} places {number_character} in a digit slot of {template}"
                );
            } else {
                assert_eq!(
                    *number_character, template_character,
                    "{number_text} does not match the shape of {template}"
                );
            }
        }
    }

    fn place_values_and_assert_fixed_group(
        column: &mut egui::Ui,
        value_font_size: f32,
        widest_number_template: &str,
        unit_text: &str,
        number_texts: &[&str],
    ) {
        let column_size = column.available_size();
        column.allocate_ui_with_layout(
            column_size,
            egui::Layout::top_down(egui::Align::Center),
            |column| {
                column.set_width(column_size.x);
                let column_center_x = column.max_rect().center().x;
                let (group_width, number_box_width) = PureToneApp::value_group_geometry(
                    column,
                    value_font_size,
                    widest_number_template,
                    unit_text,
                );
                let mut placed_groups: Vec<(Rect, f32)> = Vec::new();
                for number_text in number_texts {
                    let group_rect = PureToneApp::paint_fixed_value_with_unit(
                        column,
                        number_text,
                        unit_text,
                        widest_number_template,
                        value_font_size,
                    );
                    let unit_left_x = group_rect.left() + number_box_width;
                    assert!(
                        (group_rect.center().x - column_center_x).abs() <= 0.5,
                        "{number_text}: group center {group_center} column center {column_center_x}",
                        group_center = group_rect.center().x
                    );
                    assert!(
                        (group_rect.width() - group_width).abs() <= 0.5,
                        "{number_text}: painted width {painted_width} geometry {group_width}",
                        painted_width = group_rect.width()
                    );
                    placed_groups.push((group_rect, unit_left_x));
                }
                let (first_rect, first_unit_left_x) = placed_groups[0];
                for (group_rect, unit_left_x) in &placed_groups[1..] {
                    assert_eq!(group_rect.left(), first_rect.left());
                    assert_eq!(group_rect.right(), first_rect.right());
                    assert_eq!(group_rect.width(), first_rect.width());
                    assert_eq!(*unit_left_x, first_unit_left_x);
                }
            },
        );
    }

    #[test]
    fn widest_values_fit_full_size_in_portrait_and_landscape() {
        for (width, height) in [(360.0, 740.0), (400.0, 800.0), (800.0, 360.0)] {
            run_at(width, height, |ui| {
                let available = ui.available_size();
                let layout = PureToneApp::control_layout(ui, available);
                assert!(!layout.needs_scroll, "{width}x{height}");
                assert!(
                    (layout.value_font_size - 28.0).abs() < 0.01,
                    "{width}x{height}: {}",
                    layout.value_font_size
                );
                let column_width = (available.x - ui.spacing().item_spacing.x) / 2.0;
                let widest = PureToneApp::widest_control(ui, 1.0);
                assert!(
                    widest <= column_width - 2.0,
                    "{width}x{height}: {widest} > {column_width}"
                );

                let mut frequency = 1.0_f32;
                let mut amplitude = MINIMUM_AMPLITUDE_DECIBELS;
                let mut frequency_rect = Rect::NOTHING;
                let mut amplitude_rect = Rect::NOTHING;
                ui.columns(2, |columns| {
                    frequency_rect = PureToneApp::paint_half_column(
                        &mut columns[0],
                        &layout,
                        "Frequency",
                        "20000".to_string(),
                        " Hz",
                        FREQUENCY_WIDEST_NUMBER_TEMPLATE,
                        &mut frequency,
                        0.0,
                        1.0,
                    )
                    .rect;
                    amplitude_rect = PureToneApp::paint_half_column(
                        &mut columns[1],
                        &layout,
                        "Amplitude",
                        "-100.0".to_string(),
                        " dB",
                        AMPLITUDE_WIDEST_NUMBER_TEMPLATE,
                        &mut amplitude,
                        MINIMUM_AMPLITUDE_DECIBELS,
                        MAXIMUM_AMPLITUDE_DECIBELS,
                    )
                    .rect;
                });
                assert!(frequency_rect.right() <= amplitude_rect.left() + 1.0);
                assert!(amplitude_rect.right() <= available.x + 1.0);
            });
        }
    }
}
