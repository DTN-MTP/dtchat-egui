use crate::messages::MessagingMode;
use crate::utils::font::PrettyStr;
use dtchat_backend::dtchat::{ChatModel, Peer, Room};
use dtchat_backend::message::Content;
use dtchat_backend::Endpoint;
use eframe::egui;
use egui::{ComboBox, RichText, Ui};
use egui_file_dialog::FileDialog;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub struct MessagePromptView {
    model: Arc<Mutex<ChatModel>>,
    input_text: String,
    pbat_enabled: bool,
    file_dialog: FileDialog,
    picked_file: Option<PathBuf>,
}

#[derive(Clone)]
enum PrepareSend {
    ToRoom(Room),
    ToPeer(Peer, Endpoint),
}

impl MessagePromptView {
    pub fn new(model: Arc<Mutex<ChatModel>>) -> Self {
        Self {
            model,
            input_text: String::new(),
            pbat_enabled: false,
            file_dialog: FileDialog::new(),
            picked_file: None,
        }
    }

    pub fn show(
        &mut self,
        ctx: &egui::Context,
        ui: &mut egui::Ui,
        proto_for_peer: &mut Option<Endpoint>,
        pbat_support_by_model: bool,
        current_mode: &MessagingMode,
    ) {
        let mut prepare_send = None;

        self.file_dialog.update(ctx);
        if let Some(path) = self.file_dialog.take_picked() {
            self.picked_file = Some(path.to_path_buf());
            self.input_text = format!("Send file {}", path.to_string_lossy().to_string());
        }

        ui.add_space(8.0);

        ui.horizontal(|ui| match current_mode {
            MessagingMode::Peer(Some(peer)) => {
                let peer_has_endpoints = peer.endpoints.len() > 0;
                ui.add_enabled_ui(peer_has_endpoints, |ui| {
                    if proto_for_peer.is_none() && peer_has_endpoints {
                        *proto_for_peer = Some(peer.endpoints[0].clone());
                    }

                    let selected_text = match &proto_for_peer {
                        Some(endpoint) => {
                            prepare_send =
                                Some(PrepareSend::ToPeer(peer.clone(), endpoint.clone()));
                            endpoint.to_pretty_str()
                        }
                        None => "⚠ no endpoints".to_string(),
                    };
                    ui.label("Select target endpoint: ");
                    ComboBox::from_id_salt("protocol_selector")
                        .selected_text(selected_text.clone())
                        .show_ui(ui, |ui| {
                            for endpoint in &peer.endpoints {
                                let is_selected = selected_text == *endpoint.to_pretty_str();
                                if ui
                                    .selectable_label(is_selected, endpoint.to_pretty_str())
                                    .clicked()
                                {
                                    *proto_for_peer = Some(endpoint.clone());
                                }
                            }
                        });
                });
            }
            MessagingMode::Room(Some(room)) => {
                prepare_send = Some(PrepareSend::ToRoom(room.clone()));
                ui.label(format!("To room \"{}\"", room.name));
            }
            _ => {
                ui.colored_label(egui::Color32::DARK_GRAY, format!("Select a peer/room"));
            }
        });
        ui.add_space(8.0);

        ui.horizontal(|ui| {
            // we want to keep prepare_send for footer
            let mut must_send = prepare_send.clone();
            let input_response = ui.add_enabled(
                prepare_send.is_some() && self.picked_file.is_none(),
                |ui: &mut Ui| {
                    let text_edit = egui::TextEdit::singleline(&mut self.input_text)
                        .hint_text("Type your message...")
                        .desired_width(ui.available_width() - 24.0 - 60.0 - 24.0)
                        .margin(egui::Margin::same(6));

                    let response = ui.add(text_edit);
                    response
                },
            );

            let button_text = if self.picked_file.is_none() {
                RichText::new("\u{1F4C1}")
            } else {
                RichText::new("\u{2716}").color(egui::Color32::RED)
            };
            let pick_button = egui::Button::new(button_text)
                .corner_radius(4.0)
                .min_size(egui::Vec2::new(24.0, 24.0));

            if ui.add(pick_button).clicked() {
                self.input_text.clear();
                if self.picked_file.is_none() {
                    self.file_dialog.pick_file();
                } else {
                    self.picked_file = None
                }
            }

            ui.add_enabled(prepare_send.is_some(), |ui: &mut Ui| {
                let send_button = egui::Button::new(RichText::new("Send 📤").strong())
                    // .fill(egui::Color32::from_rgb(42, 124, 190))
                    .corner_radius(4.0)
                    .min_size(egui::Vec2::new(60.0, 24.0));

                let response = ui.add(send_button);

                let clicked = response.clicked()
                    || (input_response.lost_focus()
                        && ui.input(|i| i.key_pressed(egui::Key::Enter)));

                if !clicked {
                    must_send = None;
                }

                response
            });

            if let Some(to_send) = must_send {
                if let Ok(mut model) = self.model.lock() {
                    let content = match &self.picked_file {
                        Some(file) => Content::File(file.to_string_lossy().to_string()),
                        None => Content::Text(self.input_text.to_string()),
                    };
                    match to_send {
                        PrepareSend::ToRoom(room) => {
                            model.send_to_room(&content, &room.uuid, self.pbat_enabled);
                        }
                        PrepareSend::ToPeer(peer, endpoint) => {
                            model.send_to_peer(
                                &content,
                                &peer.uuid.clone(),
                                peer.uuid.clone(),
                                &endpoint,
                                self.pbat_enabled,
                            );
                        }
                    }
                }
                input_response.request_focus();
                self.picked_file = None;
                self.input_text.clear();
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
            ui.with_layout(
                egui::Layout::right_to_left(egui::Align::Center),
                |ui| match prepare_send {
                    Some(to_send) => match to_send {
                        PrepareSend::ToRoom(room) => {
                            ui.colored_label(egui::Color32::GRAY, format!("to room {}", room.name));
                        }
                        PrepareSend::ToPeer(peer, endpoint) => {
                            ui.colored_label(
                                egui::Color32::GRAY,
                                format!("to {} via {}", peer.name, endpoint.to_pretty_str()),
                            );
                        }
                    },
                    None => {}
                },
            );
        });
    }
}
