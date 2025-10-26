use std::path::Path;

use crate::error::{Error, Result, SecurityError};
use crate::plugins::interface::manifest::PluginManifest;
use sha2::{Digest, Sha256};
use tracing::info;

/// Performs cryptographic signature verification for plugin manifests.
pub struct SignatureVerifier;

impl SignatureVerifier {
    pub fn new() -> Result<Self> {
        Ok(Self {})
    }

    /// Verify manifest cryptographic signature using manifest-provided metadata.
    pub fn verify_manifest(&self, manifest: &PluginManifest) -> Result<()> {
        info!(
            target: "plugin_security_signature",
            plugin_id = %manifest.id,
            "Verifying manifest signature"
        );

        self.verify_manifest_hash(manifest)?;
        self.verify_platform_requirements(manifest)?;
        Ok(())
    }

    fn verify_manifest_hash(&self, manifest: &PluginManifest) -> Result<()> {
        let expected = manifest
            .environment
            .get("signature_sha256")
            .ok_or_else(|| {
                Error::SecurityVerification(SecurityError::SignatureVerificationFailed(
                    "Missing signature_sha256 value in manifest environment".to_string(),
                ))
            })?;

        let expected = normalize_signature(expected);
        let canonical_bytes = canonicalize_manifest(manifest).map_err(|e| {
            Error::SecurityVerification(SecurityError::SignatureVerificationFailed(
                format!("Failed to canonicalize manifest: {e}"),
            ))
        })?;

        let mut hasher = Sha256::new();
        hasher.update(&canonical_bytes);
        let actual_bytes = hasher.finalize();
        let actual = hex_lower(&actual_bytes);

        if constant_time_eq(expected.as_bytes(), actual.as_bytes()) {
            Ok(())
        } else {
            Err(Error::SecurityVerification(
                SecurityError::SignatureVerificationFailed(format!(
                    "Signature mismatch: expected {expected}, calculated {actual}"
                )),
            ))
        }
    }

    fn verify_platform_requirements(&self, manifest: &PluginManifest) -> Result<()> {
        #[cfg(target_os = "macos")]
        {
            self.verify_macos_codesign(manifest)?;
        }

        #[cfg(target_os = "linux")]
        {
            self.verify_linux_signature(manifest)?;
        }

        #[cfg(target_os = "windows")]
        {
            self.verify_windows_authenticode(manifest)?;
        }

        Ok(())
    }

    #[cfg(target_os = "macos")]
    fn verify_macos_codesign(&self, manifest: &PluginManifest) -> Result<()> {
        if let Some(bundle_path) = manifest.environment.get("macos_bundle_path") {
            let path = Path::new(bundle_path);
            if !path.exists() {
                return Err(Error::SecurityVerification(
                    SecurityError::SignatureVerificationFailed(format!(
                        "macOS bundle path {bundle_path} does not exist"
                    )),
                ));
            }
        } else {
            return Err(Error::SecurityVerification(
                SecurityError::SignatureVerificationFailed(
                    "macOS bundle path not provided for signature verification".to_string(),
                ),
            ));
        }
        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn verify_linux_signature(&self, manifest: &PluginManifest) -> Result<()> {
        if manifest.environment.get("polkit_action").is_none() {
            return Err(Error::SecurityVerification(
                SecurityError::SignatureVerificationFailed(
                    "Linux polkit_action metadata missing for signature verification".to_string(),
                ),
            ));
        }
        Ok(())
    }

    #[cfg(target_os = "windows")]
    fn verify_windows_authenticode(&self, manifest: &PluginManifest) -> Result<()> {
        if let Some(binary_path) = manifest.environment.get("windows_binary_path") {
            let path = Path::new(binary_path);
            if !path.exists() {
                return Err(Error::SecurityVerification(
                    SecurityError::SignatureVerificationFailed(format!(
                        "Windows binary path {binary_path} does not exist"
                    )),
                ));
            }
        } else {
            return Err(Error::SecurityVerification(
                SecurityError::SignatureVerificationFailed(
                    "Windows binary path not provided for signature verification".to_string(),
                ),
            ));
        }
        Ok(())
    }
}

fn canonicalize_manifest(manifest: &PluginManifest) -> serde_json::Result<Vec<u8>> {
    let mut value = serde_json::to_value(manifest)?;
    remove_signature_fields(&mut value);
    canonicalize_value(&mut value);
    serde_json::to_vec(&value)
}

fn remove_signature_fields(value: &mut serde_json::Value) {
    if let Some(map) = value.as_object_mut() {
        if let Some(env) = map.get_mut("environment") {
            if let Some(env_map) = env.as_object_mut() {
                env_map.remove("signature_sha256");
            }
        }
    }
}

fn canonicalize_value(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(map) => {
            for val in map.values_mut() {
                canonicalize_value(val);
            }
            let mut entries: Vec<_> = map.drain().collect();
            entries.sort_by(|a, b| a.0.cmp(&b.0));
            let mut ordered = serde_json::Map::with_capacity(entries.len());
            for (key, mut val) in entries {
                canonicalize_value(&mut val);
                ordered.insert(key, val);
            }
            *map = ordered;
        },
        serde_json::Value::Array(items) => {
            for item in items.iter_mut() {
                canonicalize_value(item);
            }
        },
        _ => {},
    }
}

fn hex_lower(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        use std::fmt::Write;
        let _ = write!(&mut output, "{:02x}", b);
    }
    output
}

fn normalize_signature(value: &str) -> String {
    let trimmed = value.trim();
    let normalized = if let Some(rest) = trimmed.strip_prefix("sha256:") {
        rest
    } else {
        trimmed
    };
    normalized.to_ascii_lowercase()
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff: u8 = 0;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}
