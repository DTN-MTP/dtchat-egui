use dtchat_backend::dtchat::Peer;
use dtchat_backend::message::{ChatMessage, MessageStatus};
use dtchat_backend::time::DTChatTime;
use egui::Color32;
use egui_plot::{AxisHints, BoxElem, BoxPlot, BoxSpread, GridMark, Legend, Plot, VLine};
use std::collections::HashMap;
use std::ops::RangeInclusive;
#[derive(Clone)]
pub struct MessageGraphView {
    auto_bounds: bool,
    show_current_time: bool,
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

trait AsIndex {
    fn as_index(&self) -> usize;
}

impl AsIndex for MessageStatus {
    fn as_index(&self) -> usize {
        match self {
            MessageStatus::Sending => 0,
            MessageStatus::Sent => 1,
            MessageStatus::ReceivedByPeer => 2,
            MessageStatus::Failed => 3,
            MessageStatus::Received => 4,
        }
    }
}

impl MessageGraphView {
    pub fn new() -> Self {
        Self {
            auto_bounds: true,
            show_current_time: true,
            hovered: false,
        }
    }

    fn create_box_element(
        &self,
        message: &ChatMessage,
        y_position: f64,
        now: f64,
    ) -> (BoxElem, f64, f64) {
        let box_name = self.truncate_text(&message.content_as_string(), 30);

        let sending = message.send_time.timestamp_millis() as f64;
        let mut med = sending;
        let mut sent = match message.send_completed {
            Some(val) => val.timestamp_millis() as f64,
            None => now,
        };
        let recv = match message.receive_time {
            Some(val) => val.timestamp_millis() as f64,
            None => now,
        };
        let pred = match message.predicted_arrival_time {
            Some(val) => {
                let p = val.timestamp_millis() as f64;
                med = p;
                p
            }
            None => recv,
        };

        // For visibility
        if message.status == MessageStatus::Failed {
            sent = sending;
        }

        let box_elem =
            BoxElem::new(y_position, BoxSpread::new(sending, sent, med, recv, pred)).name(box_name);

        (box_elem, sending, pred)
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
        local_peer: &Peer,
        other_peers: &HashMap<String, Peer>,
        current_time: &DTChatTime,
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
        let peers = &other_peers;

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

        // we try to find some bounds
        let mut first_message = now;
        let mut last_message = now;
        // Group messages by sender and status (status converted to index)
        let mut boxes_by_participant: HashMap<(String, usize), (MessageStatus, Vec<BoxElem>)> =
            HashMap::new();
        for (index, message) in messages.iter().enumerate() {
            let (box_elem, from, to) = self.create_box_element(message, index as f64, now);
            if from < first_message {
                first_message = from;
            }
            if to > last_message {
                last_message = to;
            }

            boxes_by_participant
                .entry((message.sender_uuid.clone(), message.status.as_index()))
                .or_insert((message.status.clone(), Vec::new()))
                .1
                .push(box_elem);
        }

        let num_messages = if messages.is_empty() {
            1.0
        } else {
            messages.len() as f64
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

                for ((participant_uuid, _status_as_index), (status, boxes_with_colors)) in
                    boxes_by_participant
                {
                    let participant_name = if local_peer.uuid == *participant_uuid {
                        "Me".to_string()
                    } else {
                        match peers.get(&participant_uuid) {
                            Some(p) => p.name.clone(),
                            None => "unknown".to_string(),
                        }
                    };

                    let formatter_name = participant_name.clone();

                    // Legend text
                    let status_text = match status {
                        MessageStatus::Failed => format!("{} (FAILED)", participant_name),
                        MessageStatus::ReceivedByPeer => format!("{} (ACKED)", participant_name),
                        MessageStatus::Sending => format!("{} (SENDING)", participant_name),
                        MessageStatus::Received => format!("{} (RECEIVED)", participant_name),
                        MessageStatus::Sent => format!("{} (SENT)", participant_name),
                    }
                    .to_string();

                    let status_color = match status {
                        MessageStatus::Failed => Color32::RED,
                        MessageStatus::ReceivedByPeer => Color32::GREEN,
                        MessageStatus::Sent => Color32::LIGHT_GRAY,
                        MessageStatus::Sending => Color32::YELLOW,
                        MessageStatus::Received => Color32::LIGHT_BLUE,
                    };

                    let box_plot = BoxPlot::new(boxes_with_colors)
                        .name(status_text.clone())
                        .color(status_color)
                        .horizontal()
                        .allow_hover(true)
                        .element_formatter(Box::new(move |bar, _bar_chart| {
                            let tx_time =
                                DTChatTime::from_timestamp_millis(bar.spread.quartile1 as i64)
                                    .unwrap();
                            let rx_time =
                                DTChatTime::from_timestamp_millis(bar.spread.quartile3 as i64)
                                    .unwrap();
                            let date = tx_time.date_naive() != rx_time.date_naive();

                            let status_info = format!("\nStatus: {}", status_text);

                            format!(
                                "Message: {}\nSent by {}\ntx time: {}\nrx_time: {}{}",
                                bar.name,
                                formatter_name,
                                tx_time.ts_to_str(date, true, None, &chrono::Local),
                                rx_time.ts_to_str(date, true, None, &chrono::Local),
                                status_info
                            )
                        }));

                    plot_ui.box_plot(box_plot);
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
