use std::error::Error;
use std::sync::{Arc, Mutex};

mod app;
mod ui;
#[macro_use]
mod utils;

use app::DTChatApp;
use dtchat_backend::dtchat::ChatModel;

use dtchat_backend::Engine;
use eframe::{App, NativeOptions};

fn main() -> Result<(), Box<dyn Error>> {
    let event_handler = Arc::new(Mutex::new(app::EventHandler::new(100)));

    let model = ChatModel::new();
    let model_arc = Arc::new(Mutex::new(model));

    let app = DTChatApp::new(model_arc.clone(), event_handler.clone());

    model_arc
        .lock()
        .unwrap()
        .add_observer(event_handler.clone());

    let mut network_engine = Engine::new();
    network_engine.add_observer(model_arc.clone());

    model_arc.lock().unwrap().start(network_engine);

    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([850.0, 600.0]), // width, height
        ..Default::default()
    };
    eframe::run_native(
        "DTChat",
        options,
        Box::new(
            move |cc| -> Result<Box<dyn App>, Box<dyn Error + Send + Sync>> {
                cc.egui_ctx.style_mut(|style| {
                    style.interaction.tooltip_delay = 0.33;
                    // for (_text_style, font_id) in style.text_styles.iter_mut() {
                    //     font_id.size *= 1.2;
                    // }
                });

                Ok(Box::new(app))
            },
        ),
    )?;
    Ok(())
}
