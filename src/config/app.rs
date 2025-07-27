use crate::app::{EventHandler, EventLevel};
use crate::domain::peer::Peer;
use crate::domain::peer::RawPeer;
use crate::log_with_location;
use crate::utils::load_yaml_from_file;
use serde::Deserialize;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct AppConfig {
    pub peer_list: Vec<Peer>,
    pub local_peer: Peer,
    // pub a_sabr: String,
}

#[derive(Debug, Deserialize)]
pub struct ConfigFile {
    pub peer_list: Vec<RawPeer>,
    #[serde(default)]
    #[allow(dead_code)]
    pub a_sabr: Option<String>,
}

impl AppConfig {
    pub fn from_file(file_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let config_file: ConfigFile = load_yaml_from_file(file_path)?;

        let peer_id =
            std::env::var("PEER_UUID").map_err(|_| "Variable d'environnement PEER non définie")?;

        let local_peer = config_file
            .peer_list
            .iter()
            .find(|peer| peer.uuid == peer_id)
            .ok_or_else(|| format!("Peer avec l'ID '{}' non trouvé dans la liste", peer_id))?
            .clone();

        Ok(AppConfig {
            peer_list: config_file
                .peer_list
                .iter()
                .map(|rp| Peer::from(rp.clone()))
                .collect(),
            local_peer: Peer::from(local_peer),
        })
    }

    pub fn from_default(event_handler: Option<Arc<Mutex<EventHandler>>>) -> Self {
        let config_path = "db/default.yaml";

        if let Some(eh) = &event_handler {
            if let Ok(mut handler) = eh.lock() {
                handler.add_app_event(
                    EventLevel::Info,
                    format!("Using default configuration: {}", config_path),
                );
            }
        } else {
            log_with_location!("Using default configuration: {}", config_path);
        }

        Self::from_file(config_path).unwrap_or_else(|e| {
            panic!("[APP-CONFIG]: Failed to load configuration from '{config_path}': {e}");
        })
    }
}

pub fn initialize_app_config(event_handler: Option<Arc<Mutex<EventHandler>>>) -> AppConfig {
    if let Some(eh) = &event_handler {
        if let Ok(mut handler) = eh.lock() {
            handler.add_app_event(
                EventLevel::Info,
                "Initializing application configuration".to_string(),
            );
        }
    }
    AppConfig::from_default(event_handler)
}
