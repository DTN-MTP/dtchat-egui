use dtchat_backend::{message::MessageStatus, Endpoint, EndpointProto};
use egui::Color32;
pub trait PrettyStr {
    fn to_pretty_str(&self) -> String;
}

impl PrettyStr for EndpointProto {
    fn to_pretty_str(&self) -> String {
        match self {
            EndpointProto::Udp => "↗ udp".to_string(),
            EndpointProto::Tcp => "🔁 tcp".to_string(),
            EndpointProto::Bp => "📡 bp".to_string(),
        }
    }
}

impl PrettyStr for Endpoint {
    fn to_pretty_str(&self) -> String {
        format!(
            "{} ({})",
            self.proto.to_pretty_str(),
            self.endpoint.as_str()
        )
    }
}

pub trait StatusDisplayHelper {
    // fn get_text(&self, participant_name: &str) -> String;
    fn get_color(&self) -> Color32;
    fn get_icon(&self) -> String;
    fn get_icon_text(&self, participant_name: &str) -> String;
}

impl StatusDisplayHelper for MessageStatus {
    // fn get_text(&self, participant_name: &str) -> String {
    //     match self {
    //         MessageStatus::Failed => format!("{} (error)", participant_name),
    //         MessageStatus::ReceivedByPeer => format!("{} (acked)", participant_name),
    //         MessageStatus::Sent => format!("{} (sent)", participant_name),
    //         MessageStatus::Sending => format!("{} (sending)", participant_name),
    //         MessageStatus::Received => format!("{} (received)", participant_name),
    //     }
    // }

    fn get_color(&self) -> Color32 {
        match self {
            MessageStatus::Failed => Color32::RED,
            MessageStatus::ReceivedByPeer => Color32::GREEN,
            MessageStatus::Sent => Color32::LIGHT_GRAY,
            MessageStatus::Sending => Color32::YELLOW,
            MessageStatus::Received => Color32::CYAN,
        }
    }

    fn get_icon(&self) -> String {
        match self {
            MessageStatus::Failed => "[\u{2716}]".to_string(),
            MessageStatus::ReceivedByPeer => "[\u{2714}]".to_string(),
            MessageStatus::Sent => "[\u{1F680}]".to_string(),
            MessageStatus::Sending => "[\u{1F4E4}]".to_string(),
            MessageStatus::Received => "[\u{1F4E5}]".to_string(),
        }
    }

    fn get_icon_text(&self, participant_name: &str) -> String {
        match self {
            MessageStatus::Failed => format!("{} \u{2716} (error)", participant_name),
            MessageStatus::ReceivedByPeer => format!("{} \u{2714} (acked)", participant_name),
            MessageStatus::Sent => format!("{} \u{1F680} (sent)", participant_name),
            MessageStatus::Sending => format!("{} \u{1F4E4} (sending)", participant_name),
            MessageStatus::Received => format!("{} \u{1F4E5} (received)", participant_name),
        }
    }
}
