use crate::domain::peer::PeerManager;
use crate::ui::main::ViewType;
use egui::{ComboBox, Ui};
use socket_engine::endpoint::EndpointProto;

pub struct MessageSettingsBar;

impl MessageSettingsBar {
    pub fn new() -> Self {
        Self
    }

    pub fn show(
        &mut self,
        ui: &mut Ui,
        current_view: &mut ViewType,
        selected_peer_for_relative: &mut Option<String>,
        selected_protocol_filter: &mut Option<EndpointProto>,
        peer_manager: &PeerManager,
        local_peer_uuid: &str,
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

                // Sorting options
                ui.label("Sort:");
                ComboBox::from_id_salt("sort_selector")
                    .selected_text(if let Some(ref uuid) = selected_peer_for_relative {
                        if uuid == local_peer_uuid {
                            "Me"
                        } else {
                            peer_manager
                                .peers()
                                .iter()
                                .find(|p| &p.uuid == uuid)
                                .map(|p| p.name.as_str())
                                .unwrap_or("Unknown peer")
                        }
                    } else {
                        "Standard"
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(selected_peer_for_relative, None, "Standard");
                        ui.selectable_value(
                            selected_peer_for_relative,
                            Some(local_peer_uuid.to_string()),
                            "Relative (to me)",
                        );

                        for peer in peer_manager.peers() {
                            if peer.uuid != local_peer_uuid {
                                ui.selectable_value(
                                    selected_peer_for_relative,
                                    Some(peer.uuid.clone()),
                                    format!("Relative ({})", peer.name),
                                );
                            }
                        }
                    });

                ui.separator();

                ui.label("Protocol:");
                ComboBox::from_id_salt("protocol_filter")
                    .selected_text(
                        selected_protocol_filter
                            .as_ref()
                            .map(|p| p.to_string())
                            .unwrap_or("All".to_string()),
                    )
                    .show_ui(ui, |ui| {
                        ui.selectable_value(selected_protocol_filter, None, "All");
                        ui.selectable_value(
                            selected_protocol_filter,
                            Some(EndpointProto::Tcp),
                            EndpointProto::Tcp.to_string(),
                        );
                        ui.selectable_value(
                            selected_protocol_filter,
                            Some(EndpointProto::Udp),
                            EndpointProto::Udp.to_string(),
                        );
                        ui.selectable_value(
                            selected_protocol_filter,
                            Some(EndpointProto::Bp),
                            EndpointProto::Bp.to_string(),
                        );
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
