use std::collections::HashMap;

use dtchat_backend::dtchat::{Peer, Room};
use egui::Ui;

use crate::messages::{MessagingMode, PreferencesContext};

pub struct SideSelectionView {
    last_peer: Option<Peer>,
    last_room: Option<Room>,
}

impl SideSelectionView {
    pub fn new() -> Self {
        Self {
            last_room: None,
            last_peer: None,
        }
    }

    pub fn show(
        &mut self,
        ui: &mut Ui,
        peers: &HashMap<String, Peer>,
        rooms: &HashMap<String, Room>,
        pref_ctx: &mut PreferencesContext,
        current_mode: &mut MessagingMode,
        request_filter: &mut bool,
    ) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 1.0;
            if ui
                .selectable_value(current_mode, MessagingMode::All, "All")
                .clicked()
            {
                pref_ctx.load_context("All");
                *request_filter = true;
            }
            ui.separator();
            if ui
                .selectable_value(
                    current_mode,
                    MessagingMode::Peer(self.last_peer.clone()),
                    "Peers",
                )
                .clicked()
            {
                if let Some(peer) = &self.last_peer {
                    pref_ctx.load_context(&peer.uuid);
                } else {
                    if let Some((_, peer)) = peers.iter().next() {
                        pref_ctx.load_context(&peer.uuid);
                        self.last_peer = Some(peer.clone());
                        *current_mode = MessagingMode::Peer(self.last_peer.clone())
                    }
                }

                *request_filter = true;
            };
            ui.separator();
            if ui
                .selectable_value(
                    current_mode,
                    MessagingMode::Room(self.last_room.clone()),
                    "Rooms",
                )
                .clicked()
            {
                if let Some(room) = &self.last_room {
                    pref_ctx.load_context(&room.uuid);
                } else {
                    if let Some((_, room)) = rooms.iter().next() {
                        pref_ctx.load_context(&room.uuid);
                        self.last_room = Some(room.clone());
                        *current_mode = MessagingMode::Room(self.last_room.clone())
                    }
                }
                *request_filter = true;
            };
        });
        ui.separator();
        egui::ScrollArea::vertical().show(ui, |ui| match current_mode {
            MessagingMode::Peer(peer_opt) => {
                if peers.is_empty() {
                    ui.label("No peers");
                } else {
                    for (_peer_uuid, peer) in peers {
                        if ui
                            .selectable_value(
                                peer_opt,
                                Some(peer.clone()),
                                format!("\u{1F464} {}", &peer.name),
                            )
                            .clicked()
                        {
                            self.last_peer = Some(peer.clone());
                            pref_ctx.load_context(&peer.uuid);
                            *request_filter = true;
                        };
                    }
                }
            }
            MessagingMode::Room(room_opt) => {
                if rooms.is_empty() {
                    ui.label("No rooms");
                } else {
                    for (_room_uuid, room) in rooms {
                        if ui
                            .selectable_value(
                                room_opt,
                                Some(room.clone()),
                                format!("\u{1F465} {}", &room.name),
                            )
                            .clicked()
                        {
                            self.last_room = Some(room.clone());
                            pref_ctx.load_context(&room.uuid);
                            *request_filter = true;
                        };
                    }
                }
            }
            MessagingMode::All => {
                ui.label("Showing all messages");
            }
        });
    }
}
