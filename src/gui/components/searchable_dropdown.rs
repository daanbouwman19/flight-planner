use egui::Ui;

/// Type alias for search matcher function
type SearchMatcher<'a, T> = Box<dyn Fn(&T, &str) -> bool + 'a>;

/// Type alias for random selector function  
type RandomSelector<'a, T> = Box<dyn Fn(&[T]) -> Option<T> + 'a>;

/// Type alias for current selection matcher function
type CurrentSelectionMatcher<'a, T> = Box<dyn Fn(&T) -> bool + 'a>;

/// Type alias for display formatter function
type DisplayFormatter<'a, T> = Box<dyn Fn(&T) -> String + 'a>;

/// A generic, reusable UI component for creating a searchable dropdown list.
///
/// This component is highly configurable and supports features like:
/// - Text-based search/filtering.
/// - Random item selection.
/// - An "unspecified" or "none" option.
/// - Lazy loading/chunked display for large lists to maintain performance.
/// - Customizable display formatting and search logic via closures.
pub struct SearchableDropdown<'a, T> {
    /// A slice of items of type `T` to be displayed in the dropdown.
    pub items: &'a [T],
    /// A mutable reference to the string holding the current search text.
    pub search_text: &'a mut String,
    /// A closure that determines if a given item is the currently selected one.
    pub current_selection_matcher: CurrentSelectionMatcher<'a, T>,
    /// A closure that formats an item of type `T` into a display string.
    pub display_formatter: DisplayFormatter<'a, T>,
    /// A closure that defines the logic for matching an item against a search query.
    pub search_matcher: SearchMatcher<'a, T>,
    /// A closure that defines the logic for selecting a random item from the list.
    pub random_selector: RandomSelector<'a, T>,
    /// Configuration settings for the dropdown's behavior and appearance.
    pub config: DropdownConfig<'a>,
    /// A mutable reference to a counter for managing chunked display of items.
    pub current_display_count: &'a mut usize,
}

/// Configuration settings for the `SearchableDropdown` component.
pub struct DropdownConfig<'a> {
    /// A unique identifier for the dropdown, used for egui's widget ID system.
    pub id: &'a str,
    /// If `true`, the search input field will be automatically focused when the dropdown is opened.
    pub auto_focus: bool,
    /// The text to display for the "random selection" option.
    pub random_option_text: &'a str,
    /// The text to display for the "unspecified" or "none" option.
    pub unspecified_option_text: &'a str,
    /// A flag indicating whether the "unspecified" option is currently selected.
    pub is_unspecified_selected: bool,
    /// The hint text to display in the search input field when it's empty.
    pub search_hint: &'a str,
    /// The number of items to display initially and to load in each subsequent chunk.
    pub initial_chunk_size: usize,
    /// The minimum number of characters required before a search is performed.
    pub min_search_length: usize,
    /// The maximum number of search results to display (0 for no limit).
    pub max_results: usize,
    /// The text to display when a search yields no results.
    pub no_results_text: &'a str,
    /// Additional lines of help text to display when there are no search results.
    pub no_results_help: &'a [&'a str],
    /// The minimum width of the dropdown panel.
    pub min_width: f32,
    /// The maximum height of the dropdown panel.
    pub max_height: f32,
}

/// Represents the result of a user's interaction with a `SearchableDropdown`.
#[derive(Debug, Clone)]
pub enum DropdownSelection<T> {
    /// The user selected a specific item from the list.
    Item(T),
    /// The user selected the "random" option, and a random item was chosen.
    Random(T),
    /// The user selected the "unspecified" or "none" option.
    Unspecified,
    /// No selection was made during this render pass.
    None,
}

impl<'a, T: Clone> SearchableDropdown<'a, T> {
    /// Creates a new `SearchableDropdown` component.
    ///
    /// # Arguments
    ///
    /// This method takes numerous arguments to configure the dropdown's appearance
    /// and behavior, including the items to display, mutable state references,
    /// and closures for custom logic.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        items: &'a [T],
        search_text: &'a mut String,
        current_selection_matcher: CurrentSelectionMatcher<'a, T>,
        display_formatter: DisplayFormatter<'a, T>,
        search_matcher: SearchMatcher<'a, T>,
        random_selector: RandomSelector<'a, T>,
        config: DropdownConfig<'a>,
        current_display_count: &'a mut usize,
    ) -> Self {
        // Initialize display count if it's 0
        if *current_display_count == 0 {
            *current_display_count = config.initial_chunk_size;
        }

        Self {
            items,
            search_text,
            current_selection_matcher,
            display_formatter,
            search_matcher,
            random_selector,
            config,
            current_display_count,
        }
    }

    /// Renders the searchable dropdown UI and returns the user's selection.
    ///
    /// # Arguments
    ///
    /// * `ui` - A mutable reference to the `egui::Ui` context for rendering.
    ///
    /// # Returns
    ///
    /// A `DropdownSelection<T>` indicating what, if anything, the user selected.
    pub fn render(&mut self, ui: &mut Ui) -> DropdownSelection<T> {
        let mut selection = DropdownSelection::None;

        ui.group(|ui| {
            ui.set_min_width(self.config.min_width);
            ui.set_max_height(self.config.max_height);

            // Search field at the top
            ui.horizontal(|ui| {
                ui.label("🔍");
                let search_response = ui.add(
                    egui::TextEdit::singleline(self.search_text)
                        .hint_text(self.config.search_hint)
                        .desired_width(ui.available_width() - 30.0)
                        .id(egui::Id::new(format!("{}_search", self.config.id))),
                );

                // Auto-focus the search field when dropdown is first opened (if enabled)
                if self.config.auto_focus {
                    search_response.request_focus();
                }
            });
            ui.separator();

            selection = self.render_dropdown_list(ui);
        });

        selection
    }

    /// Renders the dropdown list content
    fn render_dropdown_list(&mut self, ui: &mut Ui) -> DropdownSelection<T> {
        let mut selection = DropdownSelection::None;
        let search_text_lower = self.search_text.to_lowercase();
        let current_search_empty = self.search_text.is_empty();

        egui::ScrollArea::vertical()
            .max_height(250.0)
            .auto_shrink([false, true])
            .id_salt(format!("{}_main_scroll", self.config.id))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());

                // Always show option for random selection at the top
                if ui
                    .selectable_label(false, self.config.random_option_text)
                    .clicked()
                    && let Some(random_item) = (self.random_selector)(self.items)
                {
                    selection = DropdownSelection::Random(random_item);
                }

                // Option for unspecified selection
                if ui
                    .selectable_label(
                        self.config.is_unspecified_selected,
                        self.config.unspecified_option_text,
                    )
                    .clicked()
                {
                    selection = DropdownSelection::Unspecified;
                }

                ui.separator();

                // Handle search display
                if current_search_empty {
                    // Always show items in chunks for performance when search is empty
                    self.render_all_items_chunked(ui, &mut selection);
                } else if self.config.min_search_length > 0
                    && self.search_text.len() < self.config.min_search_length
                {
                    ui.label(format!(
                        "💡 Type at least {} characters to search",
                        self.config.min_search_length
                    ));
                } else {
                    // Show filtered items
                    self.render_filtered_items(ui, search_text_lower.trim(), &mut selection);
                }
            });

        selection
    }
}

impl Default for DropdownConfig<'_> {
    fn default() -> Self {
        Self {
            id: "default_dropdown",
            auto_focus: false, // Don't auto-focus by default to avoid conflicts
            random_option_text: "🎲 Pick random",
            unspecified_option_text: "🔀 No specific selection",
            is_unspecified_selected: false,
            search_hint: "Type to search...",
            initial_chunk_size: 100, // Show first 100 items by default
            min_search_length: 0,
            max_results: 0, // No limit
            no_results_text: "🔍 No results found",
            no_results_help: &["   Try different search terms"],
            min_width: 300.0,
            max_height: 300.0,
        }
    }
}

impl<T: Clone> SearchableDropdown<'_, T> {
    /// Renders all items in chunks for performance with auto-loading
    fn render_all_items_chunked(
        &mut self,
        ui: &mut egui::Ui,
        selection: &mut DropdownSelection<T>,
    ) {
        let total_items = self.items.len();
        let items_to_show = (*self.current_display_count).min(total_items);

        let scroll_area = egui::ScrollArea::vertical()
            .max_height(200.0)
            .auto_shrink([false, true])
            .id_salt(format!("{}_chunked_scroll", self.config.id))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());

                for item in self.items.iter().take(items_to_show) {
                    let display_text = (self.display_formatter)(item);
                    let is_selected = (self.current_selection_matcher)(item);

                    if ui.selectable_label(is_selected, display_text).clicked() {
                        *selection = DropdownSelection::Item(item.clone());
                    }
                }

                // Add some padding at the bottom for auto-loading detection
                ui.add_space(20.0);
            });

        // Auto-load more items when user scrolls near the bottom
        if items_to_show < total_items {
            let scroll_offset = scroll_area.state.offset.y;
            let content_height = scroll_area.content_size.y;
            let viewport_height = scroll_area.inner_rect.height();

            // Load more when scrolled to within 50px of the bottom
            if scroll_offset + viewport_height + 50.0 >= content_height {
                *self.current_display_count += self.config.initial_chunk_size;
            }

            // Also show remaining count
            let remaining = total_items - items_to_show;
            ui.separator();
            ui.label(format!(
                "📄 {remaining} more items available (scroll to load)"
            ));
        }
    }

    /// Renders filtered items based on search
    fn render_filtered_items(
        &self,
        ui: &mut egui::Ui,
        search_text_lower: &str,
        selection: &mut DropdownSelection<T>,
    ) {
        let mut found_matches = false;
        let mut match_count = 0;

        for item in self.items {
            if self.config.max_results > 0 && match_count >= self.config.max_results {
                break;
            }

            if (self.search_matcher)(item, search_text_lower) {
                found_matches = true;
                match_count += 1;

                let display_text = (self.display_formatter)(item);
                let is_selected = (self.current_selection_matcher)(item);

                if ui.selectable_label(is_selected, display_text).clicked() {
                    *selection = DropdownSelection::Item(item.clone());
                }
            }
        }

        // Show helpful messages based on search results
        if !found_matches {
            ui.label(self.config.no_results_text);
            for help_line in self.config.no_results_help {
                ui.label(*help_line);
            }
        } else if self.config.max_results > 0 && match_count >= self.config.max_results {
            ui.separator();
            ui.label(format!(
                "📄 Showing first {} results - refine search for more specific results",
                self.config.max_results
            ));
        }
    }
}
