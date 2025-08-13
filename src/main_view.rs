use crate::app::DisplayEvent;
use crate::header_view::HeaderView;
use crate::messages::MessagesView;
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
    pub message_view: MessagesView,
    pub network_view: NetworkView,

    // current_view
    pub current_view: ViewType,

    // data
    pub data: MirroredData,
}

impl MainView {
    pub fn new(local: Peer, model: Arc<Mutex<ChatModel>>) -> Self {
        Self {
            header_view: HeaderView::new(),
            message_view: MessagesView::new(model),
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
        self.data.other_peers = chat_model.lock().unwrap().get_other_peers();
        self.data.messages = chat_model.lock().unwrap().get_all_messages();
        self.data.pbat_support_by_model = chat_model.lock().unwrap().is_pbat_enabled();
        self.data.rooms = chat_model.lock().unwrap().get_rooms();

        self.message_view.manage_message(&self.data);

        self.data.app_events.extend(app_events);
        self.data.network_events.extend(network_events);
    }

    pub fn show(&mut self, ctx: &egui::Context, ui: &mut Ui) {
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
                self.message_view
                    .show(ctx, &mut self.data, &current_time, ui);
            }
            ViewType::Network => {
                self.network_view
                    .show(ui, &self.data.network_events, &self.data.app_events);
            }
        }
    }
}
