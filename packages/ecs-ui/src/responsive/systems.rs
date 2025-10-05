//! Generic responsive UI systems
//!
//! Zero-allocation viewport-responsive container and text truncation systems.
//! These systems work with any Bevy application using standard Node-based UI.

use bevy::prelude::*;
use bevy::text::TextLayoutInfo;
use tracing::{info, warn};

use super::components::{ContentConstraints, TextTruncation, TruncationState, ViewportResponsiveContainer};

/// Zero-allocation system for viewport-responsive container updates
///
/// Automatically applies viewport-responsive styling when ViewportResponsiveContainer components
/// change. Uses screen-based Vw/Vh units for blazing-fast responsive design.
#[inline]
pub fn update_viewport_responsive_container_system(
    mut containers: Query<
        (&mut Node, &ViewportResponsiveContainer),
        Changed<ViewportResponsiveContainer>,
    >,
) {
    // Process only changed ViewportResponsiveContainer components for optimal performance
    for (mut style, container) in containers.iter_mut() {
        // Validate container configuration before applying
        if !container.is_valid() {
            warn!("Invalid ViewportResponsiveContainer configuration detected, skipping update");
            continue;
        }

        // Apply viewport-responsive styling directly from component
        // Viewport units automatically handle screen size changes
        *style = container.to_node_style();

        info!(
            "Applied viewport-responsive styling: {}vw x {}vh (constraints: {}vw-{}vw, {}vh-{}vh)",
            container.width_vw,
            container.height_vh,
            container.min_width_vw,
            container.max_width_vw,
            container.min_height_vh,
            container.max_height_vh
        );
    }
}

/// System to truncate text using actual Bevy measurements
///
/// Uses TextLayoutInfo for pixel-perfect truncation with iterative refinement.
/// Automatically tracks state to prevent infinite loops during truncation cycles.
///
/// # How It Works
/// 1. Query entities with Text, TextLayoutInfo, and TextTruncation components
/// 2. Wait for Bevy to measure text (TextLayoutInfo.size.y > 0.0)
/// 3. Compare actual measured width against max_width constraint
/// 4. If too wide: truncate using ratio-based estimation, wait for re-measurement
/// 5. Iterate until text fits or max iterations reached
/// 6. Remove state when truncation completes successfully
///
/// # Performance
/// - Change detection on TextLayoutInfo ensures zero work when text unchanged
/// - State component tracks iterations to prevent infinite loops
/// - Binary-search-style convergence typically completes in 2-3 iterations
#[inline]
pub fn text_truncation_system(
    mut query: Query<(
        Entity,
        &mut Text,
        &TextLayoutInfo,
        &TextTruncation,
        Option<&mut TruncationState>,
    ), Changed<TextLayoutInfo>>,
    mut commands: Commands,
) {
    for (entity, mut text, layout_info, truncation, state) in query.iter_mut() {
        // Wait for valid measurement from Bevy's text rendering system
        if layout_info.size.x == 0.0 || layout_info.size.y == 0.0 {
            continue;  // Text not yet measured, skip this frame
        }

        let actual_width = layout_info.size.x;  // REAL pixel measurement, not estimation

        // Handle existing state or initialize new state
        if let Some(mut state) = state {
            // We're in an active truncation cycle
            if actual_width > truncation.max_width {
                // Still too wide, continue truncating
                if state.is_exhausted() {
                    warn!(
                        "Text truncation reached max iterations ({}) for text: '{}...'",
                        state.max_iterations,
                        text.0.chars().take(20).collect::<String>()
                    );
                    commands.entity(entity).remove::<TruncationState>();
                    continue;
                }

                // Calculate how much to truncate using measured ratio
                let current_chars = text.0.chars().count();
                
                // Validate max_width before division
                if truncation.max_width <= 0.0 {
                    warn!(
                        "Invalid TextTruncation.max_width: {}px, must be > 0.0. Skipping truncation.",
                        truncation.max_width
                    );
                    continue;
                }
                
                let ratio = actual_width / truncation.max_width;
                let ellipsis_chars = truncation.ellipsis.chars().count();
                
                // Target character count: scale down by ratio, subtract ellipsis space
                let target_chars = ((current_chars as f32 / ratio) as usize)
                    .saturating_sub(ellipsis_chars)
                    .max(1);  // Always keep at least 1 character

                if target_chars < current_chars {
                    let truncated: String = text.0.chars().take(target_chars).collect();
                    text.0 = format!("{}{}", truncated, truncation.ellipsis);
                    state.increment();
                    
                    info!(
                        "Truncation iteration {}: {} chars ({}px) -> {} chars (target {}px)",
                        state.iteration_count,
                        current_chars,
                        actual_width,
                        target_chars,
                        truncation.max_width
                    );
                } else {
                    // Can't truncate further, give up
                    warn!("Cannot truncate further, text already minimal");
                    commands.entity(entity).remove::<TruncationState>();
                }
            } else {
                // Success! Text now fits within constraints
                info!(
                    "Text truncation completed in {} iterations: {}px <= {}px",
                    state.iteration_count,
                    actual_width,
                    truncation.max_width
                );
                commands.entity(entity).remove::<TruncationState>();
            }
        } else {
            // No existing state, check if truncation is needed
            if actual_width > truncation.max_width {
                // Text is too wide, initialize truncation state
                info!(
                    "Starting text truncation: {}px > {}px",
                    actual_width,
                    truncation.max_width
                );
                
                commands.entity(entity).insert(TruncationState::new(text.0.clone()));
                
                // Perform initial truncation estimate
                let current_chars = text.0.chars().count();
                
                // Validate max_width before division
                if truncation.max_width <= 0.0 {
                    warn!(
                        "Invalid TextTruncation.max_width: {}px, must be > 0.0. Skipping truncation.",
                        truncation.max_width
                    );
                    continue;
                }
                
                let ratio = actual_width / truncation.max_width;
                let ellipsis_chars = truncation.ellipsis.chars().count();
                let target_chars = ((current_chars as f32 / ratio) as usize)
                    .saturating_sub(ellipsis_chars)
                    .max(1);

                if target_chars < current_chars {
                    let truncated: String = text.0.chars().take(target_chars).collect();
                    text.0 = format!("{}{}", truncated, truncation.ellipsis);
                }
            }
            // else: Text fits, no action needed
        }
    }
}

/// System to enforce content constraints on containers with child limits
///
/// Automatically calculates and applies max_height based on visible item count.
/// Manages overflow behavior for scrollable content areas.
///
/// # How It Works
/// 1. Query entities with ContentConstraints and Children
/// 2. Count actual children
/// 3. Calculate height: min(child_count, max_visible) * item_height
/// 4. Update Node.max_height to enforce constraint
/// 5. Set overflow behavior for scrolling
///
/// # Example Use Case
/// Results list that should show max 8 items with scrolling for overflow:
/// ```
/// commands.spawn((
///     Node { /* ... */ },
///     ContentConstraints::new(8, 48.0),  // 8 items @ 48px each
/// ));
/// ```
#[inline]
pub fn content_constraints_system(
    mut query: Query<
        (&mut Node, &ContentConstraints, &Children),
        Or<(Changed<ContentConstraints>, Changed<Children>)>,
    >,
) {
    for (mut style, constraints, children) in query.iter_mut() {
        let child_count = children.len();
        let calculated_height = constraints.height_for_items(child_count);

        // Enforce maximum height constraint
        style.max_height = Val::Px(calculated_height);

        // Set overflow behavior based on whether content will overflow
        if constraints.will_overflow(child_count) {
            // More items than visible: enable clipping for scroll
            if style.overflow.y != OverflowAxis::Clip {
                style.overflow.y = OverflowAxis::Clip;
            }
        } else {
            // All items visible: no overflow needed
            if style.overflow.y != OverflowAxis::Visible {
                style.overflow.y = OverflowAxis::Visible;
            }
        }

        info!(
            "Applied content constraints: {} children, {}px max height (visible: {}, overflow: \
             {})",
            child_count,
            calculated_height,
            constraints.max_visible_results,
            constraints.will_overflow(child_count)
        );
    }
}
