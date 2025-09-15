use crate::database::DatabasePool;
use crate::gui::components::{
    action_panel::ActionPanel,
    route_popup::RoutePopup,
    search_panel::SearchPanel,
    selection_panel::SelectionPanel,
    table_panel::TablePanel,
};
use crate::gui::events::Event;
use crate::gui::services::{AppService, Services};
use crate::gui::state::ApplicationState;
use eframe::egui;
use log;
use std::error::Error;
use std::sync::Arc;

/// Simplified GUI application using unified state and services.
/// Much cleaner than the previous approach with many ViewModels.
pub struct SimplifiedGui {
    /// All UI state in one place - what to display, user interactions.
    state: ApplicationState,
    /// All business logic services in one container.
    services: Services,
}

impl SimplifiedGui {
    /// Creates a new simplified GUI instance.
    pub fn new(database_pool: DatabasePool) -> Result<Self, Box<dyn Error>> {
        // Create the main app service
        let app_service = AppService::new(database_pool)?;
        
        // Create the services container
        let services = Services::new(app_service);
        
        // Create the application state
        let state = ApplicationState::new();
        
        Ok(Self { state, services })
    }
    
    /// Handle events from the UI components.
    fn handle_event(&mut self, event: Event) {
        match event {
            Event::DepartureAirportSelected(airport) => {
                self.state.select_departure_airport(Some(airport));
                log::info!("Departure airport selected: {}", self.state.departure_display_text());
            }
            
            Event::AircraftSelected(aircraft) => {
                self.state.select_aircraft(Some(aircraft));
                log::info!("Aircraft selected: {}", self.state.aircraft_display_text());
            }
            
            Event::GenerateRoutesForAircraft => {
                if let (Some(airport), Some(aircraft)) = (
                    &self.state.selected_departure_airport,
                    &self.state.selected_aircraft,
                ) {
                    self.services.app.generate_routes_for_aircraft(airport.clone(), aircraft.clone());
                    self.state.all_items = self.services.app.route_items()
                        .iter()
                        .map(|route| Arc::new(crate::gui::data::TableItem::Route(route.clone())))
                        .collect();
                    log::info!("Generated {} routes for aircraft", self.state.all_items.len());
                }
            }
            
            Event::GenerateRandomRoutes => {
                self.services.app.generate_random_routes();
                self.state.all_items = self.services.app.route_items()
                    .iter()
                    .map(|route| Arc::new(crate::gui::data::TableItem::Route(route.clone())))
                    .collect();
                log::info!("Generated {} random routes", self.state.all_items.len());
            }
            
            Event::GenerateNotFlownRoutes => {
                self.services.app.generate_not_flown_routes();
                self.state.all_items = self.services.app.route_items()
                    .iter()
                    .map(|route| Arc::new(crate::gui::data::TableItem::Route(route.clone())))
                    .collect();
                log::info!("Generated {} not flown routes", self.state.all_items.len());
            }
            
            Event::SearchChanged => {
                // Search handling is done in SearchPanel component
                log::info!("Search changed to: '{}'", self.state.table_search);
            }
            
            Event::SearchCleared => {
                self.state.set_table_search(String::new());
                log::info!("Search cleared");
            }
            
            Event::RouteSelected(route) => {
                // Open route popup
                self.services.popup.open_route_popup(route);
                log::info!("Route selected for flying");
            }
            
            Event::ShowStatistics => {
                self.services.popup.open_statistics_popup();
                log::info!("Statistics requested");
            }
            
            // Handle other events...
            _ => {
                log::debug!("Unhandled event: {:?}", event);
            }
        }
    }
}

impl eframe::App for SimplifiedGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Flight Planner - Simplified Architecture");
            
            ui.separator();
            
            // Selection Panel - Clean interface, no ViewModel needed!
            let mut events = SelectionPanel::render(&mut self.state, &self.services, ui);
            
            ui.separator();
            
            // Action Panel - Simple and clean
            events.extend(ActionPanel::render(&self.state, &self.services, ui));
            
            ui.separator();
            
            // Search Panel - No complex parameter passing
            events.extend(SearchPanel::render(&mut self.state, &mut self.services, ui));
            
            ui.separator();
            
            // Table Panel - Unified display logic
            events.extend(TablePanel::render(&self.state, &self.services, ui));
            
            // Handle all events at once
            for event in events {
                self.handle_event(event);
            }
        });
        
        // Handle popup dialogs
        RoutePopup::render(&mut self.services.popup, ctx);
    }
}

// For backward compatibility during transition
pub use crate::gui::ui::Gui;