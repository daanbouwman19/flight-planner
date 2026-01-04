use crate::gui::components::searchable_dropdown::{
    DropdownConfig, DropdownSelection, SearchableDropdown,
};
use crate::gui::events::Event;
use crate::models::{Aircraft, Airport};
use egui::{Stroke, Ui, vec2};
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

pub fn render_airport_dropdown(
    ui: &mut Ui,
    label: &str,
    placeholder: &str,
    selected_item: Option<&Arc<Airport>>,
    all_items: &[Arc<Airport>],
    search_text: &mut String,
    display_count: &mut usize,
    is_open: bool,
    autofocus: &mut bool,
    toggle_event: Event,
) -> DropdownAction<Airport> {
    render_generic_dropdown(
        ui,
        label,
        placeholder,
        selected_item,
        all_items,
        search_text,
        display_count,
        is_open,
        autofocus,
        toggle_event,
        "airport_dropdown",
        |a| format!("{} ({})", a.Name, a.ICAO),
        |a, search| {
            crate::util::contains_case_insensitive(&a.Name, search)
                || crate::util::contains_case_insensitive(&a.ICAO, search)
        },
    )
}

pub fn render_aircraft_dropdown(
    ui: &mut Ui,
    label: &str,
    placeholder: &str,
    selected_item: Option<&Arc<Aircraft>>,
    all_items: &[Arc<Aircraft>],
    search_text: &mut String,
    display_count: &mut usize,
    is_open: bool,
    autofocus: &mut bool,
    toggle_event: Event,
) -> DropdownAction<Aircraft> {
    render_generic_dropdown(
        ui,
        label,
        placeholder,
        selected_item,
        all_items,
        search_text,
        display_count,
        is_open,
        autofocus,
        toggle_event,
        "aircraft_dropdown",
        |a| format!("{} {}", a.manufacturer, a.variant),
        |a, search| {
            crate::util::contains_case_insensitive(&a.manufacturer, search)
                || crate::util::contains_case_insensitive(&a.variant, search)
        },
    )
}

#[allow(clippy::too_many_arguments)]
fn render_generic_dropdown<T, F1, F2>(
    ui: &mut Ui,
    label: &str,
    placeholder: &str,
    selected_item: Option<&Arc<T>>,
    all_items: &[Arc<T>],
    search_text: &mut String,
    display_count: &mut usize,
    is_open: bool,
    autofocus: &mut bool,
    _toggle_event: Event,
    base_id: &str,
    display_formatter: F1,
    search_matcher: F2,
) -> DropdownAction<T>
where
    T: PartialEq + Clone + 'static,
    F1: Fn(&T) -> String + 'static + Clone,
    F2: Fn(&T, &str) -> bool + 'static + Clone,
{
    let mut action = DropdownAction::None;

    ui.label(label);
    let display_text = selected_item
        .map_or(placeholder.to_string(), |item| display_formatter(item));

    ui.horizontal(|ui| {
        if render_dropdown_button(
            ui,
            &display_text,
            &format!("Click to select {}", label.to_lowercase().replace(':', "")),
            is_open,
        ) {
            action = DropdownAction::Toggle;
        }

        if selected_item.is_some()
            && ui.button("❌").on_hover_text("Clear selection").clicked()
        {
            action = DropdownAction::Unselect;
        }
    });

    if is_open {
        let config = DropdownConfig {
            id: base_id,
            search_hint: "Search...",
            initial_chunk_size: 50,
            auto_focus: *autofocus,
            ..Default::default()
        };

        if *autofocus {
            *autofocus = false;
        }

        // Wrapper to adapt Fn(&T) to Fn(&Arc<T>)
        let fmt_clone = display_formatter.clone();
        let display_wrapper = move |item: &Arc<T>| fmt_clone(item);

        let matcher_clone = search_matcher.clone();
        let search_wrapper = move |item: &Arc<T>, search: &str| matcher_clone(item, search);

        // Current selection matcher
        let current_selected = selected_item.cloned();
        let selection_matcher = move |item: &Arc<T>| {
            current_selected.as_ref().is_some_and(|s| s == item)
        };

        let mut dropdown = SearchableDropdown::new(
            all_items,
            search_text,
            Box::new(selection_matcher),
            Box::new(display_wrapper),
            Box::new(search_wrapper),
            Box::new(|items| items.choose(&mut rand::rng()).cloned()),
            config,
            display_count,
        );

        match dropdown.render(ui) {
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

const ICON_SIZE: f32 = 4.0;
const ICON_AREA_SIZE: egui::Vec2 = egui::vec2(20.0, 20.0);
const ICON_OFFSET: egui::Vec2 = egui::vec2(21.0, 10.0);

fn paint_chevron(ui: &mut Ui, rect: egui::Rect, open: bool) {
    let painter = ui.painter();
    let center = rect.center();
    let size = ICON_SIZE;
    let fill = ui.visuals().text_color();
    let stroke = Stroke::NONE;

    let points = if open {
        vec![
            center + vec2(-size, size / 2.0),
            center + vec2(0.0, -size / 2.0),
            center + vec2(size, size / 2.0),
        ]
    } else {
        vec![
            center + vec2(-size, -size / 2.0),
            center + vec2(0.0, size / 2.0),
            center + vec2(size, -size / 2.0),
        ]
    };

    painter.add(egui::Shape::convex_polygon(points, fill, stroke));
}

fn render_dropdown_button(ui: &mut Ui, text: &str, hover_text: &str, open: bool) -> bool {
    let response = ui
        .button(format!("{}    ", text)) // Add padding for icon
        .on_hover_text(hover_text);

    let clicked = response.clicked();

    let icon_rect = egui::Rect::from_min_size(
        response.rect.right_center() - ICON_OFFSET,
        ICON_AREA_SIZE,
    );
    paint_chevron(ui, icon_rect, open);

    clicked
}
