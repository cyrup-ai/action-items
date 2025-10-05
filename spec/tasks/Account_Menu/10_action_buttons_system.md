# Task 10: Bottom Action Buttons System Implementation

## Objective
Implement the bottom action panel with Log Out and Manage Subscription buttons, including secure logout functionality, subscription management integration, and proper button styling.

## Implementation Details

### Target Files
- `ui/src/ui/components/account/action_buttons.rs:1-200` - Bottom action buttons component
- `core/src/auth/logout.rs:1-150` - Secure logout functionality
- `core/src/subscription/management_ui.rs:1-120` - Subscription management integration
- `ui/src/ui/systems/account_actions.rs:1-180` - Action button event handling

### Bevy Implementation Patterns

#### Bottom Action Panel Layout
**Reference**: `./docs/bevy/examples/ui/flex_layout.rs:250-280` - Bottom-aligned button layout
**Reference**: `./docs/bevy/examples/ui/ui.rs:750-780` - Two-button horizontal layout with spacing
```rust
// Bottom action panel container
NodeBundle {
    style: Style {
        position_type: PositionType::Absolute,
        position: UiRect {
            left: Val::Px(0.0),
            right: Val::Px(0.0),
            bottom: Val::Px(0.0),
            ..default()
        },
        flex_direction: FlexDirection::Row,
        justify_content: JustifyContent::SpaceBetween,
        align_items: AlignItems::Center,
        width: Val::Percent(100.0),
        height: Val::Px(80.0),
        padding: UiRect::all(Val::Px(24.0)),
        ..default()
    },
    background_color: Color::rgba(0.1, 0.1, 0.1, 0.95).into(), // Semi-transparent dark background
    ..default()
}
```

#### Log Out Button (Destructive Action)
**Reference**: `./docs/bevy/examples/ui/button.rs:200-230` - Destructive button styling
**Reference**: `./docs/bevy/examples/input/mouse_input.rs:120-150` - Button interaction and confirmation
```rust
// Log Out button with destructive styling
ButtonBundle {
    style: Style {
        width: Val::Px(100.0),
        height: Val::Px(40.0),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        border: UiRect::all(Val::Px(1.0)),
        ..default()
    },
    background_color: Color::rgb(0.8, 0.2, 0.2).into(), // Red destructive color
    border_color: Color::rgb(0.9, 0.3, 0.3).into(),
    border_radius: BorderRadius::all(Val::Px(6.0)),
    ..default()
}

// Log Out button text
TextBundle::from_section(
    "Log Out",
    TextStyle {
        font: font_medium.clone(),
        font_size: 14.0,
        color: Color::WHITE,
    },
)

// Log Out button component
#[derive(Component)]
pub struct LogOutButton {
    pub requires_confirmation: bool,
}
```

#### Manage Subscription Button (Primary Action)
**Reference**: `./docs/bevy/examples/ui/button.rs:260-290` - Primary button styling
```rust
// Manage Subscription button with primary styling
ButtonBundle {
    style: Style {
        width: Val::Px(160.0),
        height: Val::Px(40.0),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        border: UiRect::all(Val::Px(1.0)),
        ..default()
    },
    background_color: Color::rgb(0.2, 0.4, 0.8).into(), // Blue primary color
    border_color: Color::rgb(0.3, 0.5, 0.9).into(),
    border_radius: BorderRadius::all(Val::Px(6.0)),
    ..default()
}

// Manage Subscription button text
TextBundle::from_section(
    "Manage Subscription",
    TextStyle {
        font: font_medium.clone(),
        font_size: 14.0,
        color: Color::WHITE,
    },
)

// Manage Subscription button component
#[derive(Component)]
pub struct ManageSubscriptionButton;
```

#### Button Interaction System
**Reference**: `./docs/bevy/examples/ui/button.rs:320-360` - Button interaction states and visual feedback
```rust
// Button interaction system for account actions
fn account_action_button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, Option<&LogOutButton>, Option<&ManageSubscriptionButton>),
        (Changed<Interaction>, With<Button>),
    >,
    mut logout_events: EventWriter<LogoutRequestEvent>,
    mut subscription_events: EventWriter<ManageSubscriptionEvent>,
) {
    for (interaction, mut color, logout_btn, subscription_btn) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Clicked => {
                if let Some(logout_button) = logout_btn {
                    if logout_button.requires_confirmation {
                        logout_events.send(LogoutRequestEvent::ConfirmationRequired);
                    } else {
                        logout_events.send(LogoutRequestEvent::Immediate);
                    }
                } else if subscription_btn.is_some() {
                    subscription_events.send(ManageSubscriptionEvent::OpenManagement);
                }
            }
            Interaction::Hovered => {
                if logout_btn.is_some() {
                    *color = Color::rgb(0.9, 0.3, 0.3).into(); // Lighter red on hover
                } else if subscription_btn.is_some() {
                    *color = Color::rgb(0.3, 0.5, 0.9).into(); // Lighter blue on hover
                }
            }
            Interaction::None => {
                if logout_btn.is_some() {
                    *color = Color::rgb(0.8, 0.2, 0.2).into(); // Default red
                } else if subscription_btn.is_some() {
                    *color = Color::rgb(0.2, 0.4, 0.8).into(); // Default blue
                }
            }
        }
    }
}
```

### Secure Logout System

#### Logout Confirmation Modal
**Reference**: `./docs/bevy/examples/ui/ui.rs:800-850` - Modal dialog implementation
```rust
// Logout confirmation modal
#[derive(Component)]
pub struct LogoutConfirmationModal;

fn spawn_logout_confirmation_modal(
    commands: &mut Commands,
    asset_server: &AssetServer,
) -> Entity {
    commands.spawn(NodeBundle {
        style: Style {
            position_type: PositionType::Absolute,
            position: UiRect::all(Val::Px(0.0)),
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        background_color: Color::rgba(0.0, 0.0, 0.0, 0.7).into(), // Modal backdrop
        ..default()
    })
    .insert(LogoutConfirmationModal)
    .with_children(|parent| {
        parent.spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                width: Val::Px(400.0),
                padding: UiRect::all(Val::Px(24.0)),
                gap: Size::all(Val::Px(16.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            background_color: Color::rgb(0.15, 0.15, 0.15).into(),
            border_radius: BorderRadius::all(Val::Px(8.0)),
            ..default()
        })
        .with_children(|modal| {
            // Modal title
            modal.spawn(TextBundle::from_section(
                "Confirm Logout",
                TextStyle {
                    font: font_bold.clone(),
                    font_size: 18.0,
                    color: Color::WHITE,
                },
            ));
            
            // Modal message
            modal.spawn(TextBundle::from_section(
                "Are you sure you want to log out? You'll need to sign in again to access your account.",
                TextStyle {
                    font: font_regular.clone(),
                    font_size: 14.0,
                    color: Color::rgba(0.8, 0.8, 0.8, 1.0),
                },
            ).with_text_alignment(TextAlignment::Center));
            
            // Button container
            modal.spawn(NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Row,
                    gap: Size::all(Val::Px(12.0)),
                    ..default()
                },
                ..default()
            })
            .with_children(|buttons| {
                // Cancel button
                buttons.spawn(ButtonBundle {
                    style: Style {
                        width: Val::Px(100.0),
                        height: Val::Px(36.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    background_color: Color::rgba(0.3, 0.3, 0.3, 1.0).into(),
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                    ..default()
                })
                .insert(CancelLogoutButton);
                
                // Confirm logout button
                buttons.spawn(ButtonBundle {
                    style: Style {
                        width: Val::Px(100.0),
                        height: Val::Px(36.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    background_color: Color::rgb(0.8, 0.2, 0.2).into(),
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                    ..default()
                })
                .insert(ConfirmLogoutButton);
            });
        });
    }).id()
}
```

#### Logout Process Implementation
**Reference**: `./docs/bevy/examples/async_tasks/async_compute.rs:280-320` - Secure session cleanup
```rust
// Secure logout implementation
pub struct LogoutManager {
    auth_service: AuthService,
    session_manager: SessionManager,
}

impl LogoutManager {
    pub async fn perform_logout(&self, user_id: &str) -> Result<(), LogoutError> {
        // Clear local session data
        self.session_manager.clear_session().await?;
        
        // Invalidate server-side session
        self.auth_service.invalidate_session(user_id).await?;
        
        // Clear cached user data
        self.clear_user_cache().await?;
        
        // Clear stored credentials
        self.clear_stored_credentials().await?;
        
        Ok(())
    }
    
    async fn clear_user_cache(&self) -> Result<(), LogoutError> {
        // Clear user profile cache
        // Clear subscription cache
        // Clear preferences cache
        // Clear temporary data
        Ok(())
    }
    
    async fn clear_stored_credentials(&self) -> Result<(), LogoutError> {
        // Remove stored tokens
        // Clear keychain entries
        // Remove cached authentication data
        Ok(())
    }
}

// Logout event handling system
fn logout_event_system(
    mut logout_events: EventReader<LogoutRequestEvent>,
    mut commands: Commands,
    auth_state: ResMut<AuthenticationState>,
    logout_manager: Res<LogoutManager>,
) {
    for event in logout_events.iter() {
        match event {
            LogoutRequestEvent::ConfirmationRequired => {
                // Show confirmation modal
                spawn_logout_confirmation_modal(&mut commands, &asset_server);
            }
            LogoutRequestEvent::Immediate | LogoutRequestEvent::Confirmed => {
                // Perform logout
                commands.spawn_task(async move {
                    if let Err(e) = logout_manager.perform_logout(&auth_state.user_id).await {
                        error!("Logout failed: {}", e);
                    }
                });
            }
            LogoutRequestEvent::Cancelled => {
                // Dismiss confirmation modal
                commands.despawn_modal::<LogoutConfirmationModal>();
            }
        }
    }
}
```

### Subscription Management Integration

#### External Subscription Management
**Reference**: `./docs/bevy/examples/app/return_after_run.rs:80-110` - External application integration
```rust
// Subscription management integration
fn subscription_management_system(
    mut subscription_events: EventReader<ManageSubscriptionEvent>,
    subscription_state: Res<SubscriptionState>,
    mut commands: Commands,
) {
    for event in subscription_events.iter() {
        match event {
            ManageSubscriptionEvent::OpenManagement => {
                let management_url = generate_subscription_management_url(&subscription_state);
                
                commands.spawn_task(async move {
                    if let Err(e) = open_subscription_management(management_url).await {
                        error!("Failed to open subscription management: {}", e);
                    }
                });
            }
        }
    }
}

async fn open_subscription_management(url: String) -> Result<(), SubscriptionError> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&url)
            .spawn()?;
    }
    
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(&["/C", "start", &url])
            .spawn()?;
    }
    
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&url)
            .spawn()?;
    }
    
    Ok(())
}

fn generate_subscription_management_url(subscription_state: &SubscriptionState) -> String {
    format!(
        "https://billing.cyrup.ai/manage?user_id={}&session_token={}",
        subscription_state.user_id,
        subscription_state.billing_session_token
    )
}
```

### Event System Definitions

#### Account Action Events
**Reference**: `./docs/bevy/examples/ecs/event.rs:80-110` - Account action event definitions
```rust
// Account action events
#[derive(Event)]
pub enum LogoutRequestEvent {
    ConfirmationRequired,
    Immediate,
    Confirmed,
    Cancelled,
}

#[derive(Event)]
pub enum ManageSubscriptionEvent {
    OpenManagement,
    UpdateSubscription { plan: String },
    CancelSubscription,
}

// Button component definitions
#[derive(Component)]
pub struct CancelLogoutButton;

#[derive(Component)]
pub struct ConfirmLogoutButton;
```

### Architecture Notes

#### Component Structure
- **ActionButtonsContainer**: Bottom panel container for action buttons
- **LogOutButton**: Component for logout functionality
- **ManageSubscriptionButton**: Component for subscription management
- **LogoutConfirmationModal**: Modal dialog for logout confirmation

#### Security Considerations
- **Complete Session Cleanup**: Comprehensive clearing of all user data
- **Server-side Invalidation**: Proper server session invalidation
- **Secure Token Handling**: Safe removal of authentication tokens
- **Error Handling**: Graceful error handling for logout failures

#### User Experience Features
- **Confirmation Modal**: Optional confirmation for logout action
- **Visual Feedback**: Clear button states and hover effects
- **External Integration**: Seamless subscription management opening
- **Loading States**: Visual feedback during logout and subscription operations

### Quality Standards
- Secure logout with complete data cleanup
- Cross-platform subscription management URL opening
- Accessible button navigation with keyboard support
- Consistent visual styling with destructive vs primary actions
- Robust error handling for all external integrations

### Integration Points
- Authentication system integration for secure logout
- Subscription billing system integration for management URL
- Modal system integration for confirmation dialogs
- External browser integration for subscription management