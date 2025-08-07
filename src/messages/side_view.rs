use std::collections::HashMap;

use dtchat_backend::dtchat::{Peer, Room};
use egui::Ui;

pub struct SideSelectionView {
    last_peer: Option<Peer>,
    last_room: Option<Room>,
    current_type: MessagingType,
}
#[derive(PartialEq)]
enum MessagingType {
    Direct,
    Room,
}

impl SideSelectionView {
    pub fn new() -> Self {
        Self {
            last_room: None,
            last_peer: None,
            current_type: MessagingType::Direct,
        }
    }

    pub fn show(
        &mut self,
        ui: &mut Ui,
        peers: &HashMap<String, Peer>,
        rooms: &HashMap<String, Room>,
        selected_room: &mut Option<Room>,
        selected_peer: &mut Option<Peer>,
    ) {
        ui.horizontal(|ui| {
            if ui
                .selectable_value(&mut self.current_type, MessagingType::Direct, "Peers")
                .clicked()
            {
                *selected_room = None;
                *selected_peer = self.last_peer.clone();
            };
            ui.separator();
            if ui
                .selectable_value(&mut self.current_type, MessagingType::Room, "Rooms")
                .clicked()
            {
                *selected_peer = None;
                *selected_room = self.last_room.clone();
            };
        });
        ui.separator();
        egui::ScrollArea::vertical().show(ui, |ui| match self.current_type {
            MessagingType::Direct => {
                if peers.is_empty() {
                    ui.label("No peers");
                } else {
                    for (_peer_uuid, peer) in peers {
                        if ui
                            .selectable_value(
                                selected_peer,
                                Some(peer.clone()),
                                format!("\u{1F464} {}", &peer.name),
                            )
                            .clicked()
                        {
                            self.last_peer = Some(peer.clone())
                        };
                    }
                }
            }
            MessagingType::Room => {
                if rooms.is_empty() {
                    ui.label("No rooms");
                } else {
                    for (_room_uuid, room) in rooms {
                        if ui
                            .selectable_value(
                                selected_room,
                                Some(room.clone()),
                                format!("\u{1F465} {}", &room.name),
                            )
                            .clicked()
                        {
                            self.last_room = Some(room.clone())
                        };
                    }
                }
            }
        });
    }
}
