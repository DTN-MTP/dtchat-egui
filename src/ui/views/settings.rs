use crate::app::{DisplayEvent, EventLevel};
use eframe::egui::{self, Color32, ScrollArea};
use std::collections::VecDeque;

pub struct SettingsView;

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
        ui.vertical(|ui| {
            // Section des événements réseau (en haut) avec ID unique
            ui.push_id("network_events_section", |ui| {
                ui.group(|ui| {
                    ui.heading("Network Events");
                    ui.separator();

                    ScrollArea::vertical()
                        .id_salt("network_scroll")
                        .max_height(200.0)
                        .show(ui, |ui| {
                            if network_events.is_empty() {
                                ui.label("No network events");
                            } else {
                                // Show newest events first
                                for (index, event) in network_events.iter().rev().enumerate() {
                                    ui.push_id(format!("network_event_{}", index), |ui| {
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

            ui.add_space(10.0);

            // Section des événements application (en bas) avec ID unique
            ui.push_id("app_events_section", |ui| {
                ui.group(|ui| {
                    ui.heading("App Events");
                    ui.separator();

                    ScrollArea::vertical()
                        .id_salt("app_scroll")
                        .max_height(200.0)
                        .show(ui, |ui| {
                            if app_events.is_empty() {
                                ui.label("No app events");
                            } else {
                                // Show newest events first
                                for (index, event) in app_events.iter().rev().enumerate() {
                                    ui.push_id(format!("app_event_{}", index), |ui| {
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
}
