use crate::app::DisplayEvent;
use crate::header_view::HeaderView;
use crate::messages::MessageView;
use crate::network_view::NetworkView;
use dtchat_backend::dtchat::{ChatModel, Peer, Room};
use dtchat_backend::message::ChatMessage;
use dtchat_backend::time::DTChatTime;
use eframe::egui;
use egui::{TopBottomPanel, Ui};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

#[derive(PartialEq, Clone, Copy)]
pub enum ViewType {
    Messages,
    Network,
}

pub struct MirroredData {
    pub messages: Vec<ChatMessage>,
    pub app_events: VecDeque<DisplayEvent>,
    pub network_events: VecDeque<DisplayEvent>,
    pub local_peer: Peer,
    pub other_peers: HashMap<String, Peer>,
    pub rooms: HashMap<String, Room>,
    pub pbat_support_by_model: bool,
}

pub struct MainView {
    //  views
    pub header_view: HeaderView,
    pub message_view: MessageView,
    pub network_view: NetworkView,

    // current_view
    pub current_view: ViewType,

    // data
    pub data: MirroredData,
}

impl MainView {
    pub fn new(local: Peer) -> Self {
        Self {
            header_view: HeaderView::new(),
            message_view: MessageView::new(),
            network_view: NetworkView {},
            current_view: ViewType::Messages,
            data: MirroredData {
                messages: vec![],
                app_events: VecDeque::new(),
                network_events: VecDeque::new(),
                local_peer: local,
                other_peers: HashMap::new(),
                rooms: HashMap::new(),
                pbat_support_by_model: false,
            },
        }
    }

    pub fn will_lock_model_to_refresh(
        &mut self,
        chat_model: &Arc<Mutex<ChatModel>>,
        app_events: VecDeque<DisplayEvent>,
        network_events: VecDeque<DisplayEvent>,
    ) {
        let sticky = self.data.messages.len() == self.message_view.max_message_count;
        self.data.other_peers = chat_model.lock().unwrap().get_other_peers();
        self.data.messages = chat_model.lock().unwrap().get_all_messages();
        self.data.pbat_support_by_model = chat_model.lock().unwrap().is_pbat_enabled();
        self.data.rooms = chat_model.lock().unwrap().get_rooms();
        if sticky {
            self.message_view.max_message_count = self.data.messages.len()
        }

        self.data.app_events.extend(app_events);
        self.data.network_events.extend(network_events);

        // Delegate sorting
        self.message_view.request_protocol_filter = true;
        self.message_view.request_sort_strategy = true;
    }

    pub fn show(&mut self, ui: &mut Ui, chat_model: &Arc<Mutex<ChatModel>>) {
        let current_time = DTChatTime::now();

        TopBottomPanel::top("header").show_inside(ui, |ui| {
            self.header_view
                .show(ui, &self.data.local_peer, current_time);

            ui.add_space(3.0);
            ui.horizontal(|ui| {
                ui.selectable_value(
                    &mut self.current_view,
                    ViewType::Messages,
                    "\u{2709} Messages",
                );
                ui.selectable_value(&mut self.current_view, ViewType::Network, "🖧 Network");
            });
            ui.add_space(3.0);
        });

        match self.current_view {
            ViewType::Messages => {
                // Although part of the message_view, we do not want to give access to the model
                // So we show it from here..
                TopBottomPanel::bottom("message_forge_panel").show_inside(ui, |ui| {
                    self.message_view.message_prompt_view.show(
                        ui,
                        &self.data.other_peers,
                        chat_model,
                        self.data.pbat_support_by_model,
                        // ..as if it was in the messages mod
                        &self.message_view.current_room,
                        &mut self.message_view.current_peer,
                    );
                });

                // For correct height calculation, draw central panel at last
                self.message_view.show(&mut self.data, &current_time, ui);
            }
            ViewType::Network => {
                self.network_view
                    .show(ui, &self.data.network_events, &self.data.app_events);
            }
        }
    }
}
