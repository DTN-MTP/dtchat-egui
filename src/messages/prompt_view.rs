use crate::utils::text::PrettyStr;
use dtchat_backend::dtchat::{ChatModel, Peer, Room};
use dtchat_backend::Endpoint;
use eframe::egui;
use egui::{ComboBox, RichText};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct MessagePromptView {
    pub input_text: String,
    pub selected_peer: Option<Peer>,
    pub selected_endpoint: Option<Endpoint>,
    pub pbat_enabled: bool,
}

impl MessagePromptView {
    pub fn new() -> Self {
        Self {
            input_text: String::new(),
            selected_peer: None,
            selected_endpoint: None, // Default TCP
            pbat_enabled: false,
        }
    }

    pub fn select_first_endpoint(&mut self) {
        if let Some(peer) = &self.selected_peer {
            if peer.endpoints.len() > 0 {
                self.selected_endpoint = Some(peer.endpoints[0].clone());
            }
        }
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        peers: &HashMap<String, Peer>,
        chat_model: &Arc<Mutex<ChatModel>>,
        pbat_support_by_model: bool,
        current_room: &Option<Room>,
    ) {
        if self.selected_peer.is_none() && !peers.is_empty() {
            let p = peers.iter().next().unwrap().1;
            self.selected_peer = Some(p.clone());
        }
        if self.selected_endpoint.is_none() {
            self.select_first_endpoint();
        }

        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.label("To:");
            let selected_peer = self.selected_peer.clone();

            ui.add_enabled_ui(peers.len() > 0, |ui| {
                ComboBox::from_id_salt("peer_selector")
                    .selected_text(
                        selected_peer
                            .as_ref()
                            .map(|p| p.name.clone())
                            .unwrap_or_else(|| "⚠ no peers".to_string()),
                    )
                    .show_ui(ui, |ui| {
                        for (peer_uuid, peer) in peers {
                            if ui
                                .selectable_label(
                                    selected_peer.as_ref().map(|p| &p.uuid) == Some(&peer_uuid),
                                    peer.name.clone(),
                                )
                                .clicked()
                            {
                                self.selected_peer = Some((*peer).clone());
                                self.select_first_endpoint();
                            }
                        }
                    });
            });

            ui.label("Protocol:");

            if let Some(peer) = selected_peer {
                let selected_text = match &self.selected_endpoint {
                    Some(endpoint) => endpoint.to_pretty_str(),
                    None => "⚠ no endpoints".to_string(),
                };
                ui.add_enabled_ui(self.selected_endpoint.is_some(), |ui| {
                    ComboBox::from_id_salt("protocol_selector")
                        .selected_text(selected_text.clone())
                        .show_ui(ui, |ui| {
                            for endpoint in &peer.endpoints {
                                let is_selected = selected_text == *endpoint.to_pretty_str();
                                if ui
                                    .selectable_label(is_selected, endpoint.to_pretty_str())
                                    .clicked()
                                {
                                    self.selected_endpoint = Some(endpoint.clone());
                                }
                            }
                        });
                });
            } else {
                ui.add_enabled_ui(false, |ui| {
                    ComboBox::from_id_salt("protocol_selector")
                        .selected_text("⚠ no peers".to_string())
                        .show_ui(ui, |_ui| {});
                });
            }
        });
        ui.add_space(8.0);

        ui.horizontal(|ui| {
            let text_edit = egui::TextEdit::singleline(&mut self.input_text)
                .hint_text("Type your message...")
                .desired_width(ui.available_width() - 80.0)
                .margin(egui::Margin::same(6));

            let response = ui.add(text_edit);

            let send_button = egui::Button::new(RichText::new("Send 📤").strong())
                // .fill(egui::Color32::from_rgb(42, 124, 190))
                .corner_radius(4.0)
                .min_size(egui::Vec2::new(60.0, 24.0));

            let should_send = ui.add(send_button).clicked()
                || (response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)));

            if should_send {
                if let Some(peer) = &self.selected_peer {
                    let content = self.input_text.trim();
                    if !content.is_empty() {
                        if let Some(endpoint) = self.selected_endpoint.clone() {
                            if let Ok(mut model) = chat_model.lock() {
                                if let Some(room) = current_room {
                                    model.send_to_room(
                                        &content.to_string(),
                                        &room.uuid,
                                        self.pbat_enabled,
                                    );
                                } else {
                                    model.send_to_peer(
                                        &content.to_string(),
                                        &"".to_string(),
                                        peer.uuid.clone(),
                                        &endpoint,
                                        self.pbat_enabled,
                                    );
                                }
                                self.input_text.clear();
                            }
                        }
                    }
                }
                response.request_focus();
            }
        });

        ui.add_space(4.0);

        ui.horizontal(|ui| {
            self.pbat_enabled = pbat_support_by_model && self.pbat_enabled;

            ui.add_enabled(
                pbat_support_by_model,
                egui::Checkbox::new(
                    &mut self.pbat_enabled,
                    " 🔭 Arrival Time Prediction (A-SABR)",
                ),
            )
            .on_disabled_hover_text("The CP_PATH env variable must be set before starting the app");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if let Some(peer) = &self.selected_peer {
                    if let Some(endpoint) = self.selected_endpoint.clone() {
                        ui.colored_label(
                            egui::Color32::GRAY,
                            format!("to {} via {}", peer.name, endpoint.to_pretty_str()),
                        );
                    }
                }
            });
        });
    }
}
