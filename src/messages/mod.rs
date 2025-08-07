use dtchat_backend::{
    dtchat::{Peer, Room},
    message::{filter_by_network_endpoint, sort_with_strategy, ChatMessage, SortStrategy},
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

pub struct MessageView {
    pub request_sort_strategy: bool,
    pub request_protocol_filter: bool,

    pub max_message_count: usize,
    pub sort_strategy: SortStrategy,
    pub protocol_filter: ProtoFilter,

    // defines the view
    pub current_room: Option<Room>,
    pub current_peer: Option<Peer>,
    // views
    pub message_prompt_view: MessagePromptView,
    pub message_settings_view: MessageSettingsView,
    pub message_list_view: MessageListView,
    pub message_graph_view: MessageGraphView,
    pub room_selection_view: SideSelectionView,

    // view to display
    pub current_view: MessageViewType,

    // messages:
    messages_to_display: Vec<ChatMessage>,
}

impl MessageView {
    pub fn new() -> Self {
        Self {
            message_prompt_view: MessagePromptView::new(),
            message_settings_view: MessageSettingsView::new(),
            current_view: MessageViewType::MessageGraph,
            request_sort_strategy: true,
            max_message_count: 0,
            sort_strategy: SortStrategy::Standard,
            request_protocol_filter: true,
            protocol_filter: ProtoFilter::NoFilter,
            current_room: None,
            current_peer: None,
            message_list_view: MessageListView::new(),
            message_graph_view: MessageGraphView::new(),
            room_selection_view: SideSelectionView::new(),
            messages_to_display: Vec::new(),
        }
    }

    fn manage_message(&mut self, data: &MirroredData) -> usize {
        if self.request_protocol_filter {
            self.messages_to_display = match &self.protocol_filter {
                ProtoFilter::NoFilter => data.messages.clone(),
                ProtoFilter::Filter(by_proto) => {
                    filter_by_network_endpoint(&data.messages, by_proto.clone())
                }
            };
            self.request_protocol_filter = false;
        }
        if self.request_sort_strategy {
            sort_with_strategy(&mut self.messages_to_display, self.sort_strategy.clone());
            self.request_sort_strategy = false;
        }

        self.messages_to_display
            .len()
            .saturating_sub(self.max_message_count)
    }

    pub fn show(&mut self, data: &MirroredData, current_time: &DTChatTime, ui: &mut Ui) {
        let start_idx: usize = self.manage_message(data);

        egui::SidePanel::right("right_panel")
            .resizable(true)
            .default_width(115.0)
            .width_range(80.0..=200.0)
            .show_inside(ui, |ui| {
                self.room_selection_view.show(
                    ui,
                    &data.other_peers,
                    &data.rooms,
                    &mut self.current_room,
                    &mut self.current_peer,
                )
            });

        // setting + message view (graph/list/etc.)
        CentralPanel::default().show_inside(ui, |ui| {
            TopBottomPanel::top("message_settings_bar").show_inside(ui, |ui| {
                self.message_settings_view.show(
                    ui,
                    &mut self.current_view,
                    &mut self.sort_strategy,
                    &mut self.request_sort_strategy,
                    &mut self.protocol_filter,
                    &mut self.request_protocol_filter,
                    &mut self.max_message_count,
                    data.messages.len(),
                    &data.local_peer,
                    &data.other_peers,
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
