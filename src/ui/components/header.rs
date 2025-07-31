use chrono::{DateTime, Utc};

use crate::{
    domain::peer::Peer,
    utils::{
        text::PrettyStr,
        time::{clock, ts_to_str},
    },
};

pub struct Header {}

impl Header {
    pub fn new() -> Self {
        Self {}
    }

    pub fn show(
        &mut self,
        ui: &mut eframe::egui::Ui,
        local_peer: &Peer,
        current_time: DateTime<Utc>,
    ) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(eframe::egui::RichText::new("📡 DTChat").size(20.0).strong());

                ui.label(eframe::egui::RichText::new("Delay-Tolerant Messaging").size(10.5));
                ui.add_space(10.0);

                let mins = chrono::Timelike::minute(&current_time);
                let hours = chrono::Timelike::hour(&current_time);
                let clock = clock(hours, mins);

                ui.label(
                    eframe::egui::RichText::new(format!(
                        "\u{1F4C5} {} ",
                        ts_to_str(
                            &current_time,
                            true,
                            true,
                            Some(format!(" {} ", clock).to_string())
                        )
                    ))
                    .size(12.0)
                    .strong(),
                );
            });

            ui.allocate_ui_with_layout(
                ui.available_size(),
                eframe::egui::Layout::right_to_left(eframe::egui::Align::RIGHT),
                |ui| {
                    ui.allocate_ui_with_layout(
                        ui.available_size(),
                        eframe::egui::Layout::top_down(eframe::egui::Align::Max), // Align to right (Max = right)
                        |ui| {
                            ui.label(
                                eframe::egui::RichText::new(&format!("👤 {}", local_peer.name))
                                    .size(12.0)
                                    .strong(),
                            );
                            ui.add_space(5.0);
                            for endpoint in &local_peer.endpoints {
                                let protocol_text = format!("{}", endpoint.to_pretty_str());
                                ui.label(eframe::egui::RichText::new(&protocol_text).size(10.5));
                            }
                        },
                    );
                },
            );
        });

        ui.add_space(5.0);
    }
}
