# REAL Search Implementation

**Using actual KeyboardInput and Text patterns from bevy/examples/input/text_input.rs - no bullshit components**

## Current Problem
- No search functionality at all
- Need real-time filtering as user types  
- Need fuzzy matching for app names
- Need result spawning/despawning

## REAL Solution from Bevy Examples

### Text Input System (REAL pattern)

**Pattern from:** `bevy/examples/input/text_input.rs`

```rust
#[derive(Component)]
struct SearchInput;

// REAL input handling - exact pattern from text_input.rs
fn search_input_system(
    mut events: EventReader<KeyboardInput>,
    mut search_text: Query<&mut Text, With<SearchInput>>,
    mut search_results: EventWriter<SearchQueryChanged>,
) {
    let Ok(mut text) = search_text.get_single_mut() else { return };
    
    for event in events.read() {
        // Only trigger changes when the key is first pressed.
        if !event.state.is_pressed() {
            continue;
        }
        
        let mut query_changed = false;
        
        match (&event.logical_key, &event.text) {
            (Key::Backspace, _) => {
                if **text == "Search..." {
                    // Clear placeholder
                    **text = String::new();
                } else {
                    text.pop();
                }
                query_changed = true;
            }
            (Key::Escape, _) => {
                **text = "Search...".to_string();
                query_changed = true;
            }
            (_, Some(inserted_text)) => {
                // Clear placeholder on first input
                if **text == "Search..." {
                    **text = String::new();
                }
                
                // Make sure the text doesn't have any control characters
                if inserted_text.chars().all(is_printable_char) {
                    text.push_str(inserted_text);
                    query_changed = true;
                }
            }
            _ => continue,
        }
        
        // Send search event when query changes
        if query_changed {
            search_results.send(SearchQueryChanged {
                query: if **text == "Search..." { String::new() } else { text.to_string() },
            });
        }
    }
}

// From text_input.rs example
fn is_printable_char(chr: char) -> bool {
    let is_in_private_use_area = ('\u{e000}'..='\u{f8ff}').contains(&chr)
        || ('\u{f0000}'..='\u{ffffd}').contains(&chr)
        || ('\u{100000}'..='\u{10fffd}').contains(&chr);

    !is_in_private_use_area && !chr.is_ascii_control()
}

#[derive(Event)]
struct SearchQueryChanged {
    query: String,
}
```

### Application Discovery (REAL patterns)

```rust
#[derive(Resource)]
struct ApplicationIndex {
    apps: Vec<AppInfo>,
}

#[derive(Clone, Debug)]
struct AppInfo {
    name: String,
    path: String,
    keywords: Vec<String>,
}

// REAL resource initialization
fn discover_applications_system(mut commands: Commands) {
    let mut apps = Vec::new();
    
    // macOS application discovery
    #[cfg(target_os = "macos")]
    {
        let app_dirs = ["/Applications", "/Applications/Utilities"];
        
        for dir in &app_dirs {
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    if let Some(name) = entry.file_name().to_str() {
                        if name.ends_with(".app") {
                            let app_name = name.trim_end_matches(".app").to_string();
                            apps.push(AppInfo {
                                name: app_name.clone(),
                                path: entry.path().display().to_string(),
                                keywords: vec![app_name.to_lowercase()],
                            });
                        }
                    }
                }
            }
        }
    }
    
    // Add built-in actions
    apps.extend(vec![
        AppInfo {
            name: "System Preferences".to_string(),
            path: "com.apple.systempreferences".to_string(),
            keywords: vec!["preferences".to_string(), "settings".to_string()],
        },
        AppInfo {
            name: "Activity Monitor".to_string(),
            path: "/Applications/Utilities/Activity Monitor.app".to_string(),
            keywords: vec!["activity".to_string(), "monitor".to_string(), "cpu".to_string()],
        },
    ]);
    
    commands.insert_resource(ApplicationIndex { apps });
}
```

### Fuzzy Search (REAL implementation)

```rust
// REAL fuzzy matching - simple and fast
fn fuzzy_match(query: &str, target: &str) -> Option<f32> {
    if query.is_empty() {
        return Some(1.0);
    }
    
    let query_lower = query.to_lowercase();
    let target_lower = target.to_lowercase();
    
    // Exact match gets highest score
    if target_lower.contains(&query_lower) {
        return Some(1.0);
    }
    
    // Character subsequence matching
    let mut query_chars = query_lower.chars();
    let mut current_char = query_chars.next()?;
    let mut matches = 0;
    let mut total_chars = 0;
    
    for target_char in target_lower.chars() {
        total_chars += 1;
        if target_char == current_char {
            matches += 1;
            if let Some(next_char) = query_chars.next() {
                current_char = next_char;
            } else {
                // All query characters matched
                return Some(matches as f32 / total_chars as f32);
            }
        }
    }
    
    None // Not all query characters found
}

// Search system
fn search_system(
    mut search_events: EventReader<SearchQueryChanged>,
    app_index: Res<ApplicationIndex>,
    mut search_results: ResMut<SearchResults>,
) {
    for event in search_events.read() {
        search_results.clear();
        
        if event.query.is_empty() {
            continue;
        }
        
        let mut scored_results: Vec<_> = app_index.apps
            .iter()
            .filter_map(|app| {
                // Try matching name
                if let Some(score) = fuzzy_match(&event.query, &app.name) {
                    return Some((app, score + 0.5)); // Bonus for name match
                }
                
                // Try matching keywords
                for keyword in &app.keywords {
                    if let Some(score) = fuzzy_match(&event.query, keyword) {
                        return Some((app, score));
                    }
                }
                
                None
            })
            .collect();
        
        // Sort by score (highest first)
        scored_results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        // Take top 8 results (like Raycast)
        search_results.items = scored_results
            .into_iter()
            .take(8)
            .map(|(app, score)| SearchResult {
                name: app.name.clone(),
                path: app.path.clone(),
                score,
            })
            .collect();
    }
}

#[derive(Resource, Default)]
struct SearchResults {
    items: Vec<SearchResult>,
}

#[derive(Clone, Debug)]
struct SearchResult {
    name: String,
    path: String,
    score: f32,
}
```

### Results UI System (REAL patterns)

**Pattern from:** `bevy/examples/ui/scroll.rs` dynamic spawning

```rust
#[derive(Component)]
struct ResultsContainer;

#[derive(Component)]
struct ResultItem {
    app_path: String,
}

// REAL result spawning system
fn update_results_system(
    mut commands: Commands,
    search_results: Res<SearchResults>,
    results_container: Query<Entity, With<ResultsContainer>>,
    existing_results: Query<Entity, With<ResultItem>>,
    fonts: Res<UiFonts>,
) {
    if !search_results.is_changed() {
        return;
    }
    
    let Ok(container) = results_container.get_single() else { return };
    
    // Despawn existing results
    for entity in existing_results.iter() {
        commands.entity(entity).despawn_recursive();
    }
    
    // Spawn new results
    commands.entity(container).with_children(|parent| {
        for (idx, result) in search_results.items.iter().enumerate() {
            let is_selected = idx == 0; // First item selected by default
            
            parent.spawn((
                Button,
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(48.0),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    padding: UiRect::all(Val::Px(8.0)),
                    ..default()
                },
                // Use selected or default gradient
                if is_selected {
                    BackgroundGradient::from(LinearGradient::to_bottom(vec![
                        ColorStop::new(Color::srgba(0.0, 0.48, 1.0, 0.60), Val::Percent(0.0)),
                        ColorStop::new(Color::srgba(0.0, 0.38, 0.80, 0.80), Val::Percent(100.0)),
                    ]))
                } else {
                    BackgroundGradient::from(LinearGradient::to_bottom(vec![
                        ColorStop::new(Color::srgba(0.13, 0.13, 0.15, 0.80), Val::Percent(0.0)),
                        ColorStop::new(Color::srgba(0.11, 0.11, 0.13, 0.85), Val::Percent(100.0)),
                    ]))
                },
                BorderRadius::all(Val::Px(6.0)),
                ResultItem {
                    app_path: result.path.clone(),
                },
            ))
            .with_children(|parent| {
                // App icon placeholder
                parent.spawn((
                    Node {
                        width: Val::Px(32.0),
                        height: Val::Px(32.0),
                        margin: UiRect::right(Val::Px(12.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.25, 0.25, 0.25, 1.0)),
                    BorderRadius::all(Val::Px(4.0)),
                ));
                
                // App name
                parent.spawn((
                    Text::new(&result.name),
                    TextFont {
                        font: fonts.medium.clone(),
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(Color::srgba(0.95, 0.95, 0.97, 1.0)),
                    Node {
                        flex_grow: 1.0,
                        ..default()
                    },
                ));
            });
        }
    });
}
```

### Navigation System (REAL patterns)

**Pattern from:** `bevy/examples/input/keyboard_input_events.rs`

```rust
#[derive(Resource, Default)]
struct SearchSelection {
    index: usize,
}

fn search_navigation_system(
    mut key_events: EventReader<KeyboardInput>,
    mut selection: ResMut<SearchSelection>,
    search_results: Res<SearchResults>,
    mut result_items: Query<(&mut BackgroundGradient, &ResultItem)>,
) {
    for event in key_events.read() {
        if !event.state.is_pressed() {
            continue;
        }
        
        let mut selection_changed = false;
        
        match event.logical_key {
            Key::ArrowDown => {
                if selection.index + 1 < search_results.items.len() {
                    selection.index += 1;
                    selection_changed = true;
                }
            }
            Key::ArrowUp => {
                if selection.index > 0 {
                    selection.index -= 1;
                    selection_changed = true;
                }
            }
            Key::Enter => {
                // Execute selected item
                if let Some(result) = search_results.items.get(selection.index) {
                    execute_app(&result.path);
                }
            }
            _ => {}
        }
        
        // Update visual selection
        if selection_changed {
            for (idx, (mut gradient, _)) in result_items.iter_mut().enumerate() {
                *gradient = if idx == selection.index {
                    // Selected gradient
                    BackgroundGradient::from(LinearGradient::to_bottom(vec![
                        ColorStop::new(Color::srgba(0.0, 0.48, 1.0, 0.60), Val::Percent(0.0)),
                        ColorStop::new(Color::srgba(0.0, 0.38, 0.80, 0.80), Val::Percent(100.0)),
                    ]))
                } else {
                    // Default gradient
                    BackgroundGradient::from(LinearGradient::to_bottom(vec![
                        ColorStop::new(Color::srgba(0.13, 0.13, 0.15, 0.80), Val::Percent(0.0)),
                        ColorStop::new(Color::srgba(0.11, 0.11, 0.13, 0.85), Val::Percent(100.0)),
                    ]))
                };
            }
        }
    }
}

fn execute_app(path: &str) {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(path)
            .spawn()
            .expect("Failed to launch application");
    }
}
```

## System Registration

```rust
// Add to main.rs systems
.add_event::<SearchQueryChanged>()
.add_systems(Startup, discover_applications_system)
.add_systems(Update, (
    search_input_system,
    search_system,
    update_results_system,
    search_navigation_system,
))
```

## Success Criteria

✅ **Real KeyboardInput handling** - exact pattern from text_input.rs  
✅ **Real fuzzy search** - simple character subsequence matching  
✅ **Real result spawning** - dynamic entity creation/destruction  
✅ **Real keyboard navigation** - arrow keys and enter  
✅ **Real app execution** - actual process launching  

**NO BULLSHIT SEARCH COMPONENTS** - Use standard Bevy patterns only