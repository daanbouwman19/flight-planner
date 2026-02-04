use egui::Ui;

/// Type alias for search matcher function
type SearchMatcher<'a, T> = Box<dyn Fn(&T, &str) -> bool + 'a>;

/// Type alias for random selector function  
type RandomSelector<'a, T> = Box<dyn Fn(&[T]) -> Option<T> + 'a>;

/// Type alias for current selection matcher function
type CurrentSelectionMatcher<'a, T> = Box<dyn Fn(&T) -> bool + 'a>;

/// Type alias for display formatter function
type DisplayFormatter<'a, T> = Box<dyn Fn(&T) -> String + 'a>;

/// Type alias for tooltip formatter function
type TooltipFormatter<'a, T> = Box<dyn Fn(&T) -> Option<String> + 'a>;

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
    /// A closure that returns an optional tooltip for an item.
    pub tooltip_formatter: TooltipFormatter<'a, T>,
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

/// Internal state for keyboard navigation.
#[derive(Clone, Copy, Debug, Default)]
struct DropdownNavigationState {
    highlighted_index: usize,
}

/// Internal cache for search results to avoid O(N) filtering every frame.
#[derive(Clone, Default)]
struct SearchCache {
    /// The search query for which these results are valid.
    query: String,
    /// Indices of items that match the query.
    matches: Vec<usize>,
    /// The index in `items` where the next search scan should start.
    next_index: usize,
    /// Whether we have finished scanning all items.
    done: bool,
    /// The length of the items slice when this cache was created (safety check).
    items_len: usize,
}

/// Internal cache for display strings to avoid allocations every frame.
#[derive(Clone, Default)]
struct DisplayCache {
    /// We use Arc<Mutex> to avoid deep cloning the HashMap every frame when retrieving from egui memory.
    /// Maps item index -> Display String.
    cache: std::sync::Arc<std::sync::Mutex<std::collections::HashMap<usize, String>>>,
    /// The length of the items slice when this cache was created (safety check).
    items_len: usize,
    /// The memory address of the items slice pointer to detect re-allocations/swaps.
    items_ptr_addr: usize,
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
        tooltip_formatter: TooltipFormatter<'a, T>,
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
            tooltip_formatter,
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
    #[cfg(not(tarpaulin_include))]
    pub fn render(&mut self, ui: &mut Ui) -> DropdownSelection<T> {
        let mut selection = DropdownSelection::None;

        ui.group(|ui| {
            ui.set_min_width(self.config.min_width);
            ui.set_max_height(self.config.max_height);

            let search_input_id = ui.make_persistent_id(self.config.id).with("search");

            // Search field at the top
            ui.horizontal(|ui| {
                ui.label("🔍");

                let has_text = !self.search_text.is_empty();
                let clear_button_size = 20.0;
                let spacing = 5.0;
                // Reserve space for the clear button if visible, plus standard padding/safety
                let reserved_width = if has_text {
                    clear_button_size + spacing
                } else {
                    0.0
                } + 10.0;

                let search_response = ui.add(
                    egui::TextEdit::singleline(self.search_text)
                        .hint_text(self.config.search_hint)
                        .desired_width(ui.available_width() - reserved_width)
                        .id(search_input_id),
                );

                // --- Keyboard Navigation Logic ---
                // Fetch total navigable items from cache or items length
                let cache_id = ui.make_persistent_id(self.config.id).with("search_cache");
                let cache = ui
                    .data(|d| d.get_temp::<SearchCache>(cache_id))
                    .unwrap_or_default();

                let list_count = if self.search_text.is_empty() {
                    self.items.len()
                } else {
                    cache.matches.len()
                };

                // 0: Random, 1: Unspecified, 2+: Items
                let total_navigable_count = 2 + list_count;

                let nav_state_id = ui.make_persistent_id(self.config.id).with("nav_state");
                let mut nav_state = ui
                    .data_mut(|d| d.get_temp::<DropdownNavigationState>(nav_state_id))
                    .unwrap_or_default();

                if search_response.has_focus() {
                    if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                        nav_state.highlighted_index =
                            (nav_state.highlighted_index + 1).min(total_navigable_count - 1);

                        // Ensure the highlighted item is rendered (handle lazy loading)
                        if nav_state.highlighted_index >= 2 {
                            let list_index = nav_state.highlighted_index - 2;
                            if list_index >= *self.current_display_count {
                                *self.current_display_count =
                                    list_index + 1 + self.config.initial_chunk_size;
                            }
                        }

                        ui.input_mut(|i| {
                            i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown)
                        });
                    } else if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                        nav_state.highlighted_index = nav_state.highlighted_index.saturating_sub(1);
                        ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp));
                    } else if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        // Determine selection based on index
                        if nav_state.highlighted_index == 0 {
                            if let Some(random_item) = (self.random_selector)(self.items) {
                                selection = DropdownSelection::Random(random_item);
                            }
                        } else if nav_state.highlighted_index == 1 {
                            selection = DropdownSelection::Unspecified;
                        } else {
                            let item_index_in_list = nav_state.highlighted_index - 2;
                            if self.search_text.is_empty() {
                                if item_index_in_list < self.items.len() {
                                    selection = DropdownSelection::Item(
                                        self.items[item_index_in_list].clone(),
                                    );
                                }
                            } else if item_index_in_list < cache.matches.len() {
                                let real_index = cache.matches[item_index_in_list];
                                if real_index < self.items.len() {
                                    selection =
                                        DropdownSelection::Item(self.items[real_index].clone());
                                }
                            }
                        }
                        ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter));
                    }
                }

                // Save updated state
                ui.data_mut(|d| d.insert_temp(nav_state_id, nav_state));

                if has_text
                    && ui
                        .add_sized(
                            [clear_button_size, clear_button_size],
                            egui::Button::new("×").small().frame(false),
                        )
                        .on_hover_text("Clear search")
                        .clicked()
                {
                    self.search_text.clear();
                    search_response.request_focus();
                }

                // Auto-focus the search field when dropdown is first opened (if enabled)
                if self.config.auto_focus {
                    search_response.request_focus();
                }
            });
            ui.separator();

            if let DropdownSelection::None = selection {
                selection = self.render_dropdown_list(ui, search_input_id);
            }
        });

        selection
    }

    /// Renders the dropdown list content
    #[cfg(not(tarpaulin_include))]
    fn render_dropdown_list(
        &mut self,
        ui: &mut Ui,
        search_input_id: egui::Id,
    ) -> DropdownSelection<T> {
        let mut selection = DropdownSelection::None;
        let current_search_empty = self.search_text.is_empty();

        // Fetch nav state
        let nav_state_id = ui.make_persistent_id(self.config.id).with("nav_state");
        let nav_state = ui
            .data(|d| d.get_temp::<DropdownNavigationState>(nav_state_id))
            .unwrap_or_default();

        // Capture scroll area output to handle infinite scrolling
        let scroll_output = egui::ScrollArea::vertical()
            .max_height(250.0)
            .auto_shrink([false, true])
            .id_salt(ui.make_persistent_id(self.config.id).with("main_scroll"))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());

                // Always show option for random selection at the top
                let is_random_highlighted = nav_state.highlighted_index == 0;
                let random_response =
                    ui.selectable_label(is_random_highlighted, self.config.random_option_text);
                if is_random_highlighted {
                    random_response.scroll_to_me(Some(egui::Align::Center));
                }
                if random_response.clicked()
                    && let Some(random_item) = (self.random_selector)(self.items)
                {
                    selection = DropdownSelection::Random(random_item);
                }

                // Option for unspecified selection
                let is_unspecified_highlighted = nav_state.highlighted_index == 1;
                let unspecified_response = ui.selectable_label(
                    self.config.is_unspecified_selected || is_unspecified_highlighted,
                    self.config.unspecified_option_text,
                );
                if is_unspecified_highlighted {
                    unspecified_response.scroll_to_me(Some(egui::Align::Center));
                }
                if unspecified_response.clicked() {
                    selection = DropdownSelection::Unspecified;
                }

                ui.separator();

                // --- Display Cache Setup ---
                let display_cache_id = ui.make_persistent_id(self.config.id).with("display_cache");
                let mut display_cache = ui
                    .data_mut(|d| d.get_temp::<DisplayCache>(display_cache_id))
                    .unwrap_or_default();

                // Invalidate cache if items length or pointer changes (e.g., list reloaded)
                if display_cache.items_len != self.items.len()
                    || display_cache.items_ptr_addr != self.items.as_ptr() as usize
                {
                    if let Ok(mut map) = display_cache.cache.lock() {
                        map.clear();
                    }
                    display_cache.items_len = self.items.len();
                    display_cache.items_ptr_addr = self.items.as_ptr() as usize;
                }

                // Handle search display
                let has_more = if current_search_empty {
                    // Always show items in chunks for performance when search is empty
                    self.render_all_items_chunked(
                        ui,
                        &mut selection,
                        &display_cache,
                        nav_state.highlighted_index,
                    )
                } else if self.config.min_search_length > 0
                    && self.search_text.len() < self.config.min_search_length
                {
                    ui.label(format!(
                        "💡 Type at least {} characters to search",
                        self.config.min_search_length
                    ));
                    false
                } else {
                    let search_text_lower = self.search_text.to_lowercase();
                    // Show filtered items (now virtualized!)
                    self.render_filtered_items(
                        ui,
                        search_text_lower.trim(),
                        &mut selection,
                        &display_cache,
                        nav_state.highlighted_index,
                        search_input_id,
                    )
                };

                // Store updated cache back to memory
                ui.data_mut(|d| d.insert_temp(display_cache_id, display_cache));

                // Add some padding at the bottom for auto-loading detection
                ui.add_space(20.0);

                has_more
            });

        // Handle infinite scroll logic outside the closure
        let has_more = scroll_output.inner;
        if has_more {
            let state = scroll_output.state;
            let scroll_offset = state.offset.y;
            let content_height = scroll_output.content_size.y;
            let viewport_height = scroll_output.inner_rect.height();

            // Load more when scrolled to within 50px of the bottom
            if scroll_offset + viewport_height + 50.0 >= content_height {
                *self.current_display_count += self.config.initial_chunk_size;
            }
        }

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
    /// Helper to render a single item label with correct styling and behavior
    fn render_item_label(
        &self,
        ui: &mut egui::Ui,
        item: &T,
        display_text: &str,
        is_selected: bool,
        is_highlighted: bool,
    ) -> egui::Response {
        let mut response = ui.selectable_label(is_selected || is_highlighted, display_text);

        if is_highlighted {
            response.scroll_to_me(Some(egui::Align::Center));
        }

        if let Some(tooltip) = (self.tooltip_formatter)(item) {
            response = response.on_hover_text(tooltip);
        }

        response
    }

    /// Renders all items in chunks for performance with auto-loading.
    /// Returns true if there are more items to load.
    #[cfg(not(tarpaulin_include))]
    fn render_all_items_chunked(
        &mut self,
        ui: &mut egui::Ui,
        selection: &mut DropdownSelection<T>,
        display_cache: &DisplayCache,
        highlighted_index: usize,
    ) -> bool {
        let total_items = self.items.len();
        let items_to_show = (*self.current_display_count).min(total_items);

        // Lock the cache map once for the entire loop
        if let Ok(mut map) = display_cache.cache.lock() {
            // Render items directly without a nested ScrollArea
            for (i, item) in self.items.iter().enumerate().take(items_to_show) {
                // Use entry API to get or insert the formatted string, preventing allocation for existing items
                let display_text = map
                    .entry(i)
                    .or_insert_with(|| (self.display_formatter)(item));

                let is_selected = (self.current_selection_matcher)(item);
                let is_highlighted = (i + 2) == highlighted_index;

                let response = self.render_item_label(
                    ui,
                    item,
                    display_text.as_str(),
                    is_selected,
                    is_highlighted,
                );

                if response.clicked() {
                    *selection = DropdownSelection::Item(item.clone());
                }
            }
        } else {
            // Fallback if lock fails (shouldn't happen in single-threaded GUI)
            for (i, item) in self.items.iter().enumerate().take(items_to_show) {
                let display_text = (self.display_formatter)(item);
                let is_selected = (self.current_selection_matcher)(item);
                let is_highlighted = (i + 2) == highlighted_index;

                let response =
                    self.render_item_label(ui, item, &display_text, is_selected, is_highlighted);

                if response.clicked() {
                    *selection = DropdownSelection::Item(item.clone());
                }
            }
        }

        let has_more = items_to_show < total_items;

        if has_more {
            // Also show remaining count
            let remaining = total_items - items_to_show;
            ui.separator();
            ui.label(format!(
                "📄 {remaining} more items available (scroll to load)"
            ));
        }

        has_more
    }

    /// Renders filtered items based on search with virtualization.
    /// Returns true if there are more matching items to load.
    #[cfg(not(tarpaulin_include))]
    fn render_filtered_items(
        &mut self,
        ui: &mut egui::Ui,
        search_text_lower: &str,
        selection: &mut DropdownSelection<T>,
        display_cache: &DisplayCache,
        highlighted_index: usize,
        search_input_id: egui::Id,
    ) -> bool {
        let max_display = *self.current_display_count;
        let hard_limit = if self.config.max_results > 0 {
            self.config.max_results
        } else {
            usize::MAX
        };

        // We stop rendering when we reach max_display or hard_limit.
        let render_limit = max_display.min(hard_limit);

        // --- Cache Logic ---
        // We use egui's temporary memory to store the state of the search.
        // This avoids re-scanning the entire list (O(N)) every frame.
        // Instead, we resume scanning from where we left off (O(K) per frame),
        // or just read from cache if we already found enough matches.
        let cache_id = ui.make_persistent_id(self.config.id).with("search_cache");
        let mut cache = ui
            .data_mut(|d| d.get_temp::<SearchCache>(cache_id))
            .unwrap_or_default();

        // Invalidate cache if query changes or underlying data size changes
        if cache.query != search_text_lower || cache.items_len != self.items.len() {
            cache = SearchCache {
                query: search_text_lower.to_owned(),
                items_len: self.items.len(),
                ..Default::default()
            };
        }

        // Resume search if we need more matches than currently cached
        // We check `matches.len() <= render_limit` to find at least one more match than needed
        // so we can correctly determine `has_more`.
        if !cache.done && cache.matches.len() <= render_limit {
            for (i, item) in self.items.iter().enumerate().skip(cache.next_index) {
                if (self.search_matcher)(item, &cache.query) {
                    cache.matches.push(i);
                    // Stop if we found enough for this frame (limit + 1)
                    if cache.matches.len() > render_limit {
                        cache.next_index = i + 1;
                        break;
                    }
                }
                cache.next_index = i + 1;
            }

            if cache.next_index >= self.items.len() {
                cache.done = true;
            }
        }

        // Store updated cache back to memory
        ui.data_mut(|d| d.insert_temp(cache_id, cache.clone()));

        // --- Render from Cache ---
        let mut match_count = 0;
        let mut found_matches = false;

        // Lock display cache
        if let Ok(mut map) = display_cache.cache.lock() {
            for (match_idx, &idx) in cache.matches.iter().enumerate() {
                if match_count >= render_limit {
                    break;
                }

                if idx < self.items.len() {
                    let item = &self.items[idx];
                    found_matches = true;
                    match_count += 1;

                    let display_text = map
                        .entry(idx)
                        .or_insert_with(|| (self.display_formatter)(item));

                    let is_selected = (self.current_selection_matcher)(item);
                    let is_highlighted = (match_idx + 2) == highlighted_index;

                    let response = self.render_item_label(
                        ui,
                        item,
                        display_text.as_str(),
                        is_selected,
                        is_highlighted,
                    );

                    if response.clicked() {
                        *selection = DropdownSelection::Item(item.clone());
                    }
                }
            }
        } else {
            // Fallback
            for (match_idx, &idx) in cache.matches.iter().enumerate() {
                if match_count >= render_limit {
                    break;
                }

                if idx < self.items.len() {
                    let item = &self.items[idx];
                    found_matches = true;
                    match_count += 1;

                    let display_text = (self.display_formatter)(item);
                    let is_selected = (self.current_selection_matcher)(item);
                    let is_highlighted = (match_idx + 2) == highlighted_index;

                    let response = self.render_item_label(
                        ui,
                        item,
                        &display_text,
                        is_selected,
                        is_highlighted,
                    );

                    if response.clicked() {
                        *selection = DropdownSelection::Item(item.clone());
                    }
                }
            }
        }

        // Determine if there are more items available
        let has_more = (!cache.done) || (cache.matches.len() > render_limit);

        // Show helpful messages based on search results
        if !found_matches && cache.done && cache.matches.is_empty() {
            ui.vertical_centered(|ui| {
                ui.label(
                    egui::RichText::new(self.config.no_results_text)
                        .strong()
                        .color(ui.visuals().weak_text_color()),
                );
                for help_line in self.config.no_results_help {
                    ui.label(egui::RichText::new(*help_line).small().weak());
                }
                ui.add_space(5.0);
                if ui
                    .button("❌ Clear Search")
                    .on_hover_text("Clear the current search filter")
                    .clicked()
                {
                    self.search_text.clear();
                    ui.memory_mut(|m| m.request_focus(search_input_id));
                }
            });
        } else if has_more {
            ui.separator();
            ui.label("📄 More items available (scroll to load)");
        } else if self.config.max_results > 0 && match_count >= self.config.max_results {
            ui.separator();
            ui.label(format!(
                "📄 Showing first {} results - refine search for more specific results",
                self.config.max_results
            ));
        }

        has_more
    }
}
