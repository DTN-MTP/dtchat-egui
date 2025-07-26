use crate::domain::peer::{Peer, PeerManager};
use chrono::{DateTime, Local, Utc};
use dtchat_backend::message::{ChatMessage, MessageStatus};
use egui::Color32;
use egui_plot::{BoxElem, BoxPlot, BoxSpread, Legend, Plot, VLine};
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Clone)]
pub struct MessageGraphView {
    show_current_time: bool,
    active_participants: HashMap<String, String>,
    filtered_participants: HashSet<String>,
}

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

pub fn ts_to_str(
    datetime: &DateTime<Utc>,
    date: bool,
    time: bool,
    separator: Option<String>,
) -> String {
    let mut res = "".to_string();
    if date {
        res += &datetime.format("%Y-%m-%d").to_string();
    }
    if let Some(sep) = separator {
        res += &sep;
    }
    if time {
        res += &datetime.format("%H:%M:%S").to_string()
    }
    res
}

impl MessageGraphView {
    pub fn new() -> Self {
        Self {
            show_current_time: true,
            active_participants: HashMap::new(),
            filtered_participants: HashSet::new(),
        }
    }

    fn update_participants(
        &mut self,
        messages: &VecDeque<ChatMessage>,
        peers: &[Peer],
        local_peer_uuid: &str,
    ) {
        self.active_participants.clear();

        // Helper function to find peer and get name
        let get_peer_name = |uuid: &str| -> String {
            if uuid == local_peer_uuid {
                "Moi".to_string()
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
    ) -> (BoxElem, String, Color32, String) {
        let tx = message.send_time.timestamp_millis() as f64;

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

        // Calcul de l'étendue de la boîte selon l'état du message
        let (start_time, end_time) = match &message.status {
            MessageStatus::ReceivedByPeer => {
                // ACK reçu - largeur = délai réel
                let ack_time = message
                    .send_completed
                    .map(|t| t.timestamp_millis() as f64)
                    .unwrap_or(tx + 1000.0);
                (tx, ack_time)
            }
            MessageStatus::Received => {
                // Message reçu d'ailleurs
                let rx_time = message
                    .receive_time
                    .map(|t| t.timestamp_millis() as f64)
                    .unwrap_or(tx);
                (tx, rx_time)
            }
            MessageStatus::Failed => {
                // Message échoué - boîte très courte, animation arrêtée
                (tx, tx + 100.0)
            }
            _ => {
                // Pas d'ACK encore - animer la boîte
                (tx, now)
            }
        };

        let box_elem = BoxElem::new(
            y_position,
            BoxSpread::new(
                start_time,
                start_time,
                start_time + (end_time - start_time) / 2.0,
                end_time,
                end_time,
            ),
        )
        .name(box_name);

        (
            box_elem,
            message.sender_uuid.clone(),
            status_color,
            status_text,
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
        messages: &VecDeque<ChatMessage>,
        local_peer_uuid: &str,
        peer_manager: &PeerManager,
    ) {
        let now = Local::now().timestamp_millis() as f64;
        let peers = peer_manager.peers();

        self.update_participants(messages, peers, local_peer_uuid);

        let reset_requested = ui
            .horizontal(|ui| {
                ui.add_space(20.0); // Padding left de 20px
                ui.button("Reset").clicked()
            })
            .inner;
        ui.add_space(3.0); // Marge bottom

        let filtered_messages: Vec<&ChatMessage> = messages
            .iter()
            .filter(|msg| self.should_show_message(msg))
            .collect();

        // Group messages by sender
        let mut boxes_by_participant: HashMap<String, Vec<(BoxElem, Color32, String)>> =
            HashMap::new();
        for (index, message) in filtered_messages.iter().enumerate() {
            let (box_elem, sender_uuid, status_color, status_text) =
                self.create_box_element(message, index as f64, now);
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

        Plot::new("DTChat Timeline")
            .allow_zoom(true)
            .allow_drag(true)
            .legend(Legend::default().position(egui_plot::Corner::RightTop))
            .show_x(true)
            .show_y(false)
            .include_y(-0.5)
            .include_y(num_messages + 0.5)
            .auto_reset(reset_requested)
            .x_axis_formatter(|mark, _range| {
                DateTime::<Utc>::from_timestamp_millis(mark.value as i64)
                    .map(|dt| dt.with_timezone(&Local).format("%H:%M:%S").to_string())
                    .unwrap_or_else(|| format!("{:.0}", mark.value))
            })
            .y_axis_formatter(|_mark, _range| String::new())
            .height(plot_height)
            .show(ui, |plot_ui| {
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
                                    let tx_time = DateTime::<Utc>::from_timestamp_millis(
                                        bar.spread.quartile1 as i64,
                                    )
                                    .unwrap();
                                    let rx_time = DateTime::<Utc>::from_timestamp_millis(
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
                                        ts_to_str(&tx_time, date, true, None),
                                        ts_to_str(&rx_time, date, true, None),
                                        status_info
                                    )
                                }));

                            plot_ui.box_plot(box_plot);
                        }
                    }
                }
            });

        ui.ctx().request_repaint();
    }
}

impl Default for MessageGraphView {
    fn default() -> Self {
        Self::new()
    }
}
