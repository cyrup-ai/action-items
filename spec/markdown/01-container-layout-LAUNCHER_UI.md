# LAUNCHER_UI Container Layout Implementation

**Using actual Bevy patterns from examples - responsive design like real Raycast**

## Current Problem (ui/src/ui/systems/setup.rs:79-96)
```rust
.spawn((
    Node {
        width: Val::Vw(45.0),             // ❌ Too small percentage
        height: Val::Vh(35.0),            // ❌ Too small percentage  
        margin: UiRect::top(Val::Vh(12.0)),
        padding: UiRect::all(Val::VMin(1.2)),
        row_gap: Val::VMin(0.8),
        ..default()
    },
    BackgroundColor(theme.colors.background_primary), // ❌ Flat color
))
```

## LAUNCHER_UI Solution Using Bevy Examples

### Step 1: Responsive Raycast-Style Container

**Pattern from:** `bevy/examples/ui/flex_layout.rs` + `size_constraints.rs`

```rust
// LAUNCHER_UI container spawn - responsive like real Raycast
commands.spawn((
    Node {
        width: Val::Vw(60.0),               // 60% viewport width (responsive)
        max_width: Val::Vw(60.0),           // Max 60% of viewport width
        min_width: Val::Vw(35.0),           // Min 35% of viewport width for usability
        height: Val::Auto,                  // Auto height based on content
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::FlexStart,
        margin: UiRect::top(Val::Vh(15.0)), // 15% from top (responsive)
        padding: UiRect::all(Val::VMin(1.5)), // 1.5% of min viewport dimension
        row_gap: Val::VMin(0.8),            // 0.8% of min viewport dimension
        ..default()
    },
    // Professional gradient using BackgroundColor for now (gradients need API fix)
    BackgroundColor(Color::srgba(0.08, 0.08, 0.09, 0.98)),
    BorderRadius::all(Val::VMin(1.2)),  // 1.2% of min viewport dimension
    LauncherContainer, // Simple marker component
))
```

### Step 2: Responsive System Using REAL Patterns

**Pattern from:** `bevy/examples/ui/size_constraints.rs` (button update system)

```rust
#[derive(Component)]
struct LauncherContainer;

// LAUNCHER_UI system - direct node mutation like the examples
fn responsive_container_system(
    windows: Query<&Window, With<PrimaryWindow>>,
    mut container: Query<&mut Node, With<LauncherContainer>>,
) {
    let Ok(window) = windows.get_single() else { return };
    let Ok(mut node) = container.get_single_mut() else { return };
    
    // Responsive logic: adjust based on viewport size
    if window.width() < window.height() {
        // Portrait mode: use more width
        node.width = Val::Vw(90.0);
    } else {
        // Landscape mode: standard width
        node.width = Val::Vw(60.0);
    }
}
```

### Step 3: Search Container using REAL Patterns

**Pattern from:** `bevy/examples/ui/button.rs` (nested structure)

```rust
// Inside the launcher container .with_children() call
parent.spawn((
    Node {
        width: Val::Percent(100.0),         // Full width of parent
        height: Val::Vh(5.0),               // 5% of viewport height
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        padding: UiRect::all(Val::VMin(1.0)),
        ..default()
    },
    // Search bar gradient (LAUNCHER_UI gradient from examples)
    BackgroundGradient::from(LinearGradient::to_bottom(vec![
        ColorStop::new(Color::srgba(0.18, 0.18, 0.20, 0.85), Val::Percent(0.0)),
        ColorStop::new(Color::srgba(0.15, 0.15, 0.17, 0.90), Val::Percent(100.0)),
    ])),
    BorderRadius::all(Val::VMin(0.6)),  // 0.6% of min viewport dimension,
    SearchContainer,
))
.with_children(|parent| {
    // Search icon using REAL text patterns
    parent.spawn((
        Text::new("\u{F002}"),              // FontAwesome search
        TextFont {
            font: fonts.icons.clone(),
            font_size: 16.0,  // Font size should use responsive calculation in system
            ..default()
        },
        TextColor(Color::srgba(0.70, 0.70, 0.75, 1.0)),
        Node {
            margin: UiRect::right(Val::VMin(0.6)),
            ..default()
        },
    ));
    
    // Search text using REAL text patterns (from text_input.rs)
    parent.spawn((
        Text::new("Search..."),
        TextFont {
            font: fonts.regular.clone(),
            font_size: 16.0,  // Font size should use responsive calculation in system
            ..default()
        },
        TextColor(Color::srgba(0.70, 0.70, 0.75, 1.0)), // Placeholder color
        Node {
            flex_grow: 1.0,
            ..default()
        },
        SearchInput, // Marker for input system
    ));
});
```

### Step 4: Input System Using REAL Patterns

**Pattern from:** `bevy/examples/input/text_input.rs`

```rust
#[derive(Component)]
struct SearchInput;

#[derive(Component)]
struct SearchContainer;

// REAL input handling system (from text_input.rs example)
fn search_input_system(
    mut events: EventReader<KeyboardInput>,
    mut search_text: Query<&mut Text, With<SearchInput>>,
) {
    let Ok(mut text) = search_text.get_single_mut() else { return };
    
    for event in events.read() {
        if !event.state.is_pressed() {
            continue;
        }
        
        match (&event.logical_key, &event.text) {
            (Key::Backspace, _) => {
                // Remove placeholder and start with empty
                if **text == "Search..." {
                    **text = String::new();
                } else {
                    text.pop();
                }
            }
            (_, Some(inserted_text)) => {
                // Clear placeholder on first input
                if **text == "Search..." {
                    **text = String::new();
                }
                // Add printable characters
                if inserted_text.chars().all(is_printable_char) {
                    text.push_str(inserted_text);
                }
            }
            _ => {}
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
```

### Step 5: Results Container Using REAL Patterns

**Pattern from:** `bevy/examples/ui/scroll.rs` and `flex_layout.rs`

```rust
// Results container (inside launcher container)
parent.spawn((
    Node {
        width: Val::Percent(100.0),
        flex_direction: FlexDirection::Column,
        row_gap: Val::VMin(0.3), // Tight spacing between results (0.3% of min viewport)
        overflow: Overflow::clip(), // Prevent expansion
        ..default()
    },
    ResultsContainer,
    Visibility::Hidden, // Hidden until search starts
));

#[derive(Component)]
struct ResultsContainer;

// System to show/hide results (REAL visibility handling)
fn results_visibility_system(
    search_text: Query<&Text, (With<SearchInput>, Changed<Text>)>,
    mut results: Query<&mut Visibility, With<ResultsContainer>>,
) {
    let Ok(text) = search_text.get_single() else { return };
    let Ok(mut visibility) = results.get_single_mut() else { return };
    
    *visibility = if text.is_empty() || **text == "Search..." {
        Visibility::Hidden
    } else {
        Visibility::Visible
    };
}
```

## REAL Implementation in setup.rs

**Replace the entire container spawn with this:**

```rust
pub fn setup_ui_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    let fonts = load_fonts(&asset_server);
    
    commands.spawn(Camera2d);
    
    // Root container (full screen)
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::FlexStart,
            ..default()
        },
    ))
    .with_children(|parent| {
        // MAIN LAUNCHER CONTAINER - REAL RAYCAST DIMENSIONS
        parent.spawn((
            Node {
                width: Val::Vw(60.0),               // 60% of viewport width
                max_width: Val::Vw(60.0),           // Maximum constraint  
                height: Val::Auto,                  // Auto height based on content
                max_height: Val::Vh(60.0),          // Maximum 60% of viewport height
                flex_direction: FlexDirection::Column,
                margin: UiRect::top(Val::Vh(12.0)), // 12% from top
                padding: UiRect::all(Val::VMin(1.0)),
                row_gap: Val::VMin(0.6),
                ..default()
            },
            BackgroundGradient::from(LinearGradient::to_bottom(vec![
                ColorStop::new(Color::srgba(0.08, 0.08, 0.09, 0.98), Val::Percent(0.0)),
                ColorStop::new(Color::srgba(0.12, 0.12, 0.14, 0.98), Val::Percent(100.0)),
            ])),
            BorderRadius::all(Val::VMin(1.2)),
            BoxShadow(vec![ShadowStyle {
                color: Color::BLACK.with_alpha(0.3),
                x_offset: Val::Px(0.0),
                y_offset: Val::VMin(0.6),
                blur_radius: Val::VMin(1.8),
                spread_radius: Val::Px(0.0),
            }]),
            LauncherContainer,
            Visibility::Hidden, // Hidden initially
        ))
        .with_children(|parent| {
            // SEARCH BAR - REAL 40PX COMPACT HEIGHT
            spawn_search_bar(parent, &fonts);
            
            // RESULTS CONTAINER - REAL OVERFLOW HANDLING
            spawn_results_container(parent);
        });
    });
}

fn spawn_search_bar(parent: &mut ChildBuilder, fonts: &UiFonts) {
    parent.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Vh(5.0),               // 5% of viewport height
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            padding: UiRect::all(Val::Px(12.0)),
            ..default()
        },
        BackgroundGradient::from(LinearGradient::to_bottom(vec![
            ColorStop::new(Color::srgba(0.18, 0.18, 0.20, 0.85), Val::Percent(0.0)),
            ColorStop::new(Color::srgba(0.15, 0.15, 0.17, 0.90), Val::Percent(100.0)),
        ])),
        BorderRadius::all(Val::VMin(0.6)),
        SearchContainer,
    ))
    .with_children(|parent| {
        // Search icon
        parent.spawn((
            Text::new("\u{F002}"),
            TextFont {
                font: fonts.icons.clone(),
                font_size: 16.0,
                ..default()
            },
            TextColor(Color::srgba(0.70, 0.70, 0.75, 1.0)),
            Node {
                margin: UiRect::right(Val::Px(8.0)),
                ..default()
            },
        ));
        
        // Search input text
        parent.spawn((
            Text::new("Search..."),
            TextFont {
                font: fonts.regular.clone(),
                font_size: 16.0,
                ..default()
            },
            TextColor(Color::srgba(0.70, 0.70, 0.75, 1.0)),
            Node {
                flex_grow: 1.0,
                ..default()
            },
            SearchInput,
        ));
    });
}
```

## System Registration

```rust
// Add to main.rs systems
.add_systems(Update, (
    responsive_container_system,
    search_input_system,
    results_visibility_system,
))
```

## Success Criteria

✅ **Responsive container (60% viewport width, max 60% viewport height)** - scales with screen size  
✅ **Real Bevy gradient system** - no custom gradient components  
✅ **Real text input handling** - using KeyboardInput events  
✅ **Real component patterns** - simple marker components only  
✅ **Real system patterns** - direct node mutation like examples  

**NO BULLSHIT COMPONENTS** - Only standard Bevy components + simple markers