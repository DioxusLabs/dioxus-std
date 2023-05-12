use dioxus::prelude::*;
use dioxus_std::{
    hooks::{self, use_geolocation},
    library::geolocation::PowerMode,
};

fn main() {
    dioxus_desktop::launch(app);
}

fn app(cx: Scope) -> Element {
    let geolocator = hooks::init_geolocator(cx, PowerMode::High, None, None).unwrap();
    let initial_coords = use_state(cx, || geolocator.get_coordinates().unwrap());
    let latest_coords = use_geolocation(cx).unwrap();

    // Google maps embed api key
    let key = std::env::var("DIOXUS_GEOLOCATION_MAP_KEY").unwrap();

    cx.render(rsx! (
        div {
            style: "text-align: center;",
            h1 { "🗺️ Dioxus Geolocation Example 🛰️" }
            h3 { "Your initial location is:"}
            p { format!("Latitude: {} | Longitude: {} | Altitude: {}", initial_coords.latitude, initial_coords.longitude, initial_coords.altitude) }
            h3 { "Your latest location is:" }
            p { format!("Latitude: {} | Longitude: {} | Altitude: {}", latest_coords.latitude, latest_coords.longitude, latest_coords.altitude) }

            iframe {
                width: "400",
                height: "400",
                style: "border: 1px solid black",
                src: "https://www.google.com/maps/embed/v1/view?key={key}&center={latest_coords.latitude},{latest_coords.longitude}&zoom=16",
            }
        }
    ))
}
