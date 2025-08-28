use dtchat_backend::{dtchat::Peer, time::DTChatTime};

use crate::utils::{clock::Clock, text::PrettyStr};

pub struct HeaderView {
    clock: Clock,
}

impl HeaderView {
    pub fn new() -> Self {
        Self {
            clock: Clock::new(&DTChatTime::now(), false),
        }
    }

    pub fn show(&mut self, ui: &mut eframe::egui::Ui, local_peer: &Peer, current_time: DTChatTime) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(eframe::egui::RichText::new("📡 DTChat").size(20.0).strong());

                ui.label(eframe::egui::RichText::new("Delay-Tolerant Messaging").size(10.5));
                ui.add_space(10.0);

                self.clock.update(&current_time);

                if ui
                    .label(
                        eframe::egui::RichText::new(format!(
                            "\u{1F4C5} {} ",
                            &current_time.ts_to_str(
                                true,
                                true,
                                Some(format!(" {} ", &self.clock.to_string()).as_str()),
                                &chrono::Local
                            )
                        ))
                        .size(12.0)
                        .strong(),
                    )
                    .clicked()
                {
                    self.clock.switch_anim(&current_time);
                }
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
