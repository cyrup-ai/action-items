//! Plugin Lifecycle & Health Management
//!
//! Handles plugin status changes, cleanup, health monitoring, and recovery
//! with atomic operations for maximum performance.

use bevy::prelude::*;

use super::capability_index::PluginCapabilityIndex;
use super::registration::PluginEntity;
use crate::components::{PluginComponent, PluginStatus};
use crate::events::*;
use crate::resources::*;
use crate::types::*;

/// Plugin health monitoring with atomic operations
#[derive(Debug, Clone, Component)]
#[repr(C)] // Optimal memory layout
pub struct PluginHealth {
    pub last_heartbeat: TimeStamp,
    pub response_time_ms: u32,
    pub error_count: u32,
    pub success_count: u32,
    pub health_score: f32, // 0.0 - 1.0
}

impl PluginHealth {
    /// Create new health monitor
    #[inline]
    pub fn new() -> Self {
        Self {
            last_heartbeat: TimeStamp::now(),
            response_time_ms: 0,
            error_count: 0,
            success_count: 0,
            health_score: 1.0,
        }
    }

    /// Update health with new response time
    #[inline]
    pub fn record_success(&mut self, response_time_ms: u32) {
        self.last_heartbeat = TimeStamp::now();
        self.response_time_ms = response_time_ms;
        self.success_count = self.success_count.saturating_add(1);
        self.update_health_score();
    }

    /// Record error
    #[inline]
    pub fn record_error(&mut self) {
        self.error_count = self.error_count.saturating_add(1);
        self.update_health_score();
    }

    /// Update health score based on success/error ratio
    #[inline]
    fn update_health_score(&mut self) {
        let total_operations = self.success_count + self.error_count;
        if total_operations == 0 {
            self.health_score = 1.0;
        } else {
            self.health_score = (self.success_count as f32) / (total_operations as f32);
        }
    }

    /// Check if plugin is healthy
    #[inline]
    pub fn is_healthy(&self) -> bool {
        self.health_score >= 0.8 && self.last_heartbeat.elapsed().as_secs() < 30
    }
}

impl Default for PluginHealth {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

/// Plugin lifecycle management system with elegant state transitions
///
/// Handles plugin status changes, cleanup, health monitoring, and recovery.
pub fn plugin_lifecycle_system(
    mut commands: Commands,
    mut plugin_registry: ResMut<PluginRegistryResource>,
    mut capability_index: ResMut<PluginCapabilityIndex>,
    mut lifecycle_events: EventReader<PluginLifecycleEvent>,
    mut plugin_query: Query<(Entity, &mut PluginComponent, &mut PluginHealth), With<PluginEntity>>,
    time: Res<Time>,
) {
    // Process lifecycle events
    for event in lifecycle_events.read() {
        if let Err(error) = process_lifecycle_event(
            &mut commands,
            &mut plugin_registry,
            &mut capability_index,
            &mut plugin_query,
            event,
        ) {
            error!(
                "Failed to process lifecycle event for plugin {}: {}",
                event.plugin_id, error
            );
        }
    }

    // Monitor plugin health and detect timeouts
    monitor_plugin_health(&mut plugin_registry, &mut plugin_query, &time);
}

/// Process individual lifecycle event with proper state transitions
#[inline]
fn process_lifecycle_event(
    commands: &mut Commands,
    plugin_registry: &mut ResMut<PluginRegistryResource>,
    capability_index: &mut ResMut<PluginCapabilityIndex>,
    plugin_query: &mut Query<(Entity, &mut PluginComponent, &mut PluginHealth), With<PluginEntity>>,
    event: &PluginLifecycleEvent,
) -> ServiceResult<()> {
    match &event.event_type {
        LifecycleEventType::Started => {
            // Update plugin status to Active
            if let Some(plugin_info) = plugin_registry.plugins.get_mut(&event.plugin_id) {
                plugin_info.status = PluginStatus::Active;
                plugin_info.last_heartbeat = Some(TimeStamp::now());
            }

            // Update plugin component
            for (_, mut plugin_component, mut health) in plugin_query.iter_mut() {
                if plugin_component.plugin_id == event.plugin_id {
                    plugin_component.status = PluginStatus::Active;
                    plugin_component.last_heartbeat = Some(TimeStamp::now());
                    health.record_success(0); // Record successful start
                    break;
                }
            }

            info!("Plugin started: {}", event.plugin_id);
        },

        LifecycleEventType::Stopped => {
            // Update plugin status to Inactive
            if let Some(plugin_info) = plugin_registry.plugins.get_mut(&event.plugin_id) {
                plugin_info.status = PluginStatus::Inactive;
            }

            // Update plugin component
            for (_, mut plugin_component, _) in plugin_query.iter_mut() {
                if plugin_component.plugin_id == event.plugin_id {
                    plugin_component.status = PluginStatus::Inactive;
                    break;
                }
            }

            info!("Plugin stopped: {}", event.plugin_id);
        },

        LifecycleEventType::Error(error_msg) => {
            // Update plugin status to Error
            if let Some(plugin_info) = plugin_registry.plugins.get_mut(&event.plugin_id) {
                plugin_info.status = PluginStatus::Error(error_msg.clone());
            }

            // Update plugin component and health
            for (_, mut plugin_component, mut health) in plugin_query.iter_mut() {
                if plugin_component.plugin_id == event.plugin_id {
                    plugin_component.status = PluginStatus::Error(error_msg.clone());
                    health.record_error();
                    break;
                }
            }

            error!("Plugin error {}: {}", event.plugin_id, error_msg);
        },

        LifecycleEventType::Unregistered => {
            // Remove plugin from registry and index
            plugin_registry.plugins.remove(&event.plugin_id);
            // Plugin channels managed by MessageInfrastructure resource
            capability_index.remove_plugin(&event.plugin_id);

            // Despawn plugin entity
            for (entity, plugin_component, _) in plugin_query.iter() {
                if plugin_component.plugin_id == event.plugin_id {
                    commands.entity(entity).despawn();
                    break;
                }
            }

            info!("Plugin unregistered: {}", event.plugin_id);
        },

        LifecycleEventType::StatusChanged(new_status) => {
            info!("Plugin {} status changed: {}", event.plugin_id, new_status);
        },

        LifecycleEventType::Registered => {
            // Already handled in registration system
        },
    }

    Ok(())
}

/// Monitor plugin health and detect unresponsive plugins
#[inline]
fn monitor_plugin_health(
    plugin_registry: &mut ResMut<PluginRegistryResource>,
    plugin_query: &mut Query<(Entity, &mut PluginComponent, &mut PluginHealth), With<PluginEntity>>,
    _time: &Res<Time>,
) {
    let now = TimeStamp::now();

    for (_, mut plugin_component, mut health) in plugin_query.iter_mut() {
        // Check if plugin has been unresponsive
        if let Some(last_heartbeat) = plugin_component.last_heartbeat
            && let Ok(duration) = now.duration_since(last_heartbeat)
                && duration.as_secs() > 60 && plugin_component.status == PluginStatus::Active {
                    // Plugin is unresponsive, mark as inactive
                    plugin_component.status = PluginStatus::Inactive;
                    health.record_error();

                    // Update registry
                    if let Some(plugin_info) =
                        plugin_registry.plugins.get_mut(&plugin_component.plugin_id)
                    {
                        plugin_info.status = PluginStatus::Inactive;
                    }

                    warn!(
                        "Plugin {} marked as inactive due to no heartbeat",
                        plugin_component.plugin_id
                    );
                }
    }
}
