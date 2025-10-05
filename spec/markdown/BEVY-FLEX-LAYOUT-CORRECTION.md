# CORRECTED: Bevy Flex-Based Layout for Action Items Launcher

## The Problem (Correctly Identified)

The launcher is expanding because of **incorrect flex properties**, not because we need fixed pixel sizes. Bevy's UI system is built on flexbox, and the solution is to properly constrain flex behavior.

## The Correct Bevy Solution

### 1. Container Layout (Proper Flex Constraints)

```rust
// CORRECT: Use flex properties to prevent expansion
commands.spawn((
    Node {
        // Responsive width with constraints
        width: Val::Percent(100.0),           // Fill available width
        max_width: Val::Vw(60.0),             // Never exceed 60% viewport width
        min_width: Val::Vw(35.0),             // Never shrink below 35% viewport width
        
        // Key: Prevent flex expansion
        flex_grow: 0.0,                       // Don't grow beyond natural size
        flex_shrink: 1.0,                     // Allow shrinking if needed
        flex_basis: Val::Auto,                // Base size from content
        
        // Proper alignment
        align_self: AlignSelf::Center,        // Center in parent
        justify_content: JustifyContent::FlexStart,
        flex_direction: FlexDirection::Column,
        
        // Content overflow handling
        overflow: Overflow::clip_y(),         // Clip vertical overflow
        
        // Spacing
        padding: UiRect::all(Val::Px(16.0)),
        row_gap: Val::Px(8.0),
        ..default()
    },
    BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.98)),
));
```

### 2. Search Bar (Flex with Fixed Height Constraint)

```rust
// CORRECT: Constrained height with flexible width
commands.spawn((
    Node {
        // Flexible width within parent constraints
        width: Val::Percent(100.0),
        height: Val::Px(40.0),               // Fixed height constraint
        max_height: Val::Px(40.0),           // Enforce height limit
        
        // Prevent vertical expansion
        flex_grow: 0.0,
        flex_shrink: 0.0,                    // Don't shrink height
        
        // Content layout
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::FlexStart,
        
        // Padding for internal spacing
        padding: UiRect {
            left: Val::Px(12.0),
            right: Val::Px(12.0),
            top: Val::Px(8.0),
            bottom: Val::Px(8.0),
        },
        ..default()
    },
    SearchBarMarker,
));
```

### 3. Results Container (Scrollable with Max Height)

```rust
// CORRECT: Scrollable results with constrained height
commands.spawn((
    Node {
        width: Val::Percent(100.0),
        max_height: Val::Px(400.0),          // Maximum height for results
        min_height: Val::Px(0.0),            // Can collapse when empty
        
        // Allow vertical growth up to max
        flex_grow: 1.0,                      // Grow to show results
        flex_shrink: 1.0,                    // Shrink when fewer results
        
        // Scroll behavior
        overflow: Overflow::clip_y(),        // Enable vertical scrolling
        flex_direction: FlexDirection::Column,
        
        ..default()
    },
    ScrollingList {
        position: 0.0,
        sensitivity: 1.0,
    },
    ResultsContainer,
));
```