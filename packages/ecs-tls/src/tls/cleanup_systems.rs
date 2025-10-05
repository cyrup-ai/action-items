//! Bevy systems for TLS cache cleanup management
//!
//! This module provides Bevy ECS systems to handle periodic cleanup of TLS caches
//! instead of using tokio::spawn for long-running tasks.

use std::time::{Duration, Instant};

use bevy::prelude::*;

use super::crl_cache::CrlCache;
use super::ocsp::OcspCache;

/// Resource for tracking OCSP cache cleanup timing
#[derive(Resource)]
pub struct OcspCleanupTimer {
    pub last_cleanup: Instant,
    pub interval: Duration,
}

impl Default for OcspCleanupTimer {
    fn default() -> Self {
        Self {
            last_cleanup: Instant::now(),
            interval: Duration::from_secs(3600), // Cleanup every hour
        }
    }
}

/// Resource for tracking CRL cache cleanup timing
#[derive(Resource)]
pub struct CrlCleanupTimer {
    pub last_cleanup: Instant,
    pub interval: Duration,
}

impl Default for CrlCleanupTimer {
    fn default() -> Self {
        Self {
            last_cleanup: Instant::now(),
            interval: Duration::from_secs(6 * 3600), // Cleanup every 6 hours
        }
    }
}

/// Component to mark entities that have TLS caches
#[derive(Component)]
pub struct TlsCacheHolder {
    pub ocsp_cache: OcspCache,
    pub crl_cache: CrlCache,
}

/// System for periodic OCSP cache cleanup
pub fn ocsp_cache_cleanup_system(
    mut ocsp_timer: ResMut<OcspCleanupTimer>,
    query: Query<&TlsCacheHolder>,
) {
    let now = Instant::now();

    if now.duration_since(ocsp_timer.last_cleanup) >= ocsp_timer.interval {
        for cache_holder in query.iter() {
            cache_holder.ocsp_cache.cleanup_cache();
        }

        ocsp_timer.last_cleanup = now;
        tracing::debug!("Performed OCSP cache cleanup");
    }
}

/// System for periodic CRL cache cleanup
pub fn crl_cache_cleanup_system(
    mut crl_timer: ResMut<CrlCleanupTimer>,
    query: Query<&TlsCacheHolder>,
) {
    let now = Instant::now();

    if now.duration_since(crl_timer.last_cleanup) >= crl_timer.interval {
        for cache_holder in query.iter() {
            cache_holder.crl_cache.cleanup_cache();
        }

        crl_timer.last_cleanup = now;
        tracing::debug!("Performed CRL cache cleanup");
    }
}

/// Plugin for TLS cleanup systems
pub struct TlsCleanupPlugin;

impl Plugin for TlsCleanupPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<OcspCleanupTimer>()
            .init_resource::<CrlCleanupTimer>()
            .add_systems(
                Update,
                (ocsp_cache_cleanup_system, crl_cache_cleanup_system).chain(),
            );
    }
}
