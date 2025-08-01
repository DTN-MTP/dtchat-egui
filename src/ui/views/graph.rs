use crate::domain::peer::{Peer, PeerManager};
use dtchat_backend::message::{ChatMessage, MessageStatus};
use dtchat_backend::time::DTChatTime;
use egui::Color32;
use egui_plot::{AxisHints, BoxElem, BoxPlot, BoxSpread, GridMark, Legend, Plot, VLine};
use std::collections::{HashMap, HashSet};
use std::ops::RangeInclusive;
#[derive(Clone)]
pub struct MessageGraphView {
    auto_bounds: bool,
    show_current_time: bool,
    active_participants: HashMap<String, String>,
    filtered_participants: HashSet<String>,
    hovered: bool,
}
#[allow(dead_code)]
trait AutoReset {
    fn auto_reset(self, auto: bool) -> Self;
}

impl AutoReset for Plot<'_> {
    fn auto_reset(self, auto: bool) -> Self {
        if auto {
            return self.reset();
        }
        self
    }
}

impl MessageGraphView {
    pub fn new() -> Self {
        Self {
            auto_bounds: true,
            show_current_time: true,
            active_participants: HashMap::new(),
            filtered_participants: HashSet::new(),
            hovered: false,
        }
    }

    fn update_participants(
        &mut self,
        messages: &[ChatMessage],
        peers: &[Peer],
        local_peer_uuid: &str,
    ) {
        self.active_participants.clear();

        // Helper function to find peer and get name
        let get_peer_name = |uuid: &str| -> String {
            if uuid == local_peer_uuid {
                "Me".to_string()
            } else {
                peers
                    .iter()
                    .find(|p| p.uuid == uuid)
                    .map(|p| p.name.clone())
                    .unwrap_or_else(|| {
                        let short_uuid = if uuid.len() >= 8 { &uuid[..8] } else { uuid };
                        format!("Peer {}", short_uuid)
                    })
            }
        };

        // Collect all unique sender UUIDs from messages
        let mut sender_uuids: std::collections::HashSet<String> = std::collections::HashSet::new();
        for message in messages {
            sender_uuids.insert(message.sender_uuid.clone());
        }

        // Add local peer if not already present
        sender_uuids.insert(local_peer_uuid.to_string());

        // Process all participants at once
        for uuid in sender_uuids {
            let name = get_peer_name(&uuid);
            self.active_participants.insert(uuid, name);
        }
    }

    fn should_show_message(&self, message: &ChatMessage) -> bool {
        !self.filtered_participants.contains(&message.sender_uuid)
    }

    fn create_box_element(
        &self,
        message: &ChatMessage,
        y_position: f64,
        now: f64,
    ) -> (BoxElem, String, Color32, String, f64, f64) {
        // Nom de la boîte simplifié sans emoji
        let box_name = self.truncate_text(&message.text, 30);

        // Statut du message pour la tooltip
        let status_text = match &message.status {
            MessageStatus::Failed => "FAILED",
            MessageStatus::ReceivedByPeer => "ACKED",
            MessageStatus::Sent => "SENT",
            MessageStatus::Sending => "SENDING",
            MessageStatus::Received => "RECEIVED",
        }
        .to_string();

        // Couleur selon le statut du message
        let status_color = match &message.status {
            MessageStatus::Failed => Color32::RED,
            MessageStatus::ReceivedByPeer => Color32::GREEN,
            MessageStatus::Sent => Color32::LIGHT_GRAY,
            MessageStatus::Sending => Color32::YELLOW,
            MessageStatus::Received => Color32::LIGHT_BLUE,
        };

        let (sent, pbat, recv) = message.get_shipment_status_timestamps();
        // Calcul de l'étendue de la boîte selon l'état du message
        let (start_time, end_time) = match &message.status {
            MessageStatus::ReceivedByPeer => {
                // let mut send_time = tx;
                // // ACK reçu - largeur = délai réel
                // let receive_time = message
                //     .receive_time
                //     .map(|t| {
                //         send_time -= 500.0;
                //         t.timestamp_millis() as f64
                //     })
                //     .unwrap_or(tx + 500.0);
                // (send_time, receive_time)
                let recv_opt = {
                    if let Some(val) = recv {
                        val as f64
                    } else {
                        sent as f64
                    }
                };
                (sent as f64, recv_opt as f64)
            }
            MessageStatus::Received => {
                let recv_opt = {
                    if let Some(val) = recv {
                        val as f64
                    } else {
                        sent as f64
                    }
                };
                (sent as f64, recv_opt as f64)
            }
            MessageStatus::Failed => {
                // Message échoué - boîte très courte, animation arrêtée
                ((sent - 50) as f64, (sent + 50) as f64)
            }
            MessageStatus::Sent | MessageStatus::Sending => {
                // Pas d'ACK encore - animer la boîte
                let recv_opt = {
                    if let Some(val) = pbat {
                        val as f64
                    } else {
                        now
                    }
                };
                (sent as f64, recv_opt)
            }
        };

        let box_elem = BoxElem::new(
            y_position,
            BoxSpread::new(start_time, start_time, start_time, end_time, end_time),
        )
        .name(box_name);

        (
            box_elem,
            message.sender_uuid.clone(),
            status_color,
            status_text,
            start_time,
            end_time,
        )
    }

    fn truncate_text(&self, text: &str, max_length: usize) -> String {
        if text.len() <= max_length {
            text.to_string()
        } else {
            let max = max_length.saturating_sub(3).min(text.len());
            format!("{}...", &text[..max])
        }
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        messages: &[ChatMessage],
        local_peer_uuid: &str,
        peer_manager: &PeerManager,
        current_time: DTChatTime,
    ) {
        let make_time_formatter = |show_date: bool, show_time: bool| {
            move |x: GridMark, _range: &RangeInclusive<f64>| {
                let datetime = DTChatTime::from_timestamp_millis(x.value as i64).unwrap();
                datetime.ts_to_str(show_date, show_time, None, &chrono::Local)
            }
        };

        let x_axes = vec![
            AxisHints::new_x()
                .formatter(make_time_formatter(true, false))
                .placement(egui_plot::VPlacement::Top),
            AxisHints::new_x()
                .formatter(make_time_formatter(false, true))
                .placement(egui_plot::VPlacement::Bottom),
        ];

        let now = current_time.timestamp_millis() as f64;
        let peers = peer_manager.peers();

        self.update_participants(messages, peers, local_peer_uuid);

        ui.horizontal(|ui| {
            ui.checkbox(&mut self.auto_bounds, "Auto_bounds");
        });

        // drag or scroll must cancel autobound
        // use hovered to treat this only if we are interacting with the graph
        // otherwise, other elements can trigger this logic (Sliders with decidedly_dragging)
        ui.input(|i| {
            if self.hovered && (i.pointer.is_decidedly_dragging() || i.raw_scroll_delta.y != 0.0) {
                self.auto_bounds = false;
            }
        });

        let filtered_messages: Vec<&ChatMessage> = messages
            .iter()
            .filter(|msg| self.should_show_message(msg))
            .collect();
        let mut first_message = now;
        let mut last_message = now;
        // Group messages by sender
        let mut boxes_by_participant: HashMap<String, Vec<(BoxElem, Color32, String)>> =
            HashMap::new();
        for (index, message) in filtered_messages.iter().enumerate() {
            let (box_elem, sender_uuid, status_color, status_text, from, to) =
                self.create_box_element(message, index as f64, now);
            if from < first_message {
                first_message = from;
            }
            if to > last_message {
                last_message = from;
            }

            boxes_by_participant
                .entry(sender_uuid)
                .or_insert(Vec::new())
                .push((box_elem, status_color, status_text));
        }

        let num_messages = if filtered_messages.is_empty() {
            1.0
        } else {
            filtered_messages.len() as f64
        };
        let plot_height = ui.available_height().max(300.0);

        let plt = Plot::new("DTChat Timeline")
            .allow_zoom(true)
            .allow_drag(true)
            .legend(Legend::default().position(egui_plot::Corner::LeftTop))
            .show_x(true)
            .show_y(false)
            .include_y(-0.5)
            .include_y(num_messages + 0.5)
            .include_x(last_message + (last_message - first_message) * 0.2)
            .custom_x_axes(x_axes)
            .custom_y_axes(vec![])
            // .auto_reset(reset_requested)
            // .auto_bounds(self.auto_bounds)
            .label_formatter(|name, value| {
                if !name.is_empty() {
                    format!("{}: {:.*}%", name, 1, value.y)
                } else {
                    let value = DTChatTime::from_timestamp_millis(value.x as i64).unwrap();
                    value.ts_to_str(false, true, None, &chrono::Local)
                }
            })
            .height(plot_height)
            .show(ui, |plot_ui| {
                plot_ui.set_auto_bounds(self.auto_bounds);
                if self.show_current_time {
                    plot_ui.vline(VLine::new(now).color(Color32::RED).name("Current Time"));
                }

                for (participant_uuid, boxes_with_colors) in &boxes_by_participant {
                    if let Some(participant_name) = self.active_participants.get(participant_uuid) {
                        let formatter_name = participant_name.clone();

                        // Séparer les boîtes par couleur de statut
                        let mut boxes_by_status: HashMap<Color32, Vec<(BoxElem, String)>> =
                            HashMap::new();
                        for (box_elem, status_color, status_text) in boxes_with_colors {
                            boxes_by_status
                                .entry(*status_color)
                                .or_insert(Vec::new())
                                .push((box_elem.clone(), status_text.clone()));
                        }

                        // Créer un BoxPlot pour chaque couleur de statut
                        for (status_color, boxes_with_status) in boxes_by_status {
                            let status_name = match status_color {
                                Color32::RED => format!("{} (FAILED)", participant_name),
                                Color32::GREEN => format!("{} (ACKED)", participant_name),
                                Color32::YELLOW => format!("{} (SENDING)", participant_name),
                                Color32::LIGHT_BLUE => format!("{} (RECEIVED)", participant_name),
                                _ => format!("{} (SENT)", participant_name),
                            };

                            // Extraire les BoxElem et garder les statuts pour la tooltip
                            let boxes: Vec<BoxElem> = boxes_with_status
                                .iter()
                                .map(|(box_elem, _)| box_elem.clone())
                                .collect();
                            let statuses: Vec<String> = boxes_with_status
                                .iter()
                                .map(|(_, status)| status.clone())
                                .collect();

                            // Cloner formatter_name pour l'utiliser dans la closure
                            let formatter_name_for_closure = formatter_name.clone();

                            let box_plot = BoxPlot::new(boxes)
                                .name(status_name)
                                .color(status_color)
                                .horizontal()
                                .allow_hover(true)
                                .element_formatter(Box::new(move |bar, _bar_chart| {
                                    let tx_time = DTChatTime::from_timestamp_millis(
                                        bar.spread.quartile1 as i64,
                                    )
                                    .unwrap();
                                    let rx_time = DTChatTime::from_timestamp_millis(
                                        bar.spread.quartile3 as i64,
                                    )
                                    .unwrap();
                                    let date = tx_time.date_naive() != rx_time.date_naive();

                                    // Essayer de récupérer le statut correspondant (approximation basée sur l'index)
                                    let status_info = if !statuses.is_empty() {
                                        format!("\nStatus: {}", &statuses[0]) // Utilise le premier statut de ce groupe
                                    } else {
                                        "".to_string()
                                    };

                                    format!(
                                        "Message: {}\nSent by {}\ntx time: {}\nrx_time: {}{}",
                                        bar.name,
                                        formatter_name_for_closure,
                                        tx_time.ts_to_str(date, true, None, &chrono::Local),
                                        rx_time.ts_to_str(date, true, None, &chrono::Local),
                                        status_info
                                    )
                                }));

                            plot_ui.box_plot(box_plot);
                        }
                    }
                }
            });
        self.hovered = plt.response.hovered();
    }
}

impl Default for MessageGraphView {
    fn default() -> Self {
        Self::new()
    }
}
