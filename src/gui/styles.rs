use crate::models::weather::FlightRules;
use eframe::egui::{Color32, Visuals};

pub fn get_flight_rules_color(rules: &FlightRules, visuals: &Visuals) -> Color32 {
    let is_dark_mode = visuals.dark_mode;

    match rules {
        FlightRules::VFR => if is_dark_mode {
            Color32::GREEN
        } else {
            // Darker Green for light mode
            Color32::from_rgb(0, 128, 0)
        },
        FlightRules::MVFR => if is_dark_mode {
            // Cornflower Blue
            Color32::from_rgb(100, 149, 237)
        } else {
            // Dark Blue for light mode
            Color32::from_rgb(0, 0, 139)
        },
        FlightRules::IFR => if is_dark_mode {
            Color32::RED
        } else {
            // Dark Red for light mode
            Color32::from_rgb(139, 0, 0)
        },
        FlightRules::LIFR => if is_dark_mode {
            // Magenta
            Color32::from_rgb(255, 0, 255)
        } else {
            // Dark Magenta / Purple for light mode
            Color32::from_rgb(128, 0, 128)
        },
        FlightRules::Unknown => visuals.text_color(),
    }
}
