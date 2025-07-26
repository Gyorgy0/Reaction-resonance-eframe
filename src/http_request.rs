use egui::TextBuffer;
use log::debug;
use serde_json::json;

pub fn get_req() {
    /*let request = ehttp::Request::get("https://raw.githubusercontent.com/Gyorgy0/Reaction-resonance-release/master/materials/gas.json");
    ehttp::fetch(request, move |result: ehttp::Result<ehttp::Response>| {
        debug!("{:?}", result.unwrap().text().unwrap());
    });*/
    let request = ehttp::Request::get(
        "https://api.github.com/repos/Gyorgy0/Reaction-resonance-release/contents/materials",
    );
    ehttp::fetch(request, move |result: ehttp::Result<ehttp::Response>| {
        let binding = result.unwrap();
        let resp = binding.text().unwrap();
        let asd: serde_json::Value = serde_json::from_str(resp).unwrap();
        println!("{:?}", asd);
    });
}
