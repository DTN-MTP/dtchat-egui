use crate::app::{DisplayEvent, EventLevel};
use eframe::egui::{self, Color32, ScrollArea};
use std::collections::VecDeque;

pub struct SettingsView;

fn show_events_in_columns(ui: &mut egui::Ui, heading: &str, app_events: &VecDeque<DisplayEvent>) {
    ui.push_id(format!("{}_section", heading), |ui| {
        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.heading(heading);
                ui.separator();

                ScrollArea::vertical()
                    .id_salt(heading)
                    .auto_shrink([false; 2])
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        if app_events.is_empty() {
                            ui.label("No events");
                        } else {
                            // Show newest events first
                            for (index, event) in app_events.iter().enumerate() {
                                ui.push_id(format!("{}_{}", heading, index), |ui| {
                                    let color = match event.level {
                                        EventLevel::Error => Color32::RED,
                                        EventLevel::Info => Color32::LIGHT_BLUE,
                                        EventLevel::Debug => Color32::GRAY,
                                    };
                                    ui.colored_label(
                                        color,
                                        format!(
                                            "[{}] {}",
                                            event.timestamp.format("%H:%M:%S"),
                                            event.message
                                        ),
                                    );
                                });
                            }
                        }
                    });
            });
        });
    });
}

impl SettingsView {
    pub fn new() -> Self {
        Self
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        network_events: &VecDeque<DisplayEvent>,
        app_events: &VecDeque<DisplayEvent>,
    ) {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
            let available_width = ui.available_width();
            let available_height = ui.available_height();

            // Force the horizontal layout to take the full height
            ui.allocate_ui_with_layout(
                egui::vec2(available_width, available_height),
                egui::Layout::left_to_right(egui::Align::TOP),
                |ui| {
                    // First vertical block
                    ui.allocate_ui(egui::vec2(available_width * 0.45, available_height), |ui| {
                        show_events_in_columns(ui, "App Events", app_events);
                    });
                    // Second vertical block
                    ui.allocate_ui(egui::vec2(available_width * 0.55, available_height), |ui| {
                        show_events_in_columns(ui, "Network Events", network_events);
                    });
                },
            );
        });
    }
}
