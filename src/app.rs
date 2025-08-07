use crate::main_view::MainView;
use crate::utils::text::PrettyStr;
use crate::utils::uuid::safe_id_display;
use dtchat_backend::dtchat::ChatModel;
use dtchat_backend::event::{
    AppEventObserver, ChatAppErrorEvent, ChatAppEvent, ChatAppInfoEvent,
    ErrorEvent::{ConnectionFailed, ReceiveFailed, SendFailed, SocketError},
    NetworkErrorEvent, NetworkEvent,
};
use dtchat_backend::event::{ConnectionEvent, DataEvent};
use dtchat_backend::time::DTChatTime;
use eframe::{egui, App};
use egui::{CentralPanel, Color32};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug)]
pub enum EventLevel {
    Info,
    Debug,
    Error,
}

#[derive(Clone, Debug)]
pub struct DisplayEvent {
    level: EventLevel,
    message: String,
    timestamp: DTChatTime,
}

impl DisplayEvent {
    pub fn new(level: EventLevel, message: String) -> Self {
        Self {
            level,
            message,
            timestamp: DTChatTime::now(),
        }
    }
    pub fn get_color(&self) -> Color32 {
        match self.level {
            EventLevel::Error => Color32::RED,
            EventLevel::Info => Color32::LIGHT_BLUE,
            EventLevel::Debug => Color32::GRAY,
        }
    }
}

impl PrettyStr for DisplayEvent {
    fn to_pretty_str(&self) -> String {
        format!(
            "[{}] {}",
            self.timestamp.ts_to_str(false, true, None, &chrono::Local),
            self.message
        )
    }
}

pub struct EventHandler {
    pub network_events: VecDeque<DisplayEvent>,
    pub app_events: VecDeque<DisplayEvent>,
    pub max_events_per_category: usize,
    pub refresh_model_request: bool,
}

impl EventHandler {
    pub fn new(max_events_per_category: usize) -> Self {
        Self {
            network_events: VecDeque::new(),
            app_events: VecDeque::new(),
            max_events_per_category,
            refresh_model_request: true,
        }
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
            ChatAppEvent::Message(info_event) => match info_event {
                ChatAppInfoEvent::Sending(msg) => {
                    self.add_app_event(
                        EventLevel::Info,
                        format!(
                            "Sending msg {} to room {}",
                            safe_id_display(&msg.uuid),
                            safe_id_display(&msg.room_uuid)
                        ),
                    );
                }
                ChatAppInfoEvent::Sent(msg) => {
                    self.add_app_event(
                        EventLevel::Info,
                        format!(
                            "Sent msg {} to room {}",
                            safe_id_display(&msg.uuid),
                            safe_id_display(&msg.room_uuid)
                        ),
                    );
                }
                ChatAppInfoEvent::Received(msg) => {
                    self.add_app_event(
                        EventLevel::Info,
                        format!(
                            "Received: msg {} from {} for room {}",
                            safe_id_display(&msg.uuid),
                            safe_id_display(&msg.sender_uuid),
                            safe_id_display(&msg.room_uuid)
                        ),
                    );
                }
                ChatAppInfoEvent::AckSent(msg, peer_id) => {
                    self.add_app_event(
                        EventLevel::Info,
                        format!(
                            "ACK sent for msg {} from room {} to {}",
                            safe_id_display(&msg.uuid),
                            safe_id_display(&msg.room_uuid),
                            peer_id
                        ),
                    );
                }
                ChatAppInfoEvent::AckReceived(msg) => {
                    self.add_app_event(
                        EventLevel::Info,
                        format!("ACK received for msg {}", safe_id_display(&msg.uuid)),
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
                        format!("Message not found: {}", safe_id_display(&msg_id)),
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
            },
            ChatAppEvent::SocketEngineInfo(network_event) => {
                let (level, event_text) = match network_event {
                    NetworkEvent::Data(data_event) => match data_event {
                        DataEvent::Received { data, from } => (
                            EventLevel::Info,
                            format!("Received {} bytes from {}", data.len(), from.to_string()),
                        ),
                        DataEvent::Sent {
                            token,
                            to,
                            bytes_sent,
                        } => (
                            EventLevel::Info,
                            format!(
                                "Sent {} bytes to {} (token: {})",
                                bytes_sent,
                                to.to_string(),
                                safe_id_display(&token)
                            ),
                        ),
                        DataEvent::Sending { token, to, bytes } => (
                            EventLevel::Info,
                            format!(
                                "Sending {} bytes to {} (token: {})",
                                bytes,
                                to.to_string(),
                                safe_id_display(&token)
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
                match network_error {
                    NetworkErrorEvent::SocketError(socket_error) => match socket_error {
                        ConnectionFailed {
                            endpoint,
                            reason,
                            token,
                        } => {
                            self.add_network_event(
                                EventLevel::Error,
                                format!(
                                    "Connection failed: {:?} (endpoint: {}, token {})",
                                    reason,
                                    endpoint,
                                    safe_id_display(&token)
                                ),
                            );
                        }
                        SendFailed {
                            endpoint,
                            token,
                            reason,
                        } => {
                            self.add_network_event(
                                EventLevel::Error,
                                format!(
                                    "Send failed: {:?} (endpoint: {}, token {})",
                                    reason,
                                    endpoint,
                                    safe_id_display(&token)
                                ),
                            );
                        }
                        ReceiveFailed { endpoint, reason } => {
                            self.add_network_event(
                                EventLevel::Error,
                                format!("Receive failed: {:?} (endpoint: {})", reason, endpoint),
                            );
                        }
                        SocketError { endpoint, reason } => {
                            self.add_network_event(
                                EventLevel::Error,
                                format!("Socket error: {:?} (endpoint: {})", reason, endpoint),
                            );
                        }
                    },
                };
            }
            ChatAppEvent::Info(info) => {
                self.add_app_event(EventLevel::Info, format!("Internal: {}", info))
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
    pub ui: MainView,
    // TODO: those 2 must be retrieve from the model
    context_initialized: bool,
}

impl DTChatApp {
    pub fn new(chat_model: Arc<Mutex<ChatModel>>, event_handler: Arc<Mutex<EventHandler>>) -> Self {
        let local = chat_model.lock().unwrap().get_localpeer();
        let ui = MainView::new(local);

        Self {
            event_handler,
            chat_model,
            ui,
            context_initialized: false,
        }
    }
}

impl App for DTChatApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.context_initialized {
            if let Ok(mut handler) = self.event_handler.lock() {
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
