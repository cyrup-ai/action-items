# Task 2: Interactive Button Group System Implementation

## Objective
Implement the three-button horizontal action group (Acknowledgements, Visit Website, Send Feedback) with proper spacing, styling, and external integration capabilities.

## Implementation Details

### Target Files
- `ui/src/ui/components/about_buttons.rs:1-150` - Button group component implementation
- `ui/src/ui/interactions/external_links.rs:1-80` - External URL and email handling system
- `core/src/external_integration.rs:1-120` - Browser and email client integration
- `ui/src/ui/components/acknowledgements_modal.rs:1-200` - Acknowledgements display system

### Bevy Implementation Patterns

#### Horizontal Button Layout
**Reference**: `./docs/bevy/examples/ui/button.rs:25-50` - Button styling and interaction setup
**Reference**: `./docs/bevy/examples/ui/ui.rs:85-110` - Horizontal container with even spacing
```rust
// Horizontal button container with centered alignment
NodeBundle {
    style: Style {
        flex_direction: FlexDirection::Row,
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        width: Val::Percent(100.0),
        gap: Size::all(Val::Px(16.0)), // Even spacing between buttons
        margin: UiRect::top(Val::Px(40.0)),
        ..default()
    },
    ..default()
}
```

#### Button Styling and States
**Reference**: `./docs/bevy/examples/ui/button.rs:60-90` - Button interaction states and visual feedback
**Reference**: `./docs/bevy/examples/ui/ui.rs:200-230` - Consistent button sizing and appearance
```rust
// Individual button with secondary styling
ButtonBundle {
    style: Style {
        width: Val::Px(140.0),
        height: Val::Px(36.0),
        border: UiRect::all(Val::Px(1.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    },
    background_color: theme.secondary_button_color.into(),
    border_color: theme.secondary_border_color.into(),
    ..default()
}
```

#### Button Interaction System
**Reference**: `./docs/bevy/examples/ui/button.rs:120-150` - Button click detection and state management
**Reference**: `./docs/bevy/examples/input/keyboard_input_events.rs:40-65` - Input event handling patterns
```rust
// Button interaction system
fn button_interaction_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &ActionButton),
        (Changed<Interaction>, With<Button>),
    >,
    mut external_events: EventWriter<ExternalActionEvent>,
) {
    for (interaction, mut color, action_button) in &mut interaction_query {
        match *interaction {
            Interaction::Clicked => {
                external_events.send(ExternalActionEvent::from(action_button.action));
            }
            Interaction::Hovered => {
                *color = theme.button_hover_color.into();
            }
            Interaction::None => {
                *color = theme.secondary_button_color.into();
            }
        }
    }
}
```

### External Integration System

#### Browser URL Opening
**Reference**: `./docs/bevy/examples/app/return_after_run.rs:30-55` - External system integration patterns
**Reference**: `./docs/bevy/examples/async_tasks/async_compute.rs:85-110` - Asynchronous external operations
```rust
// URL opening system with error handling
fn handle_external_url_system(
    mut external_events: EventReader<ExternalActionEvent>,
    mut commands: Commands,
) {
    for event in external_events.iter() {
        match event {
            ExternalActionEvent::VisitWebsite => {
                commands.spawn_task(async {
                    if let Err(e) = open_url("https://cyrup.ai").await {
                        error!("Failed to open website: {}", e);
                    }
                });
            }
            _ => {}
        }
    }
}
```

#### Email Client Integration
**Reference**: `./docs/bevy/examples/async_tasks/async_compute.rs:40-70` - Async task spawning and error handling
```rust
// Email client integration with fallback handling
async fn open_feedback_email() -> Result<(), ExternalError> {
    let mailto_url = "mailto:feedback@cyrup.ai?subject=Action%20Items%20Feedback";
    
    #[cfg(target_os = "macos")]
    {
        Command::new("open").arg(mailto_url).spawn()?;
    }
    
    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .args(&["/C", "start", mailto_url])
            .spawn()?;
    }
    
    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open").arg(mailto_url).spawn()?;
    }
    
    Ok(())
}
```

### Acknowledgements Modal System

#### Modal Container Layout
**Reference**: `./docs/bevy/examples/ui/ui.rs:300-340` - Modal overlay and backdrop implementation
**Reference**: `./docs/bevy/examples/ui/scroll.rs:45-80` - Scrollable content area for acknowledgements list
```rust
// Modal overlay with backdrop
NodeBundle {
    style: Style {
        position_type: PositionType::Absolute,
        position: UiRect::all(Val::Px(0.0)),
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    },
    background_color: Color::rgba(0.0, 0.0, 0.0, 0.7).into(),
    ..default()
}
```

#### Scrollable Acknowledgements List
**Reference**: `./docs/bevy/examples/ui/scroll.rs:15-40` - Scrollable container implementation
**Reference**: `./docs/bevy/examples/ui/text.rs:150-180` - Dynamic text list generation
```rust
// Scrollable acknowledgements content
ScrollBundle {
    style: Style {
        width: Val::Px(600.0),
        height: Val::Px(400.0),
        flex_direction: FlexDirection::Column,
        overflow: Overflow::clip_y(),
        ..default()
    },
    scroll_position: ScrollPosition::default(),
    ..default()
}
```

### Architecture Notes

#### Component Structure
- **ActionButton**: Component marking button type (Acknowledgements, Website, Feedback)
- **ExternalActionEvent**: Event for triggering external integrations
- **AcknowledgementsModal**: Component for modal state management
- **ExternalIntegrationResource**: Resource for tracking external operation status

#### Button Actions
- **Acknowledgements**: Opens modal with third-party libraries and credits
- **Visit Website**: Opens primary website URL in default browser
- **Send Feedback**: Opens email client with pre-filled feedback email

#### Error Handling Strategy
**Reference**: `./docs/bevy/examples/diagnostics/log_diagnostics.rs:20-45` - Error logging and user feedback patterns
- Graceful fallback for missing browser or email client
- User notification system for failed external operations
- Fallback URL copying to clipboard when browser unavailable

### Quality Standards
- Cross-platform compatibility for URL and email opening
- Proper error handling with user-friendly feedback
- Accessible button navigation with keyboard support
- Consistent visual feedback for all interaction states
- Performance optimization for modal opening/closing animations

### Integration Points
- Theme system integration for button styling
- Modal management system for acknowledgements display
- External system integration for browser and email client
- Keyboard navigation integration for accessibility