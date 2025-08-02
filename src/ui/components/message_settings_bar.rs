use crate::ui::main::{ProtoFilter, ViewType};
use crate::utils::text::PrettyStr;
use dtchat_backend::dtchat::Peer;
use dtchat_backend::message::SortStrategy;
use dtchat_backend::EndpointProto;
use egui::{ComboBox, Slider, Ui};

pub struct MessageSettingsBar {
    last_sort_strategy_peer: Option<Peer>,
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
            last_sort_strategy_peer: None,
        }
    }

    pub fn show(
        &mut self,
        ui: &mut Ui,
        current_view: &mut ViewType,
        // request_sort_with_strategy: &mut Option<SortStrategy>,
        // request_protocol_filter: &mut Option<ProtoFilter>,
        sort_strategy: &mut SortStrategy,
        request_sort_strategy: &mut bool,
        protocol_filter: &mut ProtoFilter,
        request_protocol_filter: &mut bool,
        max_message_count: &mut usize,
        message_in_db: usize,
        local_peer: &Peer,
        other_peers: &Vec<Peer>,
    ) {
        ui.add_space(3.0);
        ui.horizontal(|ui| {
            ui.label("View:");
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
                        ViewType::Network,
                        ViewType::Network.name(),
                    );
                });

            if *current_view != ViewType::Network {
                ui.separator();

                let previous_filter = protocol_filter.clone();
                ui.label("Filter:");
                ComboBox::from_id_salt("protocol_filter")
                    .selected_text(
                        protocol_filter.to_pretty_str(),
                    )
                    .show_ui(ui, |ui| {
                        let mut opt = ProtoFilter::NoFilter;
                        ui.selectable_value( protocol_filter, opt.clone(), opt.to_pretty_str());
                        opt = ProtoFilter::Filter(EndpointProto::Tcp);
                        ui.selectable_value(
                            protocol_filter,
                            opt.clone(),
                            opt.to_pretty_str(),
                        );
                        opt = ProtoFilter::Filter(EndpointProto::Udp);
                        ui.selectable_value(
                             protocol_filter,
                            opt.clone(),
                            opt.to_pretty_str(),
                        );
                        opt = ProtoFilter::Filter(EndpointProto::Bp);
                        ui.selectable_value(
                             protocol_filter,
                            opt.clone(),
                            opt.to_pretty_str(),
                        );
                    });

                // Trigger on selection change
                if previous_filter != *protocol_filter {
                    *request_protocol_filter = true;
                }

                ui.separator();

                ui.label("Sort strategy:");
                ui.menu_button(get_str_for_strat(local_peer.uuid.clone(), self.last_sort_strategy_peer.clone(), sort_strategy), |ui| {
                        if ui.button("Standard").on_hover_text("Sorted by sending times").clicked() {

                            *request_sort_strategy = true;
                            *sort_strategy = SortStrategy::Standard;
                            self.last_sort_strategy_peer = None;
                            ui.close_menu();
                        }
                        if ui.button("Local").on_hover_text("Sorted by receiving time for the local peer and sending times for the other peers").clicked() {
                            *request_sort_strategy = true;
                            *sort_strategy = SortStrategy::Relative(local_peer.uuid.clone());
                            self.last_sort_strategy_peer = Some(local_peer.clone());
                            ui.close_menu();
                        }
                        ui.menu_button("Relative", |ui| {
                            let mut clicked = None;

                             for peer in other_peers {

                                if ui.button(peer.name.as_str()).on_hover_text(format!("Sorted by receiving time for peer {} and sending times for the other peers", peer.name)).clicked() {
                                    clicked = Some(peer.clone());
                                }

                             if let Some(ref peer) = clicked {
                                *request_sort_strategy = true;
                                *sort_strategy = SortStrategy::Relative(peer.uuid.clone());
                                self.last_sort_strategy_peer = Some(peer.clone());
                                ui.close_menu();
                             }
                            }

                        });
                });
                let enable_slider= message_in_db > 0;

                ui.separator();
                // TODO, use this before using other sliders
                ui.style_mut().spacing.slider_width = 60.0;
                ui.add_enabled(
                enable_slider,
                {
                    let displayed = *max_message_count;
                    let str_display = if message_in_db == 0 {
                        String::from("No messages in the DB")
                    } else if *max_message_count == message_in_db {
                        String::from("Showing all messages")
                    } else if *max_message_count == 0 {
                        String::from("Showing no messages")
                    } else {
                        format!("Showing the last {} messages", displayed)
                    };

                    Slider::new( max_message_count, message_in_db..=0).text(str_display)
                }
            );
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
