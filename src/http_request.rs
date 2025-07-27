use eframe::glow::FRACTIONAL_EVEN;
use egui::{response, text, TextBuffer};
use ehttp::Response;
use futures::TryFutureExt;
use log::debug;
use serde_json::json;
use std::sync::{mpsc::channel, Arc, Mutex};

pub fn get_req(response_text: Arc<Mutex<String>>) {
    let request = ehttp::Request::get("https://raw.githubusercontent.com/Gyorgy0/Reaction-resonance-release/master/materials/liquid.json");
    let response_text_clone = Arc::clone(&response_text);
    ehttp::fetch(request, move |result: ehttp::Result<ehttp::Response>| {
        let mut response = result.unwrap();
        *response_text_clone.lock().unwrap() = response.text().unwrap().to_owned();
    });
}
