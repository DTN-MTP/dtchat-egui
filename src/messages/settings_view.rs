use std::collections::HashMap;

use crate::messages::{MessageCountToDisplay, MessageViewType, ProtoFilter};
use crate::utils::text::PrettyStr;
use dtchat_backend::dtchat::Peer;
use dtchat_backend::message::SortStrategy;
use dtchat_backend::EndpointProto;
use egui::{ComboBox, Slider, Ui};

pub struct MessageSettingsView {
    last_sort_strategy_peer: Option<Peer>,
}

fn get_str_for_strat(local_peer_uuid: String, _peer: Option<Peer>, strat: &SortStrategy) -> String {
    match strat {
        SortStrategy::Standard => "Standard".to_string(),
        SortStrategy::Relative(sort_for_uuid) => {
            if local_peer_uuid == *sort_for_uuid {
                "Local".to_string()
            } else {
                format!("Relative")
            }
        }
    }
}

fn paint_dropdown_arrow(
    painter: &egui::Painter,
    rect: egui::Rect,
    visuals: &egui::style::WidgetVisuals,
) {
    // Exact logic from egui's paint_default_icon
    let rect = egui::Rect::from_center_size(
        rect.center(),
        egui::vec2(rect.width() * 0.7, rect.height() * 0.45),
    );

    painter.add(egui::Shape::convex_polygon(
        vec![rect.left_top(), rect.right_top(), rect.center_bottom()],
        visuals.fg_stroke.color,
        egui::Stroke::NONE,
    ));
}

impl MessageSettingsView {
    pub fn new() -> Self {
        Self {
            last_sort_strategy_peer: None,
        }
    }

    pub fn show(
        &mut self,
        ui: &mut Ui,
        current_view: &mut MessageViewType,
        sort_strategy: &mut SortStrategy,
        protocol_filter: &mut ProtoFilter,
        max_message_count: &mut MessageCountToDisplay,
        message_in_db_for_ctx: usize,
        local_peer: &Peer,
        other_peers: &HashMap<String, Peer>,
        request_filter: &mut bool,
    ) {
        ui.add_space(3.0);

        //   ui.vertical(|ui| {
        ui.horizontal(|ui| {
                ui.label("View:");
                ComboBox::from_id_salt("view_selector")
                    .selected_text(current_view.name())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            current_view,
                            MessageViewType::MessageGraph,
                            MessageViewType::MessageGraph.name(),
                        );
                        ui.selectable_value(
                            current_view,
                            MessageViewType::MessageList,
                            MessageViewType::MessageList.name(),
                        );
                });

                ui.separator();
                let previous_filter = protocol_filter.clone();
                ui.label("Show:");
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
                    *request_filter = true;
                }

                ui.separator();



                ui.label("Sort by:");
                // TODO: maybe make this a separate widget if we need again this kind of combobox workaround
                ui.add_sized(egui::Vec2::new(100.0, ui.spacing().interact_size.y), |ui: &mut egui::Ui| {
                    ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
                        let button_text = get_str_for_strat(local_peer.uuid.clone(), self.last_sort_strategy_peer.clone(), sort_strategy);

                        // Outer menu button
                        let response = ui.menu_button(button_text, |ui| {
                            ui.add_sized(egui::Vec2::new(100.0, ui.spacing().interact_size.y), |ui: &mut egui::Ui| {
                                ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
                                    if ui
                                        .selectable_value(sort_strategy, SortStrategy::Standard, "Standard")
                                        .on_hover_text("Sorted by sending times")
                                        .clicked()
                                    {
                                        *request_filter = true;
                                        self.last_sort_strategy_peer = None;
                                        ui.close_menu();
                                    }

                                    if ui
                                        .selectable_value(
                                            sort_strategy,
                                            SortStrategy::Relative(local_peer.uuid.clone()),
                                            "Local",
                                        )
                                        .on_hover_text("Sorted by receiving time for the local peer and sending times for the other peers")
                                        .clicked()
                                    {
                                        *request_filter = true;
                                        self.last_sort_strategy_peer = Some(local_peer.clone());
                                        ui.close_menu();
                                    }

                                    ui.menu_button("Relative", |ui| {
                                        for (peer_uuid, peer) in other_peers {
                                            if ui
                                                .selectable_value(sort_strategy, SortStrategy::Relative(peer_uuid.clone()), &peer.name)
                                                .on_hover_text(format!(
                                                    "Sorted by receiving time for peer {} and sending times for the other peers",
                                                    peer.name
                                                ))
                                                .clicked()
                                            {
                                                *request_filter = true;
                                                self.last_sort_strategy_peer = Some(peer.clone());
                                                ui.close_menu();
                                            }
                                        }
                                    });
                                }).response
                            });
                        });

                        // --- Draw arrow like ComboBox ---
                        let icon_size = egui::Vec2::splat(ui.spacing().icon_width);

                        // Shrink to account for right-side button padding
                        let padded_rect = response.response.rect.shrink2(egui::vec2(ui.spacing().button_padding.x, 0.0));

                        let icon_rect = egui::Align2::RIGHT_CENTER.align_size_within_rect(icon_size, padded_rect);
                        let visuals = ui.style().interact(&response.response);
                        paint_dropdown_arrow(&ui.painter(), icon_rect.expand(visuals.expansion), &visuals);
                    })
                    .response
                });


                ui.separator();

                let enable_slider = message_in_db_for_ctx > 0;
                let (mut slider, display_txt) = match max_message_count {
                        MessageCountToDisplay::Nothing => (0, "Hide all".to_string()),
                        MessageCountToDisplay::All => (message_in_db_for_ctx, "Show all".to_string()),
                        MessageCountToDisplay::Last(count) => (*count, format!("Last {} messages", count).to_string()),
                    };
                // TODO, use this before using other sliders
                ui.style_mut().spacing.slider_width = 60.0;
                if ui.add_enabled(enable_slider, {

                    Slider::new(&mut slider, message_in_db_for_ctx..=0).text(display_txt)
                }).changed(){

                    if slider == 0 {
                        if *max_message_count != MessageCountToDisplay::Nothing {
                             *max_message_count = MessageCountToDisplay::Nothing;
                        }
                    } else if  slider == message_in_db_for_ctx {
                       *max_message_count = MessageCountToDisplay::All;
                    } else {
                        *max_message_count = MessageCountToDisplay::Last(slider);
                    }
            }

            });
        ui.add_space(3.0);
    }
}

impl Default for MessageSettingsView {
    fn default() -> Self {
        Self::new()
    }
}
