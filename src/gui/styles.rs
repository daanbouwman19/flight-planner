use crate::models::weather::FlightRules;
use eframe::egui::{Color32, Visuals};

pub fn get_flight_rules_color(rules: &FlightRules, visuals: &Visuals) -> Color32 {
    let (dark, light) = match rules {
        FlightRules::VFR => (
            Color32::GREEN,
            Color32::from_rgb(0, 128, 0), // Darker Green
        ),
        FlightRules::MVFR => (
            Color32::from_rgb(100, 149, 237), // Cornflower Blue
            Color32::from_rgb(0, 0, 139),     // Dark Blue
        ),
        FlightRules::IFR => (
            Color32::RED,
            Color32::from_rgb(139, 0, 0), // Dark Red
        ),
        FlightRules::LIFR => (
            Color32::from_rgb(255, 0, 255), // Magenta
            Color32::from_rgb(128, 0, 128), // Dark Magenta
        ),
        FlightRules::Unknown => return visuals.text_color(),
    };

    if visuals.dark_mode { dark } else { light }
}
