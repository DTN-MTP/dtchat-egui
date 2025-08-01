use dtchat_backend::{Endpoint, EndpointProto};
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
