use std::sync::atomic::{AtomicU64, Ordering};

use chrono::{DateTime, Utc};
use serde::Serialize;
use tracing::{error, info, warn};

use crate::error::Result;

static AUDIT_COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Serialize)]
pub struct AuditEntry {
    pub audit_id: u64,
    pub timestamp: DateTime<Utc>,
    pub plugin_id: String,
    pub action_id: String,
    pub result: AuditResult,
    pub reason: Option<String>,
}

#[derive(Debug, Serialize)]
pub enum AuditResult {
    Success,
    Denied,
    Failed,
}

pub struct AuditLogger;

impl AuditLogger {
    pub fn new() -> Result<Self> {
        Ok(Self)
    }

    /// Begin verification and return audit ID
    pub fn begin_verification(&self, plugin_id: &str, action_id: &str) -> u64 {
        let audit_id = AUDIT_COUNTER.fetch_add(1, Ordering::SeqCst);

        info!(
            target: "plugin_capability_audit",
            audit_id,
            plugin_id,
            action_id,
            timestamp = ?Utc::now(),
            "Beginning capability verification"
        );

        audit_id
    }

    /// Log successful verification
    pub fn log_success(&self, audit_id: u64, action_id: &str) {
        info!(
            target: "plugin_capability_audit",
            audit_id,
            action_id,
            result = "success",
            timestamp = ?Utc::now(),
            "Capability verification succeeded"
        );
    }

    /// Log verification denial (not an error, just not granted)
    pub fn log_denial(&self, audit_id: u64, action_id: &str, reason: &str) {
        warn!(
            target: "plugin_capability_audit",
            audit_id,
            action_id,
            result = "denied",
            reason,
            timestamp = ?Utc::now(),
            "Capability verification denied"
        );
    }

    /// Log verification failure (error occurred)
    pub fn log_failure(&self, audit_id: u64, stage: &str, error_value: &dyn std::fmt::Display) {
        error!(
            target: "plugin_capability_audit",
            audit_id,
            stage,
            result = "failed",
            error = %error_value,
            timestamp = ?Utc::now(),
            "Capability verification failed"
        );
    }

    #[allow(dead_code)]
    pub fn serialize_entry(
        &self,
        plugin_id: &str,
        action_id: &str,
        result: AuditResult,
        reason: Option<String>,
    ) -> Result<AuditEntry> {
        Ok(AuditEntry {
            audit_id: AUDIT_COUNTER.load(Ordering::SeqCst),
            timestamp: Utc::now(),
            plugin_id: plugin_id.to_string(),
            action_id: action_id.to_string(),
            result,
            reason,
        })
    }
}
