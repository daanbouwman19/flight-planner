use crate::gui::components::searchable_dropdown::{
    DropdownConfig, DropdownSelection, SearchableDropdown, SearchableDropdownCallbacks,
};
use crate::gui::icons;
use crate::models::{Aircraft, Airport};
use egui::Ui;
use rand::prelude::*;
use std::sync::Arc;

/// The initial number of items to display in a searchable dropdown list.
pub const INITIAL_DISPLAY_COUNT: usize = 50;

pub enum DropdownAction<T> {
    Toggle,
    Select(Arc<T>),
    Unselect,
    None,
}

pub struct DropdownParams<'a, T> {
    pub ui: &'a mut Ui,
    pub id: &'a str,
    pub label: &'a str,
    pub placeholder: &'a str,
    pub selected_item: Option<&'a Arc<T>>,
    pub all_items: &'a [Arc<T>],
    pub search_text: &'a mut String,
    pub display_count: &'a mut usize,
    pub is_open: bool,
    pub autofocus: &'a mut bool,
}

#[cfg(not(tarpaulin_include))]
pub fn render_airport_dropdown(params: DropdownParams<Airport>) -> DropdownAction<Airport> {
    render_generic_dropdown(
        params,
        |a| format!("{} ({})", a.Name, a.ICAO),
        |a| {
            Some(format!(
                "Elevation: {} ft\nLat: {:.4}, Lon: {:.4}",
                a.Elevation, a.Latitude, a.Longtitude
            ))
        },
        |a, search| {
            crate::util::contains_case_insensitive(&a.Name, search)
                || crate::util::contains_case_insensitive(&a.ICAO, search)
        },
    )
}

#[cfg(not(tarpaulin_include))]
pub fn render_aircraft_dropdown(params: DropdownParams<Aircraft>) -> DropdownAction<Aircraft> {
    render_generic_dropdown(
        params,
        |a| format!("{} {}", a.manufacturer, a.variant),
        |a| {
            let mut info = format!(
                "ICAO: {}\nRange: {} nm\nCruise: {} kts\nCategory: {}",
                a.icao_code, a.aircraft_range, a.cruise_speed, a.category
            );

            if let Some(dist) = a.takeoff_distance {
                info.push_str(&format!("\nTakeoff Dist: {} m", dist));
            }

            Some(info)
        },
        |a, search| {
            crate::util::contains_case_insensitive(&a.manufacturer, search)
                || crate::util::contains_case_insensitive(&a.variant, search)
        },
    )
}

#[cfg(not(tarpaulin_include))]
fn render_generic_dropdown<T, F1, F2, F3>(
    params: DropdownParams<T>,
    display_formatter: F1,
    tooltip_formatter: F3,
    search_matcher: F2,
) -> DropdownAction<T>
where
    T: PartialEq + Clone + 'static,
    F1: Fn(&T) -> String + 'static + Clone,
    F2: Fn(&T, &str) -> bool + 'static + Clone,
    F3: Fn(&T) -> Option<String> + 'static + Clone,
{
    let mut action = DropdownAction::None;

    params.ui.label(params.label);
    let display_text = params
        .selected_item
        .map_or(params.placeholder.to_string(), |item| {
            display_formatter(item)
        });

    params.ui.horizontal(|ui| {
        if render_dropdown_button(
            ui,
            &display_text,
            &format!(
                "Click to select {}",
                params
                    .label
                    .trim_end_matches(':')
                    .trim_end_matches('*')
                    .trim()
            ),
            params.is_open,
        ) {
            action = DropdownAction::Toggle;
        }

        if params.selected_item.is_some()
            && ui
                .add_sized(
                    [20.0, 20.0],
                    egui::Button::new(icons::ICON_CLOSE).small().frame(false),
                )
                .on_hover_text("Clear selection")
                .clicked()
        {
            action = DropdownAction::Unselect;
        }
    });

    if params.is_open {
        let config = DropdownConfig {
            id: params.id,
            search_hint: "Search...",
            initial_chunk_size: 50,
            auto_focus: *params.autofocus,
            ..Default::default()
        };

        if *params.autofocus {
            *params.autofocus = false;
        }

        // Wrapper to adapt Fn(&T) to Fn(&Arc<T>)
        let fmt_clone = display_formatter.clone();
        let display_wrapper = move |item: &Arc<T>| fmt_clone(item);

        let tooltip_clone = tooltip_formatter.clone();
        let tooltip_wrapper = move |item: &Arc<T>| tooltip_clone(item);

        let matcher_clone = search_matcher.clone();
        let search_wrapper = move |item: &Arc<T>, search: &str| matcher_clone(item, search);

        // Current selection matcher
        let current_selected = params.selected_item.cloned();
        let selection_matcher =
            move |item: &Arc<T>| current_selected.as_ref().is_some_and(|s| s == item);

        let callbacks = SearchableDropdownCallbacks {
            current_selection_matcher: Box::new(selection_matcher),
            display_formatter: Box::new(display_wrapper),
            tooltip_formatter: Box::new(tooltip_wrapper),
            search_matcher: Box::new(search_wrapper),
            random_selector: Box::new(|items| items.choose(&mut rand::rng()).cloned()),
        };

        let mut dropdown = SearchableDropdown::new(
            params.all_items,
            params.search_text,
            config,
            params.display_count,
            callbacks,
        );

        match dropdown.render(params.ui) {
            DropdownSelection::Item(item) | DropdownSelection::Random(item) => {
                action = DropdownAction::Select(item);
            }
            DropdownSelection::Unspecified => {
                action = DropdownAction::Unselect;
            }
            DropdownSelection::None => {}
        }
    }

    action
}

#[cfg(not(tarpaulin_include))]
fn render_dropdown_button(ui: &mut Ui, text: &str, hover_text: &str, open: bool) -> bool {
    // Determine the icon to show based on the open state
    let icon = if open {
        icons::ICON_CARET_UP
    } else {
        icons::ICON_CARET_DOWN
    };

    // Render the button with text and icon
    ui.button(format!("{} {}", text, icon))
        .on_hover_text(hover_text)
        .clicked()
}
