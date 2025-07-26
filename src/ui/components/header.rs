use crate::domain::peer::Peer;

pub struct Header {}

impl Header {
    pub fn new() -> Self {
        Self {}
    }

    pub fn show(&mut self, ui: &mut eframe::egui::Ui, local_peer: &Peer) {
        ui.add_space(5.0);

        // Header section with title and peer info in a horizontal layout
        ui.horizontal(|ui| {
            // Left side: Application title and description
            ui.vertical(|ui| {
                // Application title with emphasis (aligned left)
                ui.label(eframe::egui::RichText::new("📡 DTChat").size(20.0).strong());

                // Application description
                ui.label(
                    eframe::egui::RichText::new("Delay Tolerant Messaging")
                        .size(10.0)
                        .color(eframe::egui::Color32::from_rgb(120, 120, 120)),
                );
            });

            // Add expanding space to push the peer info to the right
            ui.allocate_ui_with_layout(
                ui.available_size(),
                eframe::egui::Layout::right_to_left(eframe::egui::Align::Center),
                |ui| {
                    // Vertical layout for peer name and protocols (aligned to the right)
                    ui.vertical(|ui| {
                        // Peer name (right aligned)
                        ui.with_layout(
                            eframe::egui::Layout::right_to_left(eframe::egui::Align::Center),
                            |ui| {
                                ui.label(
                                    eframe::egui::RichText::new(&format!("👤 {}", local_peer.name))
                                        .size(12.0)
                                        .strong(),
                                );
                            },
                        );

                        // Available protocols with addresses (right aligned)
                        for endpoint in &local_peer.endpoints {
                            ui.with_layout(
                                eframe::egui::Layout::right_to_left(eframe::egui::Align::Center),
                                |ui| {
                                    let protocol_text =
                                        format!("{} - {}", endpoint.proto, endpoint.endpoint);
                                    ui.label(
                                        eframe::egui::RichText::new(&protocol_text)
                                            .size(10.0)
                                            .color(eframe::egui::Color32::from_rgb(100, 100, 100)),
                                    );
                                },
                            );
                        }
                    });
                },
            );
        });

        ui.add_space(5.0);
    }
}
