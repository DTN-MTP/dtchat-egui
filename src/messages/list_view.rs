use std::collections::HashMap;

use crate::utils::clock::Clock;
use dtchat_backend::{
    dtchat::Peer,
    message::{ChatMessage, MessageStatus},
    time::DTChatTime,
};
use eframe::egui;
use egui::RichText;

pub struct MessageListView {
    pub show_timestamps: bool,
    pub compact_mode: bool,
    pub clock: Clock,
}

impl MessageListView {
    pub fn new() -> Self {
        let dumy_time = DTChatTime::now();
        Self {
            show_timestamps: true,
            compact_mode: false,
            clock: Clock::new(&dumy_time, true),
        }
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        messages: &[ChatMessage],
        current_time: &DTChatTime,
        local_peer: &Peer,
        other_peers: &HashMap<String, Peer>,
    ) {
        self.clock.update(current_time);
        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .stick_to_bottom(true)
            .show(ui, |ui| {
                if messages.is_empty() {
                    ui.colored_label(egui::Color32::GRAY, "No messages");
                } else {
                    for message in messages.iter() {
                        self.render(ui, message, local_peer, other_peers, self.clock.to_string());
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
        local_peer: &Peer,
        other_peers: &HashMap<String, Peer>,
        clock_str: String,
    ) {
        ui.horizontal(|ui| {
            // Trouver le nom du peer expéditeur

            let peer = if local_peer.uuid == msg.sender_uuid {
                Some(local_peer)
            } else {
                other_peers.get(&msg.sender_uuid)
            };
            let mut sep = "➡";
            // Status indicator avec couleurs selon le statut
            match &msg.status {
                MessageStatus::Failed => ui.colored_label(egui::Color32::RED, "[\u{2716}]"),
                MessageStatus::ReceivedByPeer => {
                    ui.colored_label(egui::Color32::GREEN, "[\u{2714}]")
                }
                MessageStatus::Sent => ui.colored_label(egui::Color32::LIGHT_BLUE, "[\u{1F680}]"),
                MessageStatus::Sending => ui.colored_label(egui::Color32::YELLOW, "[\u{1F4E4}]"),
                MessageStatus::Received => ui.colored_label(egui::Color32::GREEN, "[\u{1F4E5}]"),
            };

            if self.show_timestamps {
                // Format exact de dtchat_tui: [acked_time:send_time]
                let receive_time_str = match msg.receive_time {
                    Some(t) => t.ts_to_str(false, true, None, &chrono::Local),
                    None => match msg.predicted_arrival_time {
                        Some(pbat) => {
                            if msg.status != MessageStatus::Failed {
                                sep = &clock_str;
                            } else {
                                sep = "\u{1F6AB}";
                            }
                            pbat.ts_to_str(false, true, None, &chrono::Local)
                        }
                        None => "???".to_string(),
                    },
                };

                let send_time_str = msg.send_time.ts_to_str(false, true, None, &chrono::Local);
                let time_display = format!("[{}{}{}]", send_time_str, sep, receive_time_str);

                ui.colored_label(egui::Color32::LIGHT_GRAY, time_display);
                let peer_name = match peer {
                    Some(p) => p.name.clone(),
                    None => "Unknown".to_string(),
                };
                ui.label(RichText::new(format!("{}:", peer_name)).strong());
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
