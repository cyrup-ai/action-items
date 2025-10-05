//! Test that wizard auto-starts on first run with configured permissions

use bevy::prelude::*;
use action_items_ecs_permissions::{
    PermissionPlugin, PermissionType,
    wizard::{PermissionWizardPlugin, WizardState, FirstRunDetector},
};

fn main() {
    let mut app = App::new();
    
    app.add_plugins(MinimalPlugins)
        .add_plugins(bevy::state::app::StatesPlugin)
        .add_plugins(PermissionPlugin)
        .add_plugins(
            PermissionWizardPlugin::default()
                .with_required_permissions(vec![
                    PermissionType::Accessibility,
                    PermissionType::FullDiskAccess,
                    PermissionType::Camera,
                    PermissionType::Microphone,
                ])
                .with_reason("Test app requires these permissions")
        )
        .add_systems(Startup, setup_test)
        .add_systems(Update, check_wizard_state);
    
    println!("🚀 Starting wizard auto-start test...");
    
    // Run a few frames
    for i in 0..10 {
        app.update();
        println!("  Frame {}", i);
    }
    
    println!("✅ Test complete - check output above");
}

fn setup_test(mut detector: ResMut<FirstRunDetector>) {
    // Simulate first run
    detector.is_first_run = true;
    detector.wizard_completed = false;
    detector.check_completed = false;
    println!("📝 Configured first-run detector");
}

fn check_wizard_state(
    wizard_state: Res<State<WizardState>>,
    detector: Res<FirstRunDetector>,
) {
    if detector.check_completed {
        let state = wizard_state.get();
        if state.is_active() {
            println!("✅ WIZARD IS ACTIVE: {:?}", state);
        } else if *state == WizardState::NotStarted {
            println!("⚠️  Wizard still in NotStarted state");
        } else {
            println!("ℹ️  Wizard state: {:?}", state);
        }
    }
}
