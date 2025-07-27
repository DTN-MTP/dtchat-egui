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
            ViewType::Settings => "Settings",
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

pub struct UIState {
    pub message_forge: MessageForge,
    pub message_settings_bar: MessageSettingsBar,
    pub views: Views,
    pub current_view: ViewType,
    pub header: Header,
    pub selected_peer_for_relative: Option<String>,
    pub selected_protocol_filter: Option<EndpointProto>,
}

impl Default for UIState {
    fn default() -> Self {
        Self {
            message_forge: MessageForge::new(),
            message_settings_bar: MessageSettingsBar::new(),
            views: Views::new(),
            current_view: ViewType::MessageGraph,
            header: Header::new(),
            selected_peer_for_relative: None,
            selected_protocol_filter: None,
        }
    }
}

impl UIState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn show(
        &mut self,
        ui: &mut Ui,
        peer_manager: &PeerManager,
        events: &Arc<Mutex<EventHandler>>,
        chat_model: &Arc<Mutex<ChatModel>>,
    ) -> Option<(String, String)> {
        let local_peer = peer_manager.local_peer();
        let messages = if let Ok(model) = chat_model.lock() {
            VecDeque::from(model.get_all_messages())
        } else {
            VecDeque::new()
        };

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
                &mut self.selected_peer_for_relative,
                &mut self.selected_protocol_filter,
                peer_manager,
                &local_peer.uuid,
            );
        });

        TopBottomPanel::bottom("message_forge_panel").show_inside(ui, |ui| {
            self.message_forge.show(
                ui,
                peer_manager.peers(),
                &local_peer.uuid,
                chat_model,
                peer_manager,
            );
        });

        CentralPanel::default().show_inside(ui, |ui| {
            // Préparer les messages avec la stratégie de tri appropriée
            let sorted_messages = {
                let mut msg_vec: Vec<ChatMessage> = messages.iter().cloned().collect();

                // Appliquer le filtre par protocole d'abord
                if let Some(filter_by) = &self.selected_protocol_filter {
                    msg_vec = filter_by_network_endpoint(&msg_vec, filter_by.clone());
                }

                // Ensuite appliquer le tri
                if let Some(ref peer_uuid) = self.selected_peer_for_relative {
                    sort_with_strategy(&mut msg_vec, SortStrategy::Relative(peer_uuid.clone()));
                } else {
                    sort_with_strategy(&mut msg_vec, SortStrategy::Standard);
                }

                // Reconvertir en VecDeque
                let mut sorted_deque = VecDeque::new();
                for msg in msg_vec {
                    sorted_deque.push_back(msg);
                }
                sorted_deque
            };

            match self.current_view {
                ViewType::MessageGraph => {
                    self.views.message_graph.show(
                        ui,
                        &sorted_messages,
                        &local_peer.uuid,
                        &peer_manager,
                    );
                }
                ViewType::MessageList => {
                    self.views
                        .message_list
                        .show(ui, &sorted_messages, &local_peer, &peer_manager);
                }
                ViewType::Settings => {
                    self.views.settings.show(ui, &network_events, &app_events);
                }
            }
        });

        None
    }
}
