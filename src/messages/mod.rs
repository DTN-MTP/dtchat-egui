use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use dtchat_backend::{
    dtchat::{ChatModel, Peer, Room},
    message::{sort_with_strategy, ChatMessage, SortStrategy},
    time::DTChatTime,
    EndpointProto,
};
use egui::{CentralPanel, TopBottomPanel, Ui};

use crate::{
    main_view::MirroredData,
    messages::{
        graph_view::MessageGraphView, list_view::MessageListView, prompt_view::MessagePromptView,
        settings_view::MessageSettingsView, side_view::SideSelectionView,
    },
    utils::text::PrettyStr,
};

pub mod graph_view;
pub mod list_view;
pub mod prompt_view;
pub mod settings_view;
pub mod side_view;

#[derive(PartialEq, Clone)]
pub enum MessageCountToDisplay {
    Nothing,
    All,
    Last(usize),
}

#[derive(PartialEq, Clone)]
struct Preferences {
    pub max_message_count: MessageCountToDisplay,
    pub sort_strategy: SortStrategy,
    pub protocol_filter: ProtoFilter,
}

impl Preferences {
    pub fn new() -> Self {
        Self {
            max_message_count: MessageCountToDisplay::All,
            sort_strategy: SortStrategy::Standard,
            protocol_filter: ProtoFilter::NoFilter,
        }
    }
}

pub struct PreferencesContext {
    last_uuid: Option<String>,
    current_context: Preferences,
    context_map: HashMap<String, Preferences>,
}

impl PreferencesContext {
    pub fn new() -> Self {
        Self {
            last_uuid: None,
            current_context: Preferences::new(),
            context_map: HashMap::new(),
        }
    }

    pub fn load_context(&mut self, uuid: &str) {
        if let Some(ref last_uuid) = self.last_uuid {
            if last_uuid != uuid {
                self.context_map
                    .insert(last_uuid.clone(), self.current_context.clone());
            }
        }

        self.current_context = self
            .context_map
            .entry(uuid.to_string())
            .or_insert_with(|| Preferences::new())
            .clone();

        self.last_uuid = Some(uuid.to_string());
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum MessageViewType {
    MessageGraph,
    MessageList,
}

impl MessageViewType {
    pub fn name(&self) -> &'static str {
        match self {
            MessageViewType::MessageGraph => "📈 Graph",
            MessageViewType::MessageList => "💬 List ",
        }
    }
}

#[derive(PartialEq)]
pub enum MessagingMode {
    All,
    Peer(Option<Peer>),
    Room(Option<Room>),
}

#[derive(Clone, PartialEq, Eq)]
pub enum ProtoFilter {
    NoFilter,
    Filter(EndpointProto),
}

impl PrettyStr for ProtoFilter {
    fn to_pretty_str(&self) -> String {
        match self {
            ProtoFilter::NoFilter => "All protocol".to_string(),
            ProtoFilter::Filter(endpoint_proto) => endpoint_proto.to_pretty_str(),
        }
    }
}

impl std::fmt::Display for ProtoFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProtoFilter::NoFilter => write!(f, "All"),
            ProtoFilter::Filter(proto) => write!(f, "{}", proto),
        }
    }
}

pub struct MessagesView {
    pub request_filter: bool,

    // defines the view/preferences
    pub current_mode: MessagingMode,
    pub pref_ctx: PreferencesContext,

    // views
    pub message_prompt_view: MessagePromptView,
    pub message_settings_view: MessageSettingsView,
    pub message_list_view: MessageListView,
    pub message_graph_view: MessageGraphView,
    pub room_selection_view: SideSelectionView,

    // view to display
    pub current_view: MessageViewType,

    // messages:
    pub messages_to_display: Vec<ChatMessage>,
}

impl MessagesView {
    pub fn new(model: Arc<Mutex<ChatModel>>) -> Self {
        Self {
            message_prompt_view: MessagePromptView::new(model),
            message_settings_view: MessageSettingsView::new(),
            current_view: MessageViewType::MessageGraph,
            request_filter: false,
            pref_ctx: PreferencesContext::new(),

            current_mode: MessagingMode::All,
            message_list_view: MessageListView::new(),
            message_graph_view: MessageGraphView::new(),
            room_selection_view: SideSelectionView::new(),
            messages_to_display: Vec::new(),
        }
    }

    pub fn manage_message(&mut self, data: &MirroredData) {
        self.messages_to_display = data
            .messages
            .iter()
            .filter(|msg| {
                let mut retain = true;

                match &self.pref_ctx.current_context.protocol_filter {
                    ProtoFilter::NoFilter => (),
                    ProtoFilter::Filter(endpoint_proto) => {
                        if msg.source_endpoint.proto != *endpoint_proto {
                            retain = false;
                        }
                    }
                }

                match &self.current_mode {
                    MessagingMode::Peer(peer_opt) => {
                        if let Some(peer) = peer_opt {
                            if !(msg.room_uuid == peer.uuid
                                && msg.sender_uuid == data.local_peer.uuid
                                || msg.sender_uuid == peer.uuid
                                    && msg.room_uuid == data.local_peer.uuid)
                            {
                                retain = false;
                            }
                        }
                    }
                    MessagingMode::Room(room_opt) => {
                        if let Some(room) = room_opt {
                            if msg.room_uuid != room.uuid {
                                retain = false;
                            }
                        }
                    }
                    MessagingMode::All => (),
                }
                retain
            })
            .cloned()
            .collect();

        sort_with_strategy(
            &mut self.messages_to_display,
            self.pref_ctx.current_context.sort_strategy.clone(),
        );

        // Should be safe as long as those flags are not supposed to be raised asynchronously
    }

    fn message_to_display_bounds(&mut self) -> usize {
        let msgs = self.messages_to_display.len();
        match self.pref_ctx.current_context.max_message_count {
            MessageCountToDisplay::Nothing => msgs,
            MessageCountToDisplay::All => 0,
            MessageCountToDisplay::Last(count) => msgs.saturating_sub(count),
        }
    }

    pub fn show(
        &mut self,
        ctx: &egui::Context,
        data: &MirroredData,
        current_time: &DTChatTime,
        ui: &mut Ui,
    ) {
        let peer_context_changed = self.request_filter;
        if self.request_filter {
            self.manage_message(data);
            self.request_filter = false;
        }

        TopBottomPanel::bottom("message_forge_panel").show_inside(ui, |ui| {
            self.message_prompt_view.show(
                ctx,
                ui,
                data.pbat_support_by_model,
                &self.current_mode,
                peer_context_changed,
            );
        });

        let start_idx: usize = self.message_to_display_bounds();

        egui::SidePanel::right("right_panel")
            .resizable(true)
            .default_width(115.0)
            .width_range(80.0..=200.0)
            .show_inside(ui, |ui| {
                self.room_selection_view.show(
                    ui,
                    &data.other_peers,
                    &data.rooms,
                    &mut self.pref_ctx,
                    &mut self.current_mode,
                    &mut self.request_filter,
                )
            });

        // setting + message view (graph/list/etc.)
        CentralPanel::default().show_inside(ui, |ui| {
            TopBottomPanel::top("message_settings_bar").show_inside(ui, |ui| {
                self.message_settings_view.show(
                    ui,
                    &mut self.current_view,
                    &mut self.pref_ctx.current_context.sort_strategy,
                    &mut self.pref_ctx.current_context.protocol_filter,
                    &mut self.pref_ctx.current_context.max_message_count,
                    self.messages_to_display.len(),
                    &data.local_peer,
                    &data.other_peers,
                    &mut self.request_filter,
                );
            });
            match self.current_view {
                MessageViewType::MessageGraph => {
                    self.message_graph_view.show(
                        ui,
                        &self.messages_to_display[start_idx..],
                        &data.local_peer,
                        &data.other_peers,
                        current_time,
                    );
                }
                MessageViewType::MessageList => {
                    self.message_list_view.show(
                        ui,
                        &self.messages_to_display[start_idx..],
                        current_time,
                        &data.local_peer,
                        &data.other_peers,
                    );
                }
            }
        });
    }
}
