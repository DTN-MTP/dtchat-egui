use crate::domain::peer::{Peer, PeerManager};
use dtchat_backend::dtchat::ChatModel;
use eframe::egui;
use egui::ComboBox;
use socket_engine::endpoint::{Endpoint, EndpointProto};
use std::sync::{Arc, Mutex};

pub struct MessageForge {
    pub input_text: String,
    pub selected_peer: Option<Peer>,
    pub selected_protocol: EndpointProto,
    pub pbat_enabled: bool,
}

impl MessageForge {
    pub fn new() -> Self {
        Self {
            input_text: String::new(),
            selected_peer: None,
            selected_protocol: EndpointProto::Tcp, // Default TCP
            pbat_enabled: false,
        }
    }

    fn get_available_protocols(&self) -> Vec<EndpointProto> {
        if let Some(peer) = &self.selected_peer {
            peer.get_available_protocols()
        } else {
            vec![]
        }
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        peers: &[Peer],
        local_peer_uuid: &str,
        chat_model: &Arc<Mutex<ChatModel>>,
        peer_manager: &PeerManager,
    ) {
        let available_peers: Vec<&Peer> = peers
            .iter()
            .filter(|peer| peer.uuid != local_peer_uuid)
            .collect();

        if self.selected_peer.is_none() && !available_peers.is_empty() {
            self.selected_peer = Some(available_peers[0].clone());
        }

        if self.selected_peer.is_some() {
            let available_protocols = self.get_available_protocols();

            if !available_protocols.contains(&self.selected_protocol)
                && !available_protocols.is_empty()
            {
                self.selected_protocol = available_protocols[0].clone();
            }
        }

        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.label("To:");
            let selected_peer = self.selected_peer.clone();
            ComboBox::from_id_salt("peer_selector")
                .selected_text(
                    selected_peer
                        .as_ref()
                        .map(|p| p.name.clone())
                        .unwrap_or_else(|| "Select peer".to_string()),
                )
                .show_ui(ui, |ui| {
                    for peer in &available_peers {
                        if ui
                            .selectable_label(
                                selected_peer.as_ref().map(|p| &p.uuid) == Some(&peer.uuid),
                                peer.name.clone(),
                            )
                            .clicked()
                        {
                            self.selected_peer = Some((*peer).clone());
                        }
                    }
                });
            ui.label("Protocol:");
            let available_protocols = self.get_available_protocols();
            ComboBox::from_id_salt("protocol_selector")
                .selected_text(self.selected_protocol.to_string())
                .show_ui(ui, |ui| {
                    for protocol in &available_protocols {
                        let is_selected = self.selected_protocol == *protocol;
                        if ui
                            .selectable_label(is_selected, protocol.to_string())
                            .clicked()
                        {
                            self.selected_protocol = protocol.clone();
                        }
                    }
                });
        });

        ui.add_space(8.0);

        ui.horizontal(|ui| {
            let text_edit = egui::TextEdit::singleline(&mut self.input_text)
                .hint_text("Type your message...")
                .desired_width(ui.available_width() - 80.0)
                .margin(egui::Margin::same(6));

            let response = ui.add(text_edit);

            let send_button = egui::Button::new("Send")
                .fill(egui::Color32::from_rgb(52, 144, 220))
                .corner_radius(4.0)
                .min_size(egui::Vec2::new(60.0, 28.0));

            let should_send = ui.add(send_button).clicked()
                || (response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)));

            if should_send {
                if let Some(peer) = &self.selected_peer {
                    let content = self.input_text.trim();
                    if !content.is_empty() {
                        if let Some(endpoint) = peer_manager.find_endpoint_for_peer_with_protocol(
                            &peer.uuid,
                            self.selected_protocol.clone(),
                        ) {
                            if let Ok(mut model) = chat_model.lock() {
                                model.send_to_peer(
                                    &content.to_string(),
                                    &"room".to_string(),
                                    &endpoint,
                                );
                                self.input_text.clear();
                            }
                        }
                    }
                }
            }
        });

        ui.add_space(4.0);

        ui.horizontal(|ui| {
            ui.checkbox(&mut self.pbat_enabled, "Enable PBAT");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if let Some(peer) = &self.selected_peer {
                    if let Some(endpoint) = peer_manager.find_endpoint_for_peer_with_protocol(
                        &peer.uuid,
                        self.selected_protocol.clone(),
                    ) {
                        let address = endpoint.endpoint;
                        ui.colored_label(
                            egui::Color32::GRAY,
                            format!("{} via {} ({})", peer.name, self.selected_protocol, address),
                        );
                    } else {
                        ui.colored_label(
                            egui::Color32::GRAY,
                            format!("{} via {}", peer.name, self.selected_protocol),
                        );
                    }
                }
            });
        });
    }
}
