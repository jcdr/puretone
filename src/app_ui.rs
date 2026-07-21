use crate::audio_engine::SineAudioEngine;
use crate::audio_math::{
    frequency_hertz_from_log_normalized, log_normalized_from_frequency_hertz,
    MAXIMUM_AMPLITUDE_DECIBELS, MINIMUM_AMPLITUDE_DECIBELS,
};
use eframe::egui::{
    self, pos2, Align, Color32, FontFamily, FontId, Layout, Rect, RichText, Sense, Shape, Stroke,
    Vec2,
};

const DEFAULT_FREQUENCY_HERTZ: f32 = 440.0;
const DEFAULT_AMPLITUDE_DECIBELS: f32 = -20.0;
const LABEL_FONT_SIZE: f32 = 28.0;
const VALUE_FONT_SIZE: f32 = 28.0;
const FREQUENCY_NUMBER_DIGIT_SLOTS: usize = 4;
const AMPLITUDE_NUMBER_DIGIT_SLOTS: usize = 5;
const SLIDER_THICKNESS_FRACTION_OF_HALF: f32 = 0.50;
const SLIDER_RAIL_FRACTION_OF_THICKNESS: f32 = 0.22;
const SLIDER_RAIL_COLOR: Color32 = Color32::from_rgb(60, 60, 60);
const SLIDER_HANDLE_FILL: Color32 = Color32::WHITE;
const SLIDER_HANDLE_STROKE: Color32 = Color32::from_rgb(220, 220, 230);
const VALUE_TEXT_COLOR: Color32 = Color32::from_rgb(230, 230, 235);

pub struct PureToneApp {
    frequency_log_normalized: f32,
    amplitude_decibels: f32,
    audio_engine: SineAudioEngine,
    dark_style_applied: bool,
}

impl Default for PureToneApp {
    fn default() -> Self {
        Self::new()
    }
}

impl PureToneApp {
    pub fn new() -> Self {
        let frequency_log_normalized =
            log_normalized_from_frequency_hertz(DEFAULT_FREQUENCY_HERTZ);
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

    fn value_font_id() -> FontId {
        FontId::new(VALUE_FONT_SIZE, FontFamily::Proportional)
    }

    fn format_frequency_number(frequency_hertz: f32) -> String {
        format!("{:.0}", frequency_hertz)
    }

    fn format_amplitude_number(amplitude_decibels: f32) -> String {
        format!("{:.1}", amplitude_decibels)
    }

    fn value_figure_width(ui: &egui::Ui, font_id: &FontId) -> f32 {
        ui.fonts(|fonts| {
            "0123456789.-"
                .chars()
                .map(|character| fonts.glyph_width(font_id, character))
                .fold(0.0_f32, f32::max)
        })
    }

    fn paint_fixed_value_with_unit(
        ui: &mut egui::Ui,
        number_text: &str,
        unit_text: &str,
        number_digit_slots: usize,
    ) {
        let font_id = Self::value_font_id();
        let figure_width = Self::value_figure_width(ui, &font_id);
        let number_slot_width = figure_width * number_digit_slots as f32;
        let unit_width = ui.fonts(|fonts| {
            unit_text
                .chars()
                .map(|character| fonts.glyph_width(&font_id, character))
                .sum::<f32>()
        });
        let row_height = ui.fonts(|fonts| fonts.row_height(&font_id));
        let total_width = number_slot_width + unit_width;

        ui.allocate_ui_with_layout(
            Vec2::new(total_width, row_height),
            Layout::left_to_right(Align::Center),
            |ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.allocate_ui_with_layout(
                    Vec2::new(number_slot_width, row_height),
                    Layout::right_to_left(Align::Center),
                    |ui| {
                        ui.set_min_width(number_slot_width);
                        ui.set_max_width(number_slot_width);
                        ui.label(
                            RichText::new(number_text)
                                .font(font_id.clone())
                                .color(VALUE_TEXT_COLOR)
                                .strong(),
                        );
                    },
                );
                ui.label(
                    RichText::new(unit_text)
                        .font(font_id)
                        .color(VALUE_TEXT_COLOR)
                        .strong(),
                );
            },
        );
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

    fn vertical_value_slider(
        ui: &mut egui::Ui,
        value: &mut f32,
        range_start: f32,
        range_end: f32,
        slider_thickness: f32,
        slider_length: f32,
        slider_rail_height: f32,
    ) -> egui::Response {
        let desired = Vec2::new(slider_thickness, slider_length);
        let (rect, mut response) = ui.allocate_exact_size(desired, Sense::click_and_drag());

        let handle_radius = (rect.width() / 2.5).clamp(18.0, 48.0);
        let usable = rect.y_range().shrink(handle_radius);
        let span = (range_end - range_start).abs().max(f32::EPSILON);

        if let Some(pointer) = response.interact_pointer_pos() {
            let normalized_from_top = ((pointer.y - usable.min) / usable.span()).clamp(0.0, 1.0);
            let normalized = 1.0 - normalized_from_top;
            *value = range_start + normalized * (range_end - range_start);
            response.mark_changed();
        }

        let normalized = ((*value - range_start) / span).clamp(0.0, 1.0);
        let handle_y = usable.min + (1.0 - normalized) * usable.span();
        let handle_center = pos2(rect.center().x, handle_y);

        if ui.is_rect_visible(rect) {
            let rail_half = slider_rail_height * 0.5;
            let rail_rect = Rect::from_min_max(
                pos2(rect.center().x - rail_half, rect.top()),
                pos2(rect.center().x + rail_half, rect.bottom()),
            );
            ui.painter().rect_filled(rail_rect, rail_half, SLIDER_RAIL_COLOR);

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
        title_text: &str,
        number_text: String,
        unit_text: &str,
        number_digit_slots: usize,
        slider_thickness: f32,
        slider_rail_height: f32,
        value: &mut f32,
        range_start: f32,
        range_end: f32,
    ) -> egui::Response {
        let column_size = ui.available_size();
        ui.allocate_ui_with_layout(column_size, Layout::top_down(Align::Center), |ui| {
            ui.set_width(column_size.x);
            ui.set_min_height(column_size.y);
            ui.set_max_height(column_size.y);
            ui.add_space(16.0);
            ui.spacing_mut().item_spacing.y = 0.0;
            ui.label(
                RichText::new(title_text)
                    .size(LABEL_FONT_SIZE)
                    .color(VALUE_TEXT_COLOR)
                    .strong(),
            );
            Self::paint_fixed_value_with_unit(ui, &number_text, unit_text, number_digit_slots);
            ui.add_space(16.0);
            let bottom_padding = 16.0;
            let slider_length = (ui.available_height() - bottom_padding).max(80.0);
            Self::vertical_value_slider(
                ui,
                value,
                range_start,
                range_end,
                slider_thickness,
                slider_length,
                slider_rail_height,
            )
        })
        .inner
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
                let available = ui.available_size();
                let half_width = available.x * 0.5;
                let slider_thickness =
                    (half_width * SLIDER_THICKNESS_FRACTION_OF_HALF).clamp(96.0, 200.0);
                let slider_rail_height =
                    (slider_thickness * SLIDER_RAIL_FRACTION_OF_THICKNESS).clamp(12.0, 36.0);

                ui.columns(2, |columns| {
                    let frequency_response = Self::paint_half_column(
                        &mut columns[0],
                        "Frequency",
                        Self::format_frequency_number(frequency_hertz),
                        " Hz",
                        FREQUENCY_NUMBER_DIGIT_SLOTS,
                        slider_thickness,
                        slider_rail_height,
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
                        "Amplitude",
                        Self::format_amplitude_number(self.amplitude_decibels),
                        " dB",
                        AMPLITUDE_NUMBER_DIGIT_SLOTS,
                        slider_thickness,
                        slider_rail_height,
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
