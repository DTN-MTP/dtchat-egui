use dtchat_backend::time::DTChatTime;

use crate::{
    domain::peer::Peer,
    utils::{text::PrettyStr, time::clock},
};

pub struct Header {
    minutes: u32,
    clock: String,
}

impl Header {
    pub fn new() -> Self {
        Self {
            minutes: 100,
            clock: String::new(),
        }
    }

    pub fn show(&mut self, ui: &mut eframe::egui::Ui, local_peer: &Peer, current_time: DTChatTime) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(eframe::egui::RichText::new("📡 DTChat").size(20.0).strong());

                ui.label(eframe::egui::RichText::new("Delay-Tolerant Messaging").size(10.5));
                ui.add_space(10.0);

                let (mins, hours) = current_time.mins_hours(&chrono::Local);
                if self.minutes != mins {
                    self.minutes = mins;
                    self.clock = format!(" {} ", clock(hours, mins));
                }

                ui.label(
                    eframe::egui::RichText::new(format!(
                        "\u{1F4C5} {} ",
                        &current_time.ts_to_str(true, true, Some(&self.clock), &chrono::Local)
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
