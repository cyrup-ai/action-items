//! Plugin Authentication System
//!
//! Production-grade secure plugin authentication with token store,
//! cryptographically secure tokens, and const-time validation.

use std::sync::atomic::{AtomicU32, Ordering};

use bevy::prelude::*;
use rustc_hash::FxHashMap;

use super::permissions::PluginPermissions;
use crate::types::*;

/// Production token store for secure plugin authentication
#[derive(Resource)]
pub struct PluginTokenStore {
    /// Secure token storage with SHA-256 hashed tokens
    token_hashes: FxHashMap<String, String>, // plugin_id -> token_hash
    /// Token metadata for audit and expiration
    token_metadata: FxHashMap<String, TokenMetadata>,
    /// Revoked tokens for security
    revoked_tokens: rustc_hash::FxHashSet<String>,
}

/// Token metadata for audit and security
#[derive(Debug, Clone)]
pub struct TokenMetadata {
    pub created_at: TimeStamp,
    pub expires_at: Option<TimeStamp>,
    pub last_used: Option<TimeStamp>,
    pub usage_count: u64,
    pub permissions: PluginPermissions,
    pub issuer: String,
}

impl Default for PluginTokenStore {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginTokenStore {
    pub fn new() -> Self {
        Self {
            token_hashes: FxHashMap::default(),
            token_metadata: FxHashMap::default(),
            revoked_tokens: rustc_hash::FxHashSet::default(),
        }
    }

    /// Generate a new secure token for a plugin
    #[inline]
    pub fn generate_token(&mut self, plugin_id: &str, permissions: PluginPermissions) -> String {
        // Generate cryptographically secure token
        use rand::{Rng, rng};

        let mut rng = rng();
        let random1 = rng.random::<u64>();
        let random2 = rng.random::<u64>();

        let token_data = format!("plugin_{}_{:x}_{:x}", plugin_id, random1, random2);

        // Hash the token for secure storage
        let token_hash = self.hash_token(&token_data);

        // Store token metadata
        let metadata = TokenMetadata {
            created_at: TimeStamp::now(),
            expires_at: TimeStamp::now()
                .add_duration(std::time::Duration::from_secs(86400 * 365))
                .ok(), // 1 year
            last_used: None,
            usage_count: 0,
            permissions,
            issuer: "ecs-service-bridge".to_string(),
        };

        self.token_hashes.insert(plugin_id.to_string(), token_hash);
        self.token_metadata.insert(plugin_id.to_string(), metadata);

        token_data
    }

    /// Validate token with production-grade security
    #[inline]
    pub fn validate_token(&mut self, plugin_id: &str, token: &str) -> bool {
        // Check if token is revoked
        if self.revoked_tokens.contains(token) {
            return false;
        }

        // Get stored token hash
        let stored_hash = match self.token_hashes.get(plugin_id) {
            Some(hash) => hash,
            None => return false, // Plugin not registered
        };

        // Hash provided token for comparison
        let provided_hash = self.hash_token(token);

        // Const-time comparison to prevent timing attacks
        let is_valid = self.const_time_compare(stored_hash, &provided_hash);

        if is_valid {
            // Update usage metadata
            if let Some(metadata) = self.token_metadata.get_mut(plugin_id) {
                // Check expiration
                if let Some(expires_at) = metadata.expires_at
                    && TimeStamp::now() > expires_at {
                        return false; // Token expired
                    }

                metadata.last_used = Some(TimeStamp::now());
                metadata.usage_count = metadata.usage_count.saturating_add(1);
            }
        }

        is_valid
    }

    /// Revoke a token for security
    #[inline]
    pub fn revoke_token(&mut self, plugin_id: &str, token: &str) {
        self.revoked_tokens.insert(token.to_string());
        self.token_hashes.remove(plugin_id);
        self.token_metadata.remove(plugin_id);
    }

    /// Hash token securely using SHA-256
    #[inline]
    fn hash_token(&self, token: &str) -> String {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        hasher.update(b"service_bridge_salt"); // Salt to prevent rainbow tables
        format!("{:x}", hasher.finalize())
    }

    /// Const-time string comparison to prevent timing attacks
    #[inline]
    fn const_time_compare(&self, a: &str, b: &str) -> bool {
        if a.len() != b.len() {
            return false;
        }

        let mut result = 0u8;
        for (byte_a, byte_b) in a.bytes().zip(b.bytes()) {
            result |= byte_a ^ byte_b;
        }

        result == 0
    }

    /// Get token metadata for audit
    #[inline]
    pub fn get_token_metadata(&self, plugin_id: &str) -> Option<&TokenMetadata> {
        self.token_metadata.get(plugin_id)
    }

    /// Clean up expired tokens
    #[inline]
    pub fn cleanup_expired_tokens(&mut self) {
        let now = TimeStamp::now();
        let mut expired_plugins = Vec::new();

        for (plugin_id, metadata) in &self.token_metadata {
            if let Some(expires_at) = metadata.expires_at
                && now > expires_at {
                    expired_plugins.push(plugin_id.clone());
                }
        }

        for plugin_id in expired_plugins {
            self.token_hashes.remove(&plugin_id);
            self.token_metadata.remove(&plugin_id);
        }
    }
}

/// Validate authentication token with production token store
#[inline]
pub fn validate_authentication_token(
    plugin_id: &str,
    token: &str,
    token_store: &mut ResMut<PluginTokenStore>,
) -> bool {
    // Basic format validation first
    if token.len() < 32 || token.len() > 512 {
        return false;
    }

    // Check token contains only valid characters
    for byte in token.bytes() {
        if !byte.is_ascii_alphanumeric() && byte != b'_' && byte != b'-' {
            return false;
        }
    }

    // Validate expected prefix format
    let expected_prefix = format!("plugin_{}_", plugin_id);
    if !token.starts_with(&expected_prefix) {
        return false;
    }

    // Extract and validate the hash components after prefix
    let token_suffix = &token[expected_prefix.len()..];
    let hash_parts: Vec<&str> = token_suffix.split('_').collect();

    // Must have at least 2 hash parts (main hash + random component)
    if hash_parts.len() < 2 {
        return false;
    }

    // Validate each hash part is valid hexadecimal (indicating proper generation)
    for part in &hash_parts {
        if part.len() < 8 || part.len() > 16 {
            return false;
        }
        if !part.chars().all(|c| c.is_ascii_hexdigit()) {
            return false;
        }
    }

    // Production token validation using existing PluginTokenStore with const-time comparison
    token_store.validate_token(plugin_id, token)
}

/// Generate unique instance ID for plugin
#[inline]
pub fn generate_instance_id() -> u32 {
    static INSTANCE_COUNTER: AtomicU32 = AtomicU32::new(1);
    INSTANCE_COUNTER.fetch_add(1, Ordering::Relaxed)
}

/// Resource to track last cleanup time safely
#[derive(Resource)]
pub struct TokenCleanupState {
    pub last_cleanup: std::time::SystemTime,
}

impl Default for TokenCleanupState {
    fn default() -> Self {
        Self {
            last_cleanup: std::time::SystemTime::UNIX_EPOCH,
        }
    }
}

/// Token store cleanup and management system for production deployment
pub fn token_store_management_system(
    mut token_store: ResMut<PluginTokenStore>,
    mut cleanup_state: ResMut<TokenCleanupState>,
) {
    // Clean up expired tokens periodically
    let now = std::time::SystemTime::now();
    let should_cleanup = now
        .duration_since(cleanup_state.last_cleanup)
        .map(|d| d.as_secs() > 300) // 5 minute cleanup interval
        .unwrap_or(true);

    if should_cleanup {
        token_store.cleanup_expired_tokens();
        cleanup_state.last_cleanup = now;
        debug!("Cleaned up expired authentication tokens");
    }
}
