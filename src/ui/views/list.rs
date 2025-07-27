use crate::domain::peer::{Peer, PeerManager};
use dtchat_backend::message::ChatMessage;
use eframe::egui;
use std::collections::VecDeque;

pub struct MessageListView {
    pub max_messages_displayed: usize,
    pub show_timestamps: bool,
    pub compact_mode: bool,
}

impl MessageListView {
    pub fn new() -> Self {
        Self {
            max_messages_displayed: 8, // Comme dans dtchat_tui
            show_timestamps: true,
            compact_mode: false,
        }
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        messages: &VecDeque<ChatMessage>,
        local_peer: &Peer,
        peer_manager: &PeerManager,
    ) {
        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                if messages.is_empty() {
                    ui.colored_label(egui::Color32::GRAY, "Empty chat");
                } else {
                    // Afficher les derniers messages dans l'ordre chronologique
                    let messages_to_show: Vec<_> = messages
                        .iter()
                        .rev()
                        .take(self.max_messages_displayed)
                        .collect::<Vec<_>>()
                        .into_iter()
                        .rev()
                        .collect();

                    for message in messages_to_show {
                        self.render(ui, message, &local_peer.uuid, peer_manager);
                        ui.add_space(4.0);
                    }
                }
            });
    }

    /// Rendre un message avec le format exact de dtchat_tui.rs
    fn render(
        &self,
        ui: &mut egui::Ui,
        msg: &ChatMessage,
        local_uuid: &str,
        peer_manager: &PeerManager,
    ) {
        ui.horizontal(|ui| {
            // Trouver le nom du peer expéditeur
            let sender_name = if msg.sender_uuid == local_uuid {
                "You" // Message local
            } else {
                // Chercher le peer dans la liste
                peer_manager
                    .peers()
                    .iter()
                    .find(|p| p.uuid == msg.sender_uuid)
                    .map(|p| p.name.as_str())
                    .unwrap_or("Unknown")
            };

            ui.label(format!("[{}]", sender_name));

            // Status indicator avec couleurs selon le statut
            let (status_text, status_color) = match &msg.status {
                dtchat_backend::message::MessageStatus::Failed => ("FAILED", egui::Color32::RED),
                dtchat_backend::message::MessageStatus::ReceivedByPeer => {
                    ("ACKED", egui::Color32::GREEN)
                }
                dtchat_backend::message::MessageStatus::Sent => ("SENT", egui::Color32::LIGHT_GRAY),
                dtchat_backend::message::MessageStatus::Sending => {
                    ("SENDING", egui::Color32::YELLOW)
                }
                dtchat_backend::message::MessageStatus::Received => {
                    ("RECEIVED", egui::Color32::LIGHT_BLUE)
                }
            };

            ui.colored_label(status_color, format!("[{}]", status_text));

            if self.show_timestamps {
                // Format exact de dtchat_tui: [acked_time:send_time]
                let acked_time_str = match msg.send_completed {
                    Some(t) => t.format("%H:%M:%S").to_string(),
                    None => String::new(),
                };
                let send_time_str = msg.send_time.format("%H:%M:%S").to_string();
                let time_display = format!("[{}:{}]", acked_time_str, send_time_str);

                ui.colored_label(egui::Color32::LIGHT_GRAY, time_display);
            }

            // Message text avec troncature exacte de dtchat_tui (40 chars -> 37 + "...")
            let display_text = if msg.text.len() > 40 && !self.compact_mode {
                format!("{}...", &msg.text[..37])
            } else {
                msg.text.clone()
            };

            ui.label(display_text);
        });
    }
}

impl Default for MessageListView {
    fn default() -> Self {
        Self::new()
    }
}
