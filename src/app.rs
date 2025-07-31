use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use chrono::{DateTime, Utc};
use dtchat_backend::dtchat::ChatModel;
use dtchat_backend::event::{
    AppEventObserver, ChatAppErrorEvent, ChatAppEvent, ChatAppInfoEvent, NetworkErrorEvent,
    NetworkEvent,
};
use dtchat_backend::event::{ConnectionEvent, DataEvent};
use eframe::{egui, App};
use egui::CentralPanel;

use crate::domain::peer::{Peer, PeerManager};
use crate::ui::main::UIState;
use crate::utils::uuid::safe_message_id_display;

#[derive(Clone, Debug)]
pub enum EventLevel {
    Info,
    Debug,
    Error,
}

#[derive(Clone, Debug)]
pub struct DisplayEvent {
    pub level: EventLevel,
    pub message: String,
    pub timestamp: DateTime<Utc>,
}

impl DisplayEvent {
    pub fn new(level: EventLevel, message: String) -> Self {
        Self {
            level,
            message,
            timestamp: Utc::now(),
        }
    }
}

#[derive(Default)]
pub struct EventHandler {
    pub network_events: VecDeque<DisplayEvent>,
    pub app_events: VecDeque<DisplayEvent>,
    pub max_events_per_category: usize,
    pub ctx: Option<egui::Context>,
    pub refresh_model_request: bool,
}

impl EventHandler {
    pub fn new(max_events_per_category: usize) -> Self {
        Self {
            network_events: VecDeque::new(),
            app_events: VecDeque::new(),
            max_events_per_category,
            ctx: None,
            refresh_model_request: true,
        }
    }

    pub fn set_context(&mut self, ctx: egui::Context) {
        self.ctx = Some(ctx);
    }
    pub fn add_network_event(&mut self, level: EventLevel, message: String) {
        let event = DisplayEvent::new(level, message);
        self.network_events.push_back(event);

        if self.network_events.len() > self.max_events_per_category {
            self.network_events.pop_front();
        }
    }
    pub fn add_app_event(&mut self, level: EventLevel, message: String) {
        let event = DisplayEvent::new(level, message);
        self.app_events.push_back(event);

        if self.app_events.len() > self.max_events_per_category {
            self.app_events.pop_front();
        }
    }
    pub fn network_events(&self) -> VecDeque<DisplayEvent> {
        self.network_events.clone()
    }
    pub fn app_events(&self) -> VecDeque<DisplayEvent> {
        self.app_events.clone()
    }

    pub fn handle_chat_app_event(&mut self, app_event: ChatAppEvent) {
        match app_event {
            ChatAppEvent::Info(info_event) => match info_event {
                ChatAppInfoEvent::Sending(msg) => {
                    self.add_app_event(
                        EventLevel::Info,
                        format!("Sending: {}", safe_message_id_display(&msg.uuid)),
                    );
                }
                ChatAppInfoEvent::Sent(msg) => {
                    self.add_app_event(
                        EventLevel::Info,
                        format!("Sent: {}", safe_message_id_display(&msg.uuid)),
                    );
                }
                ChatAppInfoEvent::Received(msg) => {
                    self.add_app_event(
                        EventLevel::Info,
                        format!("Received: {}", safe_message_id_display(&msg.uuid)),
                    );
                }
                ChatAppInfoEvent::AckSent(msg, peer_id) => {
                    self.add_app_event(
                        EventLevel::Info,
                        format!(
                            "ACK sent for {} to {}",
                            safe_message_id_display(&msg.uuid),
                            peer_id
                        ),
                    );
                }
                ChatAppInfoEvent::AckReceived(msg) => {
                    self.add_app_event(
                        EventLevel::Info,
                        format!("ACK received for {}", safe_message_id_display(&msg.uuid)),
                    );
                }
            },
            ChatAppEvent::Error(error_event) => match error_event {
                ChatAppErrorEvent::ProtocolDecode(error) => {
                    self.add_app_event(EventLevel::Error, format!("Protocol decode: {}", error));
                }
                ChatAppErrorEvent::ProtocolEncode(error) => {
                    self.add_app_event(EventLevel::Error, format!("Protocol encode: {}", error));
                }
                ChatAppErrorEvent::InvalidMessage(error) => {
                    self.add_app_event(EventLevel::Error, format!("Invalid message: {}", error));
                }
                ChatAppErrorEvent::MessageNotFound(msg_id) => {
                    self.add_app_event(
                        EventLevel::Error,
                        format!("Message not found: {}", safe_message_id_display(&msg_id)),
                    );
                }
                ChatAppErrorEvent::PeerNotFound(peer_id) => {
                    self.add_app_event(EventLevel::Error, format!("Peer not found: {}", peer_id));
                }
                ChatAppErrorEvent::NoEngineAttached => {
                    self.add_app_event(EventLevel::Error, "No engine attached".to_string());
                }
                ChatAppErrorEvent::InternalError(error) => {
                    self.add_app_event(EventLevel::Error, format!("Internal: {}", error));
                }
                ChatAppErrorEvent::HostNotReachable(error) => {
                    self.add_network_event(
                        EventLevel::Error,
                        format!("Connection failed: {}", error),
                    );
                    self.add_app_event(EventLevel::Error, format!("Host not reachable: {}", error));
                }
            },
            ChatAppEvent::SocketEngineInfo(network_event) => {
                let (level, event_text) = match network_event {
                    NetworkEvent::Data(data_event) => match data_event {
                        DataEvent::Received { data, from } => (
                            EventLevel::Info,
                            format!("Received {} bytes from {}", data.len(), from.to_string()),
                        ),
                        DataEvent::Sent {
                            message_id,
                            to,
                            bytes_sent,
                        } => (
                            EventLevel::Info,
                            format!(
                                "Sent {} bytes to {} (token: {})",
                                bytes_sent,
                                to.to_string(),
                                safe_message_id_display(&message_id)
                            ),
                        ),
                        DataEvent::Sending {
                            message_id,
                            to,
                            bytes,
                        } => (
                            EventLevel::Info,
                            format!(
                                "Sending {} bytes to {} (token: {})",
                                bytes,
                                to.to_string(),
                                safe_message_id_display(&message_id)
                            ),
                        ),
                    },
                    NetworkEvent::Connection(connection_event) => match connection_event {
                        // TODO: not working even from dtchat-backend and socket-engine
                        ConnectionEvent::ListenerStarted { endpoint } => (
                            EventLevel::Info,
                            format!("Listening on {}", endpoint.to_string()),
                        ),
                        ConnectionEvent::Established { remote } => {
                            let client_addr = remote.endpoint;
                            (
                                EventLevel::Debug,
                                format!("Connection established (client: {})", client_addr),
                            )
                        }
                        ConnectionEvent::Closed { remote } => {
                            let message = match remote {
                                Some(remote_ep) => {
                                    let client_addr = remote_ep.endpoint;
                                    format!("Connection closed (client: {})", client_addr)
                                }
                                None => "Connection closed (no client info)".to_string(),
                            };
                            (EventLevel::Debug, message)
                        }
                    },
                };

                self.add_network_event(level, event_text);
            }
            ChatAppEvent::SocketEngineError(network_error) => {
                let error_text = match network_error {
                    NetworkErrorEvent::SocketError(socket_error) => {
                        format!("Socket error: {:?}", socket_error)
                    }
                };
                self.add_network_event(EventLevel::Error, error_text);
            }
        }

        self.refresh_model_request = true;
    }
}

impl AppEventObserver for EventHandler {
    fn on_event(&mut self, event: ChatAppEvent) {
        self.handle_chat_app_event(event);
    }
}

pub struct DTChatApp {
    pub event_handler: Arc<Mutex<EventHandler>>,
    pub chat_model: Arc<Mutex<ChatModel>>,
    pub ui: UIState,
    // TODO: those 2 must be retrieve from the model
    context_initialized: bool,
}

impl DTChatApp {
    pub fn new(
        chat_model: Arc<Mutex<ChatModel>>,
        local_peer: Peer,
        dist_peers: Vec<Peer>,
        event_handler: Arc<Mutex<EventHandler>>,
    ) -> Self {
        Self {
            event_handler,
            chat_model,
            ui: UIState::new(PeerManager::new(local_peer.clone(), dist_peers.clone())),
            context_initialized: false,
        }
    }
}

impl App for DTChatApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.context_initialized {
            if let Ok(mut handler) = self.event_handler.lock() {
                handler.set_context(ctx.clone());
                handler.add_app_event(EventLevel::Info, "DTChat GUI initialized".to_string());
            }
            self.context_initialized = true;
        }

        let mut update_request_with_events: Option<(
            VecDeque<DisplayEvent>,
            VecDeque<DisplayEvent>,
        )> = None;
        // Update the mirror of the model if something changed
        if let Ok(mut handler) = self.event_handler.lock() {
            if handler.refresh_model_request {
                handler.refresh_model_request = false;
                update_request_with_events = Some((handler.app_events(), handler.network_events()));
            }
        }
        if let Some((app_events, net_events)) = update_request_with_events {
            self.ui
                .will_lock_model_to_refresh(&self.chat_model, app_events, net_events);
        }

        CentralPanel::default().show(ctx, |ui| {
            self.ui.show(ui, &self.chat_model);
        });

        ctx.request_repaint();
    }
}
