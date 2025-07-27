use std::error::Error;
use std::sync::{Arc, Mutex};

mod app;
mod config;
mod domain;
mod ui;
#[macro_use]
mod utils;

use app::DTChatApp;
use config::initialize_app_config;
use dtchat_backend::db::simple_vec::SimpleVecDB;
use dtchat_backend::dtchat::{ChatModel, Peer as BackendPeer};
use eframe::{App, NativeOptions};
use socket_engine::endpoint::Endpoint;
use socket_engine::engine::Engine;

use crate::domain::peer::Peer;

fn convert_peer_to_backend(domain_peer: &Peer) -> BackendPeer {
    let endpoints: Vec<Endpoint> = domain_peer.endpoints.clone();
    BackendPeer {
        uuid: domain_peer.uuid.clone(),
        name: domain_peer.name.clone(),
        endpoints,
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let event_handler = Arc::new(Mutex::new(app::EventHandler::new(10)));

    let app_config = initialize_app_config(Some(event_handler.clone()));
    let local_peer = app_config.local_peer;
    let shared_peers = app_config.peer_list;

    let backend_local_peer = convert_peer_to_backend(&local_peer);
    let backend_peers: Vec<BackendPeer> =
        shared_peers.iter().map(convert_peer_to_backend).collect();

    let db = Box::new(SimpleVecDB::default());

    let model = ChatModel::new(backend_local_peer, backend_peers.clone(), db);
    let model_arc = Arc::new(Mutex::new(model));

    let app = DTChatApp::new(
        model_arc.clone(),
        local_peer,
        shared_peers,
        event_handler.clone(),
    );

    model_arc
        .lock()
        .unwrap()
        .add_observer(event_handler.clone());

    let mut network_engine = Engine::new();
    network_engine.add_observer(model_arc.clone());

    model_arc.lock().unwrap().start(network_engine);

    let options = NativeOptions::default();
    eframe::run_native(
        "DTChat",
        options,
        Box::new(
            move |_| -> Result<Box<dyn App>, Box<dyn Error + Send + Sync>> { Ok(Box::new(app)) },
        ),
    )?;
    Ok(())
}
