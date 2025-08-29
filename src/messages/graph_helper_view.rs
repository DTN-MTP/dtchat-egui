use dtchat_backend::message::MessageStatus;
use egui::Color32;
use egui_plot::{BoxElem, BoxPlot, BoxSpread, Legend, Plot, VLine};

use crate::utils::font::StatusDisplayHelper;

#[derive(Clone)]
pub struct GraphHelperView {
    now: f64,
    min_x: f64,
    max_x: f64,
    boxes: Vec<(MessageStatus, BoxElem, String)>,
}

impl GraphHelperView {
    pub fn new() -> Self {
        let now = 3.0;

        // Sent and received
        let sending_0 = 0.0;
        let sent_0 = 0.0;
        let received_0 = 1.4;
        let pred_0 = 1.4;

        // Sent and received/Acked
        let sending_1 = 0.0;
        let sent_1 = 0.2;
        let received_1 = 2.0;
        let pred_1 = 2.0;

        // Sent, in transmission after pred
        let sending_2 = 0.5;
        let sent_2 = 0.6;
        let received_2 = now;
        let pred_2 = 2.4;

        // Sent, in transmission before pred
        let sending_3 = 1.0;
        let sent_3 = 1.3;
        let received_3 = now;
        let pred_3 = 3.5;

        let sending_4 = 1.3;
        let sent_4 = 1.5;
        let received_4 = now;
        let pred_4 = now;

        let box_elem_0 = (
            MessageStatus::Received,
            BoxElem::new(
                0.0,
                BoxSpread::new(sending_0, sent_0, pred_0, received_0, pred_0),
            )
            .name("Peer"),
            "Received messages appear as cyan boxes without whiskers. The sender is indicated in the legend. The box spans from the message creation time to its reception time.".to_string(),
        );

        let box_elem_1 = (
            MessageStatus::ReceivedByPeer,
            BoxElem::new(
                0.0,
                BoxSpread::new(sending_1, sent_1, pred_1, received_1, pred_1),
            )
            .name("Me"),
             "Sent messages (acknowledged by the peer) appear as green boxes. They may include a left whisker, representing the delay between delegating transmission to the socket (asynchronous) and the socket’s confirmation that transmission is complete.".to_string(),

                   );

        let box_elem_2 = (
            MessageStatus::Sent,
            BoxElem::new(
                0.0,
                BoxSpread::new(sending_2, sent_2, pred_2, received_2, pred_2),
            )
            .name("Me"),
             "Because of round-trip delays, once the current time surpasses the predicted arrival time, the whisker is replaced by the boxplot median.".to_string(),

        );

        let box_elem_3 = (
            MessageStatus::Sent,
            BoxElem::new(
                0.0,
                BoxSpread::new(sending_3, sent_3, pred_3, received_3, pred_3),
            )
            .name("Me"),
            "Sent messages (not acknowledged) appear as light grey boxes, spanning from the sending time to the current time. If enabled, a right whisker shows the predicted arrival time. ".to_string(),
        );

        let box_elem_4 = (
            MessageStatus::Failed,
            BoxElem::new(
                0.0,
                BoxSpread::new(sending_4, sent_4, pred_4, received_4, pred_4),
            )
            .name("Me"),
            "Messages that encounter an error upon sending appear as red boxes, spanning from the sending time to the current time.".to_string(),
        );

        Self {
            now,
            min_x: sending_1,
            max_x: pred_3,
            boxes: vec![box_elem_0, box_elem_1, box_elem_3, box_elem_2, box_elem_4],
        }
    }

    fn show_single_plot(
        &self,
        ui: &mut egui::Ui,
        idx: usize,
        status: &MessageStatus,
        box_elem: &BoxElem,
        description: &String,
    ) {
        ui.separator();
        ui.label(description);
        Plot::new(idx)
            .allow_zoom(false)
            .allow_drag(false)
            .legend(Legend::default().position(egui_plot::Corner::LeftTop))
            .include_x(self.min_x - 1.5)
            .include_x(self.max_x)
            .include_y(0.5)
            .include_y(-0.5)
            .show_x(false)
            .show_y(false)
            .show_grid(false)
            .height(50.0)
            .label_formatter(|_name, _value| "".to_string())
            .custom_x_axes(vec![])
            .custom_y_axes(vec![])
            .show(ui, |plot_ui| {
                plot_ui.vline(
                    VLine::new(self.now)
                        .color(Color32::ORANGE)
                        .name("Current Time"),
                );

                let status_text = status.get_icon_text(&box_elem.name);
                let status_color = status.get_color();

                let box_plot = BoxPlot::new(vec![box_elem.clone()])
                    .name(status_text.clone())
                    .color(status_color)
                    .horizontal()
                    .allow_hover(true)
                    .element_formatter(Box::new(move |_bar, _bar_chart| "".to_owned()));

                plot_ui.box_plot(box_plot);
            });
    }

    pub fn show(&self, ui: &mut egui::Ui) {
        ui.label("Drag to move. Scroll to move up/down. Ctrl & scroll to zoom in/out. Enable the \"Auto bounds\" option to adapt the view dynamically. Click on the legend to display/hide elements. Hovering over any box displays the corresponding basic message information.");
        ui.add_space(8.0);
        for (idx, (status, box_elem, descr)) in self.boxes.iter().enumerate() {
            self.show_single_plot(ui, idx, status, box_elem, descr);
        }
    }
}
