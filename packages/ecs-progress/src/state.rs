use bevy::prelude::*;
use bevy::state::state::FreelyMutableState;

use crate::prelude::*;

/// System to clear progress data - used for OnEnter/OnExit state transitions  
pub fn clear_progress<S: FreelyMutableState>(
    mut monitor: ResMut<ProgressMonitor<S>>,
) {
    monitor.reset();

    #[cfg(feature = "debug")]
    debug!("Progress data cleared for state transition");
}

/// System to emit ProgressUpdate events for monitoring
pub fn emit_progress_update<S: FreelyMutableState>(
    monitor: Res<ProgressMonitor<S>>,
    state: Res<State<S>>,
    mut update_writer: EventWriter<ProgressUpdate<S>>,
) {
    let visible = monitor.get_total_visible();
    let hidden = monitor.get_total_hidden();
    let is_complete = monitor.is_complete();

    update_writer.write(ProgressUpdate {
        visible,
        hidden,
        is_complete,
        state: state.get().clone(),
    });
}

#[cfg(test)]
mod tests {
    use bevy::app::{App, Update};

    use super::*;

    #[derive(States, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
    enum TestState {
        #[default]
        Loading,
    }

    #[test]
    fn test_clear_progress() {
        let mut app = App::new();
        app.init_resource::<ProgressMonitor<TestState>>();

        // Add some progress
        {
            let mut monitor =
                app.world_mut().resource_mut::<ProgressMonitor<TestState>>();
            monitor.update_visible(EntryId::new(), Progress {
                done: 5,
                total: 10,
            });
        }

        // Clear it
        app.add_systems(Update, clear_progress::<TestState>);
        app.update();

        // Verify it's cleared
        let monitor = app.world().resource::<ProgressMonitor<TestState>>();
        assert_eq!(monitor.get_total_visible(), Progress::default());
    }
}
