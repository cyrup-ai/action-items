//! Plugin Permissions System
//!
//! Bitfield-based permission system for maximum performance with O(1) operations
//! and compact memory representation.

use bevy::prelude::*;
use rustc_hash::FxHashMap;

/// Plugin permissions with bitfield-based operations for maximum performance
#[derive(Debug, Clone, Component, Default)]
#[repr(C)] // Optimal memory layout
pub struct PluginPermissions {
    /// Bitfield of permissions for O(1) checks
    permissions: u64,
    /// Additional permissions beyond the first 64
    extended_permissions: FxHashMap<String, bool>,
}

impl PluginPermissions {
    /// Permission bit positions
    pub const CLIPBOARD_READ: u64 = 1 << 0;
    pub const CLIPBOARD_WRITE: u64 = 1 << 1;
    pub const STORAGE_READ: u64 = 1 << 2;
    pub const STORAGE_WRITE: u64 = 1 << 3;
    pub const HTTP_REQUEST: u64 = 1 << 4;
    pub const NOTIFICATION_SEND: u64 = 1 << 5;
    pub const PLUGIN_DISCOVERY: u64 = 1 << 6;
    pub const SYSTEM_INFO: u64 = 1 << 7;
    pub const FILE_READ: u64 = 1 << 8;
    pub const FILE_WRITE: u64 = 1 << 9;
    pub const NETWORK_ACCESS: u64 = 1 << 10;

    /// Create new permissions with no access
    #[inline]
    pub fn new() -> Self {
        Self {
            permissions: 0,
            extended_permissions: FxHashMap::default(),
        }
    }

    /// Create permissions with basic access
    #[inline]
    pub fn basic() -> Self {
        Self {
            permissions: Self::PLUGIN_DISCOVERY | Self::SYSTEM_INFO,
            extended_permissions: FxHashMap::default(),
        }
    }

    /// Grant permission with O(1) operation
    #[inline]
    pub fn grant(&mut self, permission: u64) {
        self.permissions |= permission;
    }

    /// Revoke permission with O(1) operation
    #[inline]
    pub fn revoke(&mut self, permission: u64) {
        self.permissions &= !permission;
    }

    /// Check permission with O(1) const-time operation
    #[inline]
    pub const fn has_permission(&self, permission: u64) -> bool {
        (self.permissions & permission) == permission
    }

    /// Grant extended permission
    #[inline]
    pub fn grant_extended(&mut self, permission: String) {
        self.extended_permissions.insert(permission, true);
    }

    /// Check extended permission
    #[inline]
    pub fn has_extended_permission(&self, permission: &str) -> bool {
        self.extended_permissions
            .get(permission)
            .copied()
            .unwrap_or(false)
    }

    /// Builder pattern for fluent API
    #[inline]
    pub fn with_permission(mut self, permission: u64) -> Self {
        self.grant(permission);
        self
    }

    /// Builder pattern for extended permissions
    #[inline]
    pub fn with_extended_permission(mut self, permission: String) -> Self {
        self.grant_extended(permission);
        self
    }
}
