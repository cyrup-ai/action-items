use bevy::ecs::schedule::{InternedScheduleLabel, ScheduleLabel};
use bevy::prelude::*;
use bevy::state::state::{FreelyMutableState, States};

use crate::prelude::*;

/// Hook trait for reacting to progress events
pub trait ProgressHook<S: States>: Send + Sync + 'static {
    /// Called when progress is updated
    fn on_progress_update(&self, progress: Progress, hidden: HiddenProgress);
    /// Called when progress is complete
    fn on_progress_complete(&self, state: &S);
}

/// Simplified hook that only cares about completion
pub trait CompletionHook<S: States>: Send + Sync + 'static {
    /// Called when all progress is complete
    fn on_complete(&self, state: &S);
}

impl<S: States, T: CompletionHook<S>> ProgressHook<S> for T {
    fn on_progress_update(&self, _progress: Progress, _hidden: HiddenProgress) {
        // Default implementation does nothing for progress updates
    }

    fn on_progress_complete(&self, state: &S) {
        self.on_complete(state);
    }
}

/// Enhanced plugin for progress tracking with events, hooks, and better
/// ergonomics
pub struct ProgressPlugin<S: States> {
    transitions: StateTransitionConfig<S>,
    check_progress_schedule: InternedScheduleLabel,
    autoclear_on_enter: bool,
    autoclear_on_exit: bool,
    hooks: Vec<Box<dyn ProgressHook<S>>>,
    #[cfg(feature = "assets")]
    track_assets: bool,
    #[cfg(feature = "assets")]
    autoclear_assets_on_enter: bool,
    #[cfg(feature = "assets")]
    autoclear_assets_on_exit: bool,
    #[cfg(feature = "debug")]
    enable_debug_logging: bool,
}

/// System set for progress checking and transitions
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub struct ProgressCheckSet;

/// System set for asset progress tracking
#[cfg(feature = "assets")]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
#[allow(dead_code)]
pub struct AssetProgressSet;

impl<S: States> Default for ProgressPlugin<S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S: States> ProgressPlugin<S> {
    /// Create a new progress plugin
    pub fn new() -> Self {
        Self {
            check_progress_schedule: Last.intern(),
            transitions: StateTransitionConfig::default(),
            autoclear_on_enter: true,
            autoclear_on_exit: false,
            hooks: Vec::new(),
            #[cfg(feature = "assets")]
            track_assets: false,
            #[cfg(feature = "assets")]
            autoclear_assets_on_enter: false,
            #[cfg(feature = "assets")]
            autoclear_assets_on_exit: true,
            #[cfg(feature = "debug")]
            enable_debug_logging: false,
        }
    }

    /// Add a state transition (from -> to)
    pub fn with_transition(mut self, from: S, to: S) -> Self {
        self.transitions.add_transition(from, to);
        self
    }

    /// Set the schedule where progress is checked
    pub fn check_in<L: ScheduleLabel>(mut self, schedule: L) -> Self {
        self.check_progress_schedule = schedule.intern();
        self
    }

    /// Configure automatic clearing of progress data
    pub fn auto_clear(mut self, on_enter: bool, on_exit: bool) -> Self {
        self.autoclear_on_enter = on_enter;
        self.autoclear_on_exit = on_exit;
        self
    }

    /// Add a progress hook
    pub fn with_hook<H: ProgressHook<S>>(mut self, hook: H) -> Self {
        self.hooks.push(Box::new(hook));
        self
    }

    /// Add a completion-only hook
    pub fn with_completion_hook<H: CompletionHook<S>>(
        mut self,
        hook: H,
    ) -> Self {
        self.hooks.push(Box::new(hook));
        self
    }

    #[cfg(feature = "assets")]
    /// Enable asset tracking
    pub fn with_assets(mut self) -> Self {
        self.track_assets = true;
        self
    }

    #[cfg(feature = "assets")]
    /// Configure automatic clearing of asset data
    pub fn auto_clear_assets(mut self, on_enter: bool, on_exit: bool) -> Self {
        self.autoclear_assets_on_enter = on_enter;
        self.autoclear_assets_on_exit = on_exit;
        self
    }

    #[cfg(feature = "debug")]
    /// Enable debug logging
    pub fn with_debug_logging(mut self) -> Self {
        self.enable_debug_logging = true;
        self
    }
}

impl<S: States + FreelyMutableState> Plugin for ProgressPlugin<S> {
    fn build(&self, app: &mut App) {
        // Add events
        app.add_event::<Progress>();
        app.add_event::<HiddenProgress>();
        app.add_event::<ProgressComplete<S>>();
        app.add_event::<ProgressUpdate<S>>();

        // Add resources
        app.init_resource::<ProgressMonitor<S>>();
        app.insert_resource(self.transitions.clone());

        // Core progress systems
        app.add_systems(
            self.check_progress_schedule,
            (
                check_progress_completion::<S>,
                handle_progress_complete::<S>,
            )
                .chain()
                .in_set(ProgressCheckSet)
                .run_if(in_tracked_state::<S>),
        );

        app.add_systems(
            PostUpdate,
            (
                crate::entity::sum_entity_progress::<S>,
                crate::system::track_progress_system::<S>,
                crate::system::track_hidden_progress_system::<S>,
            )
                .run_if(in_tracked_state::<S>),
        );

        // State transition systems
        for state in self.transitions.get_tracked_states() {
            if self.autoclear_on_enter {
                app.add_systems(
                    OnEnter(state.clone()),
                    crate::system::clear_progress::<S>,
                );
            }
            if self.autoclear_on_exit {
                app.add_systems(
                    OnExit(state.clone()),
                    crate::system::clear_progress::<S>,
                );
            }
        }

        #[cfg(feature = "async")]
        {
            app.add_systems(
                PreUpdate,
                crate::process_async_progress::<S>
                    .run_if(in_tracked_state::<S>),
            );
        }

        #[cfg(feature = "debug")]
        if self.enable_debug_logging {
            app.insert_resource(
                crate::debug::ProgressLogger::default().with_enabled(true),
            );
            app.add_systems(
                self.check_progress_schedule,
                crate::debug::debug_progress::<S>
                    .in_set(ProgressCheckSet)
                    .before(check_progress_completion::<S>)
                    .run_if(in_tracked_state::<S>),
            );
        }

        #[cfg(feature = "assets")]
        if self.track_assets {
            app.init_resource::<crate::assets::AssetsTracker<S>>();
            app.add_systems(
                PostUpdate,
                crate::assets::track_assets_progress::<S>
                    .in_set(crate::assets::AssetProgressSet)
                    .run_if(in_tracked_state::<S>),
            );

            for state in self.transitions.get_tracked_states() {
                if self.autoclear_assets_on_enter {
                    app.add_systems(
                        OnEnter(state.clone()),
                        crate::assets::clear_assets_tracker::<S>
                            .after(crate::system::clear_progress::<S>),
                    );
                }
                if self.autoclear_assets_on_exit {
                    app.add_systems(
                        OnExit(state.clone()),
                        crate::assets::clear_assets_tracker::<S>
                            .after(crate::system::clear_progress::<S>),
                    );
                }
            }
        }
    }
}

/// Resource for state transition configuration  
#[derive(Resource, Clone)]
pub struct StateTransitionConfig<S: States> {
    transitions: std::collections::HashMap<S, S>,
}

impl<S: States> Default for StateTransitionConfig<S> {
    fn default() -> Self {
        Self {
            transitions: std::collections::HashMap::default(),
        }
    }
}

impl<S: States> StateTransitionConfig<S> {
    /// Add a state transition to track progress during the transition from
    /// `from` to `to`
    pub fn add_transition(&mut self, from: S, to: S) {
        self.transitions.insert(from, to);
    }

    /// Get the destination state for a given source state transition
    pub fn get_transition(&self, from: &S) -> Option<&S> {
        self.transitions.get(from)
    }

    /// Get an iterator over all tracked states (source states of transitions)
    pub fn get_tracked_states(&self) -> impl Iterator<Item = &S> {
        self.transitions.keys()
    }

    /// Check if progress tracking is enabled for the given state
    pub fn has_transition(&self, state: &S) -> bool {
        self.transitions.contains_key(state)
    }
}

/// System to check if progress is complete and emit completion event
fn check_progress_completion<S: FreelyMutableState>(
    monitor: Res<ProgressMonitor<S>>,
    _config: Res<StateTransitionConfig<S>>,
    state: Res<State<S>>,
    mut completion_writer: EventWriter<ProgressComplete<S>>,
) {
    if monitor.is_complete() {
        completion_writer.write(ProgressComplete {
            state: state.get().clone(),
        });
    }
}

/// System to handle progress completion events and trigger state transitions
fn handle_progress_complete<S: FreelyMutableState>(
    mut completion_reader: EventReader<ProgressComplete<S>>,
    config: Res<StateTransitionConfig<S>>,
    mut next_state: ResMut<NextState<S>>,
) {
    for event in completion_reader.read() {
        if let Some(next) = config.get_transition(&event.state) {
            next_state.set(next.clone());
        }
    }
}

/// Run condition to check if we're in a tracked state
pub fn in_tracked_state<S: FreelyMutableState>(
    config: Res<StateTransitionConfig<S>>,
    state: Option<Res<State<S>>>,
) -> bool {
    state
        .map(|s| config.has_transition(s.get()))
        .unwrap_or(false)
}
