//! Test to prove privacy indicators implementation works correctly

use action_items_ui::{PrivacyConfiguration, PrivacyIndicatorPlugin, PrivacyIndicators};
use bevy::prelude::*;

#[test]
fn test_privacy_indicators_creation() {
    // Test privacy indicators component creation
    let indicators = PrivacyIndicators::new(true, false, true);

    assert!(indicators.full_control);
    assert!(!indicators.no_collection);
    assert!(indicators.encrypted);
    assert!(!indicators.info_expanded);
}

#[test]
fn test_privacy_configuration_defaults() {
    // Test privacy configuration with secure defaults
    let config = PrivacyConfiguration::secure_default();

    assert!(!config.data_collection_enabled);
    assert!(config.full_user_control);
    assert!(config.encryption_active);
}

#[test]
fn test_privacy_configuration_indicator_calculation() {
    // Test privacy indicator calculation
    let config = PrivacyConfiguration::secure_default();
    let (full_control, no_collection, encrypted) = config.calculate_indicators();

    assert!(full_control);
    assert!(no_collection);
    assert!(encrypted);
}

#[test]
fn test_privacy_indicators_state_updates() {
    // Test privacy indicators state update detection
    let mut indicators = PrivacyIndicators::default();

    // First update should return true (changed)
    let changed1 = indicators.update_states(true, false, true);
    assert!(changed1);

    // Second update with same values should return false (no change)
    let changed2 = indicators.update_states(true, false, true);
    assert!(!changed2);

    // Third update with different values should return true (changed)
    let changed3 = indicators.update_states(false, true, false);
    assert!(changed3);
}

#[test]
fn test_privacy_plugin_integration() {
    // Test privacy plugin can be added to Bevy app without panicking
    let mut app = App::new();

    // Add minimal plugins for testing
    app.add_plugins((bevy::time::TimePlugin, PrivacyIndicatorPlugin));

    // Run startup systems to initialize resources
    app.update();

    // Verify privacy configuration is initialized after startup
    assert!(app.world().contains_resource::<PrivacyConfiguration>());
}
