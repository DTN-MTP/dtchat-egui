use serde::Deserialize;
use socket_engine::endpoint::Endpoint;

// Constantes pour les protocoles supportés
pub const PROTOCOL_TCP: &str = "TCP";
pub const PROTOCOL_UDP: &str = "UDP";
pub const PROTOCOL_BP: &str = "BP";

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct EndpointConfig {
    #[serde(rename = "type")]
    pub endpoint_type: String,
    pub address: String,
}

impl EndpointConfig {
    pub fn to_endpoint(&self) -> Option<Endpoint> {
        match self.endpoint_type.as_str() {
            PROTOCOL_TCP => Some(Endpoint::Tcp(self.address.clone())),
            PROTOCOL_UDP => Some(Endpoint::Udp(self.address.clone())),
            PROTOCOL_BP => Some(Endpoint::Bp(self.address.clone())),
            _ => None,
        }
    }

    pub fn protocol_to_endpoint(protocol: &str) -> Option<Endpoint> {
        match protocol {
            PROTOCOL_TCP => Some(Endpoint::Tcp(String::new())),
            PROTOCOL_UDP => Some(Endpoint::Udp(String::new())),
            PROTOCOL_BP => Some(Endpoint::Bp(String::new())),
            _ => None,
        }
    }

    pub fn endpoint_to_protocol(endpoint: &Endpoint) -> &'static str {
        match endpoint {
            Endpoint::Tcp(_) => PROTOCOL_TCP,
            Endpoint::Udp(_) => PROTOCOL_UDP,
            Endpoint::Bp(_) => PROTOCOL_BP,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct Peer {
    pub uuid: String,
    pub name: String,
    pub endpoints: Vec<EndpointConfig>,
    pub color: u32,
}

impl Peer {
    pub fn get_endpoint_by_protocol(&self, protocol: &str) -> Option<Endpoint> {
        self.endpoints
            .iter()
            .find(|ep| ep.endpoint_type == protocol)
            .and_then(|ep| ep.to_endpoint())
    }

    pub fn get_available_protocols(&self) -> Vec<String> {
        self.endpoints
            .iter()
            .map(|ep| ep.endpoint_type.clone())
            .collect()
    }
}

impl Default for Peer {
    fn default() -> Self {
        Self {
            uuid: "unknown".to_string(),
            name: "Unknown".to_string(),
            endpoints: Vec::new(),
            color: 0,
        }
    }
}

pub struct PeerManager {
    local_peer: Peer,
    peers: Vec<Peer>,
}

impl PeerManager {
    pub fn new(local_peer: Peer, peers: Vec<Peer>) -> Self {
        Self { local_peer, peers }
    }

    pub fn find_endpoint_for_peer_with_protocol(
        &self,
        peer_uuid: &str,
        protocol: &str,
    ) -> Option<Endpoint> {
        self.peers
            .iter()
            .find(|peer| peer.uuid == peer_uuid)
            .and_then(|peer| peer.get_endpoint_by_protocol(protocol))
    }

    pub fn peers(&self) -> &[Peer] {
        &self.peers
    }

    pub fn local_peer(&self) -> &Peer {
        &self.local_peer
    }
}
