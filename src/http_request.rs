use log::debug;

pub fn get_req() {
    let request = ehttp::Request::get("https://raw.githubusercontent.com/Gyorgy0/Reaction-resonance-release/master/materials/gas.json");
    ehttp::fetch(request, move |result: ehttp::Result<ehttp::Response>| {
        debug!("{:?}", result.unwrap().text().unwrap());
    });
}
