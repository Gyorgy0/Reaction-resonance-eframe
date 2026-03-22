use std::sync::{Arc, Mutex};

pub fn get_materials(response_text: Arc<Mutex<Vec<String>>>) {
    let request = ehttp::Request::get(
        "https://raw.githubusercontent.com/Gyorgy0/Reaction-resonance-release/master/materials/solid.json",
    );
    let response_text_clone = Arc::clone(&response_text);
    ehttp::fetch(request, move |result: ehttp::Result<ehttp::Response>| {
        let response = result.unwrap();
        response_text_clone
            .lock()
            .unwrap()
            .push(response.text().unwrap().to_owned());
    });
    let request = ehttp::Request::get(
        "https://raw.githubusercontent.com/Gyorgy0/Reaction-resonance-release/master/materials/powder.json",
    );
    let response_text_clone = Arc::clone(&response_text);
    ehttp::fetch(request, move |result: ehttp::Result<ehttp::Response>| {
        let response = result.unwrap();
        response_text_clone
            .lock()
            .unwrap()
            .push(response.text().unwrap().to_owned());
    });
    let request = ehttp::Request::get(
        "https://raw.githubusercontent.com/Gyorgy0/Reaction-resonance-release/master/materials/plasma.json",
    );
    let response_text_clone = Arc::clone(&response_text);
    ehttp::fetch(request, move |result: ehttp::Result<ehttp::Response>| {
        let response = result.unwrap();
        response_text_clone
            .lock()
            .unwrap()
            .push(response.text().unwrap().to_owned());
    });
    let request = ehttp::Request::get(
        "https://raw.githubusercontent.com/Gyorgy0/Reaction-resonance-release/master/materials/liquid.json",
    );
    let response_text_clone = Arc::clone(&response_text);
    ehttp::fetch(request, move |result: ehttp::Result<ehttp::Response>| {
        let response = result.unwrap();
        response_text_clone
            .lock()
            .unwrap()
            .push(response.text().unwrap().to_owned());
    });
    let request = ehttp::Request::get(
        "https://raw.githubusercontent.com/Gyorgy0/Reaction-resonance-release/master/materials/gas.json",
    );
    let response_text_clone = Arc::clone(&response_text);
    ehttp::fetch(request, move |result: ehttp::Result<ehttp::Response>| {
        let response = result.unwrap();
        response_text_clone
            .lock()
            .unwrap()
            .push(response.text().unwrap().to_owned());
    });
    let request = ehttp::Request::get(
        "https://raw.githubusercontent.com/Gyorgy0/Reaction-resonance-release/master/materials/life.json",
    );
    let response_text_clone = Arc::clone(&response_text);
    ehttp::fetch(request, move |result: ehttp::Result<ehttp::Response>| {
        let response = result.unwrap();
        response_text_clone
            .lock()
            .unwrap()
            .push(response.text().unwrap().to_owned());
    });
}

pub fn get_melting_transitions(response_text: Arc<Mutex<Vec<String>>>) {
    let request = ehttp::Request::get(
        "https://raw.githubusercontent.com/Gyorgy0/Reaction-resonance-release/master/materials/solid.json",
    );
    let response_text_clone = Arc::clone(&response_text);
    ehttp::fetch(request, move |result: ehttp::Result<ehttp::Response>| {
        let response = result.unwrap();
        response_text_clone
            .lock()
            .unwrap()
            .push(response.text().unwrap().to_owned());
    });
    let request = ehttp::Request::get(
        "https://raw.githubusercontent.com/Gyorgy0/Reaction-resonance-release/master/materials/powder.json",
    );
}
pub fn get_boiling_transitions(response_text: Arc<Mutex<Vec<String>>>) {
    let request = ehttp::Request::get(
        "https://raw.githubusercontent.com/Gyorgy0/Reaction-resonance-release/master/materials/solid.json",
    );
    let response_text_clone = Arc::clone(&response_text);
    ehttp::fetch(request, move |result: ehttp::Result<ehttp::Response>| {
        let response = result.unwrap();
        response_text_clone
            .lock()
            .unwrap()
            .push(response.text().unwrap().to_owned());
    });
    let request = ehttp::Request::get(
        "https://raw.githubusercontent.com/Gyorgy0/Reaction-resonance-release/master/materials/powder.json",
    );
}
pub fn get_sublimation_transitions(response_text: Arc<Mutex<Vec<String>>>) {
    let request = ehttp::Request::get(
        "https://raw.githubusercontent.com/Gyorgy0/Reaction-resonance-release/master/materials/solid.json",
    );
    let response_text_clone = Arc::clone(&response_text);
    ehttp::fetch(request, move |result: ehttp::Result<ehttp::Response>| {
        let response = result.unwrap();
        response_text_clone
            .lock()
            .unwrap()
            .push(response.text().unwrap().to_owned());
    });
    let request = ehttp::Request::get(
        "https://raw.githubusercontent.com/Gyorgy0/Reaction-resonance-release/master/materials/powder.json",
    );
}
pub fn get_ionization_transitions(response_text: Arc<Mutex<Vec<String>>>) {
    let request = ehttp::Request::get(
        "https://raw.githubusercontent.com/Gyorgy0/Reaction-resonance-release/master/materials/solid.json",
    );
    let response_text_clone = Arc::clone(&response_text);
    ehttp::fetch(request, move |result: ehttp::Result<ehttp::Response>| {
        let response = result.unwrap();
        response_text_clone
            .lock()
            .unwrap()
            .push(response.text().unwrap().to_owned());
    });
    let request = ehttp::Request::get(
        "https://raw.githubusercontent.com/Gyorgy0/Reaction-resonance-release/master/materials/powder.json",
    );
}
