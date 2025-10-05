# About_Menu Task 2: External Navigation System

## Task Overview
Implement secure external URL and action handling system for the About menu, enabling safe navigation to external resources while maintaining application security.

## Implementation Requirements

### Core Components
```rust
// External navigation system
#[derive(Component, Reflect, Debug)]
pub struct ExternalNavigationComponent {
    pub url_whitelist: Vec<String>,
    pub action_handlers: HashMap<String, ActionHandler>,
    pub security_context: SecurityContext,
}

#[derive(Reflect, Debug)]
pub struct ActionHandler {
    pub action_type: ActionType,
    pub validation_rules: Vec<ValidationRule>,
    pub callback: Box<dyn Fn(&ActionContext) -> Result<(), NavigationError>>,
}

#[derive(Reflect, Debug)]
pub enum ActionType {
    OpenUrl,
    OpenFile,
    CopyToClipboard,
    ShowDialog,
}
```

### Security Framework
```rust
// Security validation for external actions
#[derive(Resource, Reflect)]
pub struct NavigationSecurityResource {
    pub allowed_domains: HashSet<String>,
    pub blocked_schemes: HashSet<String>,
    pub sanitization_rules: Vec<SanitizationRule>,
}

pub fn validate_external_navigation(
    url: &str,
    security: &NavigationSecurityResource,
) -> Result<ValidatedUrl, SecurityError> {
    // Implementation with zero allocations
}
```

### Bevy System Integration
```rust
pub fn handle_external_navigation_system(
    mut commands: Commands,
    navigation_query: Query<&ExternalNavigationComponent>,
    input_events: EventReader<NavigationEvent>,
) {
    // System implementation with change detection
}
```

## Bevy Examples Integration

### Related Examples from docs/bevy/examples:
- `ui/button.rs` - UI interaction patterns
- `input/keyboard_input_events.rs` - Event handling
- `async_compute/async_compute.rs` - Async operation patterns

### Implementation Pattern
```rust
// Based on ui/button.rs interaction pattern
fn button_interaction_system(
    mut interaction_query: Query<(&Interaction, &mut BackgroundColor, &NavigationAction)>,
    mut navigation_events: EventWriter<ExternalNavigationEvent>,
) {
    for (interaction, mut color, nav_action) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                navigation_events.send(ExternalNavigationEvent::new(nav_action.clone()));
            }
            // Handle other states
        }
    }
}
```

## Security Requirements
- URL whitelist validation before any external navigation
- Scheme validation (https, file, etc.)
- Input sanitization for all external parameters
- Security context tracking for audit logging

## Performance Constraints
- **ZERO ALLOCATIONS** during navigation validation
- Pre-compiled regex patterns for URL validation
- Cached security contexts to avoid repeated validation
- Async navigation to prevent UI blocking

## Success Criteria
- All external links open safely without security warnings
- No unwrap()/expect() calls in production code
- Zero-allocation navigation validation
- Complete audit trail for all external actions
- Graceful handling of navigation failures

## DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA
Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

## Testing Requirements
- Unit tests for URL validation logic
- Integration tests for security context handling
- Performance tests for zero-allocation validation
- Security tests for malicious URL handling