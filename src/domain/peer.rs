use std::fmt;

use dtchat_backend::Endpoint;
use egui::Color32;
use serde::Deserializer;
use serde::{
    de::{self, Visitor},
    Deserialize,
};

use crate::utils::color::Color32FromStr;

#[derive(Clone, Debug)]
pub struct EndpointWrapper(pub Endpoint);

impl<'de> Deserialize<'de> for EndpointWrapper {
    fn deserialize<D>(deserializer: D) -> Result<EndpointWrapper, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct EndpointVisitor;

        impl<'de> Visitor<'de> for EndpointVisitor {
            type Value = EndpointWrapper;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string like 'tcp 127.0.0.1:8000'")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Endpoint::from_str(v)
                    .map(EndpointWrapper)
                    .map_err(E::custom)
            }
        }

        deserializer.deserialize_str(EndpointVisitor)
    }
}

// Optional: Convert from wrapper to inner type
impl From<EndpointWrapper> for Endpoint {
    fn from(wrapper: EndpointWrapper) -> Self {
        wrapper.0
    }
}

// === Updated Peer Struct Using EndpointWrapper for Deserialization ===

#[derive(Clone, Debug, Deserialize)]
pub struct RawPeer {
    pub uuid: String,
    pub name: String,
    pub endpoints: Vec<EndpointWrapper>,
    pub color: String,
}

// === Final Peer Struct You Want ===

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Peer {
    pub uuid: String,
    pub name: String,
    pub endpoints: Vec<Endpoint>,
    pub color: Color32,
}

// === Helper Conversion ===
impl From<RawPeer> for Peer {
    fn from(raw: RawPeer) -> Self {
        Peer {
            uuid: raw.uuid,
            name: raw.name,
            color: Color32::from_str(&raw.color),
            endpoints: raw.endpoints.into_iter().map(|e| e.into()).collect(),
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

    pub fn peers(&self) -> &[Peer] {
        &self.peers
    }

    pub fn local_peer(&self) -> &Peer {
        &self.local_peer
    }
}
