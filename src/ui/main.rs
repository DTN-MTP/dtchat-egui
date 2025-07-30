use crate::app::EventHandler;
use crate::domain::peer::PeerManager;
use crate::ui::components::header::Header;
use crate::ui::components::message_forge::MessageForge;
use crate::ui::components::message_settings_bar::MessageSettingsBar;
use crate::ui::views::graph::MessageGraphView;
use crate::ui::views::list::MessageListView;
use crate::ui::views::settings::SettingsView;
use dtchat_backend::dtchat::ChatModel;
use dtchat_backend::message::{
    filter_by_network_endpoint, sort_with_strategy, ChatMessage, SortStrategy,
};
use eframe::egui;
use egui::{CentralPanel, TopBottomPanel, Ui};
use socket_engine::endpoint::EndpointProto;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

#[derive(PartialEq, Clone, Copy)]
pub enum ViewType {
    MessageGraph,
    MessageList,
    Settings,
}

impl ViewType {
    pub fn name(&self) -> &'static str {
        match self {
            ViewType::MessageGraph => "Graph",
            ViewType::MessageList => "List",
            ViewType::Settings => "Events",
        }
    }
}

pub struct Views {
    pub message_graph: MessageGraphView,
    pub message_list: MessageListView,
    pub settings: SettingsView,
}

impl Views {
    pub fn new() -> Self {
        Self {
            message_graph: MessageGraphView::new(),
            message_list: MessageListView::new(),
            settings: SettingsView::new(),
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub enum ProtoFilter {
    NoFilter,
    Filter(EndpointProto),
}

impl std::fmt::Display for ProtoFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProtoFilter::NoFilter => write!(f, "All"),
            ProtoFilter::Filter(proto) => write!(f, "{}", proto),
        }
    }
}

pub struct UIState {
    pub message_forge: MessageForge,
    pub message_settings_bar: MessageSettingsBar,
    pub views: Views,
    pub current_view: ViewType,
    pub header: Header,
    pub sort_strategy: SortStrategy,
    pub request_sort_strategy: bool,
    pub protocol_filter: ProtoFilter,
    pub request_protocol_filter: bool,
    pub pbat_support_by_model: bool,

    // data
    pub messages: Vec<ChatMessage>,
    pub messages_to_display: Vec<ChatMessage>,
    peer_manager: PeerManager,
}

impl UIState {
    pub fn new(peer_manager: PeerManager, messages: Vec<ChatMessage>) -> Self {
        Self {
            message_forge: MessageForge::new(),
            message_settings_bar: MessageSettingsBar::new(),
            views: Views::new(),
            current_view: ViewType::MessageGraph,
            header: Header::new(),
            request_sort_strategy: true,
            sort_strategy: SortStrategy::Standard,
            request_protocol_filter: true,
            protocol_filter: ProtoFilter::NoFilter,
            messages_to_display: vec![],
            messages,
            peer_manager,
            pbat_support_by_model: false,
        }
    }

    pub fn will_lock_model_to_refresh(&mut self, chat_model: &Arc<Mutex<ChatModel>>) {
        self.messages = chat_model.lock().unwrap().get_all_messages();
        self.pbat_support_by_model = chat_model.lock().unwrap().is_pbat_enabled();
        self.request_protocol_filter = true;
        self.request_sort_strategy = true;
    }

    pub fn show(
        &mut self,
        ui: &mut Ui,
        events: &Arc<Mutex<EventHandler>>,
        chat_model: &Arc<Mutex<ChatModel>>,
    ) -> Option<(String, String)> {
        let peer_manager = &self.peer_manager;
        let local_peer = self.peer_manager.local_peer();
        let (network_events, app_events) = if let Ok(handler) = events.lock() {
            (
                handler.network_events().clone(),
                handler.app_events().clone(),
            )
        } else {
            (VecDeque::new(), VecDeque::new())
        };

        TopBottomPanel::top("header").show_inside(ui, |ui| {
            self.header.show(ui, local_peer);
        });

        TopBottomPanel::top("message_settings_bar").show_inside(ui, |ui| {
            self.message_settings_bar.show(
                ui,
                &mut self.current_view,
                &mut self.sort_strategy,
                &mut self.request_sort_strategy,
                &mut self.protocol_filter,
                &mut self.request_protocol_filter,
                peer_manager,
                &local_peer,
            );
        });

        TopBottomPanel::bottom("message_forge_panel").show_inside(ui, |ui| {
            self.message_forge.show(
                ui,
                peer_manager.peers(),
                &local_peer.uuid,
                chat_model,
                peer_manager,
                self.pbat_support_by_model,
            );
        });

        CentralPanel::default().show_inside(ui, |ui| {
            // Préparer les messages avec la stratégie de tri appropriée
            if self.request_protocol_filter {
                self.messages_to_display = match &self.protocol_filter {
                    ProtoFilter::NoFilter => self.messages.clone(),
                    ProtoFilter::Filter(by_proto) => {
                        filter_by_network_endpoint(&self.messages, by_proto.clone())
                    }
                };
                self.request_protocol_filter = false;
            }
            if self.request_sort_strategy {
                sort_with_strategy(&mut self.messages_to_display, self.sort_strategy.clone());
                self.request_sort_strategy = false;
            }

            match self.current_view {
                ViewType::MessageGraph => {
                    self.views.message_graph.show(
                        ui,
                        &self.messages_to_display,
                        &local_peer.uuid,
                        &peer_manager,
                    );
                }
                ViewType::MessageList => {
                    self.views.message_list.show(
                        ui,
                        &self.messages_to_display,
                        &local_peer,
                        &peer_manager,
                    );
                }
                ViewType::Settings => {
                    self.views.settings.show(ui, &network_events, &app_events);
                }
            }
        });

        None
    }
}
