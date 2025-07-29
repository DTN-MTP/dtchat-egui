use crate::domain::peer::{Peer, PeerManager};
use crate::ui::main::{ProtoFilter, ViewType};
use dtchat_backend::message::SortStrategy;
use egui::{ComboBox, Ui};
use socket_engine::endpoint::EndpointProto;

pub struct MessageSettingsBar {
    last_sort_strategy: SortStrategy,
    last_sort_strategy_peer: Option<Peer>,
    last_proto_filter: Option<ProtoFilter>,
}

fn get_str_for_strat(local_peer_uuid: String, peer: Option<Peer>, strat: &SortStrategy) -> String {
    match strat {
        SortStrategy::Standard => "Standard".to_string(),
        SortStrategy::Relative(sort_for_uuid) => {
            if local_peer_uuid == *sort_for_uuid {
                "Local".to_string()
            } else {
                format!("Relative ({})", peer.unwrap().name)
            }
        }
    }
}

impl MessageSettingsBar {
    pub fn new() -> Self {
        Self {
            last_sort_strategy: SortStrategy::Standard,
            last_sort_strategy_peer: None,
            last_proto_filter: None,
        }
    }

    pub fn show(
        &mut self,
        ui: &mut Ui,
        current_view: &mut ViewType,
        request_sort_with_strategy: &mut Option<SortStrategy>,
        request_protocol_filter: &mut Option<ProtoFilter>,
        peer_manager: &PeerManager,
        local_peer: &Peer,
    ) {
        ui.add_space(3.0);
        ui.horizontal(|ui| {
            ui.label("Views:");
            ComboBox::from_id_salt("view_selector")
                .selected_text(current_view.name())
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        current_view,
                        ViewType::MessageGraph,
                        ViewType::MessageGraph.name(),
                    );
                    ui.selectable_value(
                        current_view,
                        ViewType::MessageList,
                        ViewType::MessageList.name(),
                    );
                    ui.selectable_value(
                        current_view,
                        ViewType::Settings,
                        ViewType::Settings.name(),
                    );
                });

            if *current_view != ViewType::Settings {
                ui.separator();

                let previous_filter = self.last_proto_filter.clone();
                ui.label("Protocol:");
                ComboBox::from_id_salt("protocol_filter")
                    .selected_text(
                        &self.last_proto_filter
                            .as_ref()
                            .map(|p| p.to_string())
                            .unwrap_or("All".to_string()),
                    )
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.last_proto_filter, None, "All");
                        ui.selectable_value(
                            &mut self.last_proto_filter,
                            Some(ProtoFilter::Filter(EndpointProto::Tcp)),
                            EndpointProto::Tcp.to_string(),
                        );
                        ui.selectable_value(
                            &mut self.last_proto_filter,
                            Some(ProtoFilter::Filter(EndpointProto::Udp)),
                            EndpointProto::Udp.to_string(),
                        );
                        ui.selectable_value(
                            &mut self.last_proto_filter,
                            Some(ProtoFilter::Filter(EndpointProto::Bp)),
                            EndpointProto::Bp.to_string(),
                        );
                    });

                // Trigger on selection change
                if previous_filter != self.last_proto_filter {
                    *request_protocol_filter = self.last_proto_filter.clone();
                }

                ui.separator();

                ui.label("Sort strategy:");
                ui.menu_button(get_str_for_strat(local_peer.uuid.clone(), self.last_sort_strategy_peer.clone(), &self.last_sort_strategy), |ui| {
                        if ui.button("Standard").on_hover_text("Sorted by sending times").clicked() {

                            *request_sort_with_strategy = Some(SortStrategy::Standard);
                            self.last_sort_strategy = SortStrategy::Standard;
                            self.last_sort_strategy_peer = None;
                            ui.close_menu();
                        }
                        if ui.button("Local").on_hover_text("Sorted by receiving time for the local peer and sending times for the other peers").clicked() {
                            // locked_model.sort_messages(SortStrategy::Relative(local_peer.clone()));
                            *request_sort_with_strategy = Some(SortStrategy::Relative(local_peer.uuid.clone()));
                            self.last_sort_strategy = SortStrategy::Standard;
                            self.last_sort_strategy_peer = Some(local_peer.clone());
                            ui.close_menu();
                        }
                        ui.menu_button("Relative", |ui| {
                            let mut clicked = None;

                             for peer in peer_manager.peers() {
                            if peer.uuid != local_peer.uuid {
                                if ui.button(peer.name.as_str()).on_hover_text(format!("Sorted by receiving time for peer {} and sending times for the other peers", peer.name)).clicked() {
                                    clicked = Some(peer.clone());
                                }
                             }
                             if let Some(ref peer) = clicked {
                                *request_sort_with_strategy = Some(SortStrategy::Relative(peer.uuid.clone()));
                                self.last_sort_strategy = SortStrategy::Relative(peer.uuid.clone());
                                self.last_sort_strategy_peer = Some(peer.clone());
                                ui.close_menu();
                             }
                            }

                        });
                });



            }
        });
        ui.add_space(3.0);
    }
}

impl Default for MessageSettingsBar {
    fn default() -> Self {
        Self::new()
    }
}
