use std::path::Path;

#[cfg(target_os = "macos")]
use objc2_core_foundation::{CFError, CFString, CFURL, CFRetained};
#[cfg(target_os = "macos")]
use objc2_security::{
    errSecSuccess, SecCSFlags, SecCopyErrorMessageString, SecRequirement, SecStaticCode,
};
#[cfg(target_os = "macos")]
use std::ptr::NonNull;

#[cfg(target_os = "linux")]
use anyhow::{anyhow, Result as AnyhowResult};
#[cfg(target_os = "linux")]
use glib::{self, prelude::*, MainContext};
#[cfg(target_os = "linux")]
use once_cell::sync::OnceCell;
#[cfg(target_os = "linux")]
use polkit::{Authority, CheckAuthorizationFlags, Subject};
#[cfg(target_os = "linux")]
use sequoia_openpgp::{
    cert::{Cert, CertParser, KeyHandle},
    parse::stream::{DetachedVerifierBuilder, MessageLayer, MessageStructure, VerificationHelper},
    policy::StandardPolicy,
    types::HashAlgorithm,
    Fingerprint,
};
#[cfg(target_os = "linux")]
use sequoia_policy_config::ConfiguredStandardPolicy;
#[cfg(target_os = "linux")]
use std::{fs::File, io::{BufReader, Read}, path::{Path, PathBuf}};

#[cfg(target_os = "windows")]
use std::{os::windows::ffi::OsStrExt, ptr};
#[cfg(target_os = "windows")]
use windows_sys::Win32::{
    Security::WinTrust::{
        WinVerifyTrust, WINTRUST_ACTION_GENERIC_VERIFY_V2, WINTRUST_DATA, WINTRUST_DATA_0,
        WINTRUST_FILE_INFO, WTD_CHOICE_FILE, WTD_REVOKE_WHOLECHAIN, WTD_SAFER_FLAG,
        WTD_STATEACTION_CLOSE, WTD_STATEACTION_IGNORE, WTD_UI_NONE, WTD_UICONTEXT_EXECUTE,
        WTD_USE_DEFAULT_OSVER_CHECK,
    },
};
#[cfg(target_os = "windows")]
use windows_sys::Win32::Security::WinTrust::WTD_REVOCATION_CHECK_END_CERT;

use crate::error::{Error, Result, SecurityError};
use crate::plugins::interface::PluginManifest;
use sha2::{Digest, Sha256};
use tracing::{info, warn};

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

#[cfg(target_os = "linux")]
struct LinuxSignatureMetadata {
    signer_fingerprint: String,
    signature_path: PathBuf,
    artifact_path: PathBuf,
    keyring_path: PathBuf,
}

#[cfg(target_os = "linux")]
impl LinuxSignatureMetadata {
    fn from_manifest(manifest: &PluginManifest) -> Result<Self> {
        let signer_fingerprint = manifest
            .environment
            .get("linux_signer_fingerprint")
            .map(|value| value.trim().to_ascii_uppercase())
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                Error::SecurityVerification(SecurityError::SignatureVerificationFailed(
                    "linux_signer_fingerprint metadata missing for signature verification".to_string(),
                ))
            })?;

        let signature_path = manifest
            .environment
            .get("linux_signature_path")
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
            .ok_or_else(|| {
                Error::SecurityVerification(SecurityError::SignatureVerificationFailed(
                    "linux_signature_path metadata missing for signature verification".to_string(),
                ))
            })?;

        let artifact_path = manifest
            .environment
            .get("linux_signed_artifact")
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
            .ok_or_else(|| {
                Error::SecurityVerification(SecurityError::SignatureVerificationFailed(
                    "linux_signed_artifact metadata missing for signature verification".to_string(),
                ))
            })?;

        let keyring_path = std::env::var("LINUX_TRUSTED_KEYRING_PATH")
            .map(PathBuf::from)
            .or_else(|_| manifest.environment.get("linux_trusted_keyring").map(|value| PathBuf::from(value.trim())))
            .unwrap_or_else(|| PathBuf::from("/etc/action-items/trusted.gpg"));

        Ok(Self {
            signer_fingerprint,
            signature_path,
            artifact_path,
            keyring_path,
        })
    }
}

#[cfg(target_os = "linux")]
fn perform_gpg_verification(metadata: &LinuxSignatureMetadata) -> Result<()> {
    ensure_path_exists(&metadata.signature_path, "linux_signature_path")?;
    ensure_path_exists(&metadata.artifact_path, "linux_signed_artifact")?;
    ensure_path_exists(&metadata.keyring_path, "trusted keyring path")?;

    let signature_bytes = read_all_bytes(&metadata.signature_path).map_err(|err| {
        Error::SecurityVerification(SecurityError::SignatureVerificationFailed(format!(
            "Failed to read detached signature at {}: {err}",
            metadata.signature_path.display()
        )))
    })?;

    let artifact_file = File::open(&metadata.artifact_path).map_err(|err| {
        Error::SecurityVerification(SecurityError::SignatureVerificationFailed(format!(
            "Failed to open signed artifact at {}: {err}",
            metadata.artifact_path.display()
        )))
    })?;
    let mut artifact_reader = BufReader::new(artifact_file);

    let helper = GpgVerificationHelper::new(&metadata.signer_fingerprint, &metadata.keyring_path)?;
    let policy = helper.policy();

    let mut verifier = DetachedVerifierBuilder::from_bytes(signature_bytes.as_slice())
        .map_err(|err| Error::SecurityVerification(SecurityError::SignatureVerificationFailed(format!(
            "Failed to parse detached signature: {err}"
        ))))?
        .with_policy(policy, None, helper)
        .map_err(|err| Error::SecurityVerification(SecurityError::SignatureVerificationFailed(format!(
            "Failed to initialize OpenPGP verifier: {err}"
        ))))?;

    verifier
        .verify_reader(&mut artifact_reader)
        .map_err(|err| Error::SecurityVerification(SecurityError::SignatureVerificationFailed(format!(
            "Detached signature verification failed for {}: {err}",
            metadata.artifact_path.display()
        ))))?;

    Ok(())
}

#[cfg(target_os = "linux")]
fn ensure_path_exists(path: &Path, label: &str) -> Result<()> {
    if path.exists() {
        Ok(())
    } else {
        Err(Error::SecurityVerification(SecurityError::SignatureVerificationFailed(
            format!("The {label} '{}' does not exist", path.display()),
        )))
    }
}

#[cfg(target_os = "linux")]
fn read_all_bytes(path: &Path) -> std::io::Result<Vec<u8>> {
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}

#[cfg(target_os = "linux")]
struct GpgVerificationHelper {
    policy: &'static StandardPolicy,
    trusted_certs: Vec<Cert>,
    expected_fingerprint: Fingerprint,
}

#[cfg(target_os = "linux")]
impl GpgVerificationHelper {
    fn new(expected_fingerprint: &str, keyring_path: &Path) -> Result<Self> {
        let fingerprint = Fingerprint::from_hex(expected_fingerprint).map_err(|err| {
            Error::SecurityVerification(SecurityError::SignatureVerificationFailed(format!(
                "Invalid linux_signer_fingerprint value: {err}"
            )))
        })?;

        let trusted_certs = load_trusted_certs(keyring_path).map_err(|err| {
            Error::SecurityVerification(SecurityError::SignatureVerificationFailed(format!(
                "Failed to load trusted keyring from {}: {err}",
                keyring_path.display()
            )))
        })?;

        if trusted_certs.is_empty() {
            return Err(Error::SecurityVerification(SecurityError::SignatureVerificationFailed(
                "Trusted keyring contains no certificates".to_string(),
            )));
        }

        let policy = load_policy().map_err(|err| {
            Error::SecurityVerification(SecurityError::SignatureVerificationFailed(format!(
                "Failed to load Sequoia policy configuration: {err}"
            )))
        })?;

        Ok(Self {
            policy,
            trusted_certs,
            expected_fingerprint: fingerprint,
        })
    }

    fn policy(&self) -> &'static StandardPolicy {
        self.policy
    }
}

#[cfg(target_os = "linux")]
impl VerificationHelper for GpgVerificationHelper {
    fn get_certs(&mut self, _ids: &[KeyHandle]) -> sequoia_openpgp::Result<Vec<Cert>> {
        let matches: Vec<Cert> = self
            .trusted_certs
            .iter()
            .filter(|cert| {
                cert.fingerprints().any(|fingerprint| fingerprint == self.expected_fingerprint)
            })
            .cloned()
            .collect();

        if matches.is_empty() {
            return Err(sequoia_openpgp::Error::from(anyhow!(
                "Expected signer with fingerprint {} not found in trusted keyring",
                self.expected_fingerprint
            )));
        }

        Ok(matches)
    }

    fn check(&mut self, structure: MessageStructure) -> sequoia_openpgp::Result<()> {
        for layer in structure.into_iter() {
            if let MessageLayer::SignatureGroup { results } = layer {
                for result in results {
                    let signature = result.map_err(sequoia_openpgp::Error::from)?;
                    let certificant = signature.cert();

                    let valid_key = certificant
                        .keys()
                        .with_policy(self.policy, None)
                        .supports(HashAlgorithm::SHA256)
                        .for_signing()
                        .alive()
                        .revoked(false)
                        .certified()
                        .find(|key| key.fingerprint() == self.expected_fingerprint);

                    if valid_key.is_none() {
                        return Err(sequoia_openpgp::Error::from(anyhow!(
                            "Signer key is not valid for signing or fingerprint mismatch"
                        )));
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(target_os = "linux")]
fn load_trusted_certs(path: &Path) -> AnyhowResult<Vec<Cert>> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer)?;

    // Try single cert first.
    if let Ok(cert) = Cert::from_bytes(buffer.as_slice()) {
        return Ok(vec![cert]);
    }

    // Fall back to parsing keyring.
    let mut parser = CertParser::from_bytes(buffer.as_slice())?;
    let mut certs = Vec::new();
    while let Some(cert) = parser.next().transpose()? {
        certs.push(cert);
    }

    if certs.is_empty() {
        Err(anyhow!("Trusted keyring contained no certificates"))
    } else {
        Ok(certs)
    }
}

#[cfg(target_os = "linux")]
fn load_policy() -> AnyhowResult<&'static StandardPolicy> {
    static POLICY: OnceCell<&'static StandardPolicy> = OnceCell::new();

    POLICY.get_or_try_init(|| {
        let mut configured = ConfiguredStandardPolicy::new();

        if let Err(err) = configured.parse_env_config(ConfiguredStandardPolicy::ENV_VAR) {
            warn!("Failed to parse Sequoia policy from env: {err}");
        }
        if let Err(err) = configured.parse_default_config() {
            warn!("Failed to parse default Sequoia crypto policy: {err}");
        }

        let policy = configured.build();
        Ok(Box::leak(Box::new(policy)))
    })
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
        let bundle_path_value = manifest.environment.get("macos_bundle_path").ok_or_else(|| {
            Error::SecurityVerification(SecurityError::SignatureVerificationFailed(
                "macOS bundle path not provided for signature verification".to_string(),
            ))
        })?;
        let bundle_path = bundle_path_value.trim();
        if bundle_path.is_empty() {
            return Err(Error::SecurityVerification(
                SecurityError::SignatureVerificationFailed(
                    "macOS bundle path is empty".to_string(),
                ),
            ));
        }

        let path = Path::new(bundle_path);
        if !path.exists() {
            return Err(Error::SecurityVerification(SecurityError::SignatureVerificationFailed(
                format!("macOS bundle path {bundle_path} does not exist"),
            )));
        }

        let cf_url = if path.is_dir() {
            CFURL::from_directory_path(path)
        } else {
            CFURL::from_file_path(path)
        }
        .ok_or_else(|| {
            Error::SecurityVerification(SecurityError::SignatureVerificationFailed(
                format!("Failed to construct CFURL for macOS bundle path {bundle_path}"),
            ))
        })?;

        let mut static_code_ptr: *const SecStaticCode = std::ptr::null();
        let static_code_slot = NonNull::from(&mut static_code_ptr);
        let create_status = unsafe {
            SecStaticCode::create_with_path(&cf_url, SecCSFlags::DefaultFlags, static_code_slot)
        };
        if create_status != errSecSuccess {
            return Err(Error::SecurityVerification(
                SecurityError::SignatureVerificationFailed(format!(
                    "SecStaticCodeCreateWithPath failed for {bundle_path}: {}",
                    macos_status_message(create_status, None)
                )),
            ));
        }

        let static_code = NonNull::new(static_code_ptr as *mut SecStaticCode)
            .map(|ptr| unsafe { CFRetained::from_raw(ptr) })
            .ok_or_else(|| {
                Error::SecurityVerification(SecurityError::SignatureVerificationFailed(
                    format!("SecStaticCodeCreateWithPath did not return a code reference for {bundle_path}"),
                ))
            })?;

        let requirement_value = manifest.environment.get("macos_requirement").ok_or_else(|| {
            Error::SecurityVerification(SecurityError::SignatureVerificationFailed(
                "macos_requirement not provided for signature verification".to_string(),
            ))
        })?;
        let requirement_text = requirement_value.trim();
        if requirement_text.is_empty() {
            return Err(Error::SecurityVerification(
                SecurityError::SignatureVerificationFailed(
                    "macos_requirement was provided but empty".to_string(),
                ),
            ));
        }

        let cf_string = CFString::from_str(requirement_text);
        let mut requirement_ptr: *mut SecRequirement = std::ptr::null_mut();
        let requirement_slot = NonNull::from(&mut requirement_ptr);
        let requirement_status = unsafe {
            SecRequirement::create_with_string(
                &cf_string,
                SecCSFlags::DefaultFlags,
                requirement_slot,
            )
        };
        if requirement_status != errSecSuccess {
            return Err(Error::SecurityVerification(
                SecurityError::SignatureVerificationFailed(format!(
                    "SecRequirementCreateWithString failed: {}",
                    macos_status_message(requirement_status, None)
                )),
            ));
        }

        let requirement = NonNull::new(requirement_ptr)
            .map(|ptr| unsafe { CFRetained::from_raw(ptr) })
            .ok_or_else(|| {
                Error::SecurityVerification(SecurityError::SignatureVerificationFailed(
                    "SecRequirementCreateWithString returned null".to_string(),
                ))
            })?;

        let requirement_ref = Some(requirement.as_ref());

        let mut error_ref: *mut CFError = std::ptr::null_mut();
        let validity_flags =
            SecCSFlags::ConsiderExpiration | SecCSFlags::CheckTrustedAnchors | SecCSFlags::EnforceRevocationChecks;
        let validity_status = unsafe {
            static_code
                .as_ref()
                .check_validity_with_errors(validity_flags, requirement_ref, &mut error_ref)
        };

        if validity_status != errSecSuccess {
            let cf_error = NonNull::new(error_ref).map(|ptr| unsafe { CFRetained::from_raw(ptr) });
            return Err(Error::SecurityVerification(
                SecurityError::SignatureVerificationFailed(format!(
                    "macOS code signing verification failed for {bundle_path}: {}",
                    macos_status_message(validity_status, cf_error)
                )),
            ));
        }

        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn verify_linux_signature(&self, manifest: &PluginManifest) -> Result<()> {
        let metadata = LinuxSignatureMetadata::from_manifest(manifest)?;

        let polkit_action_value = manifest.environment.get("polkit_action").ok_or_else(|| {
            Error::SecurityVerification(SecurityError::SignatureVerificationFailed(
                "Linux polkit_action metadata missing for signature verification".to_string(),
            ))
        })?;
        let polkit_action = polkit_action_value.trim();
        if polkit_action.is_empty() {
            return Err(Error::SecurityVerification(
                SecurityError::SignatureVerificationFailed(
                    "Linux polkit_action metadata is empty".to_string(),
                ),
            ));
        }

        let subject_value = manifest.environment.get("linux_subject").ok_or_else(|| {
            Error::SecurityVerification(SecurityError::SignatureVerificationFailed(
                "Linux linux_subject metadata missing for signature verification".to_string(),
            ))
        })?;
        let subject_spec = subject_value.trim();
        if subject_spec.is_empty() {
            return Err(Error::SecurityVerification(
                SecurityError::SignatureVerificationFailed(
                    "Linux linux_subject metadata is empty".to_string(),
                ),
            ));
        }

        let context = MainContext::new();
        let acquire_guard = context.acquire().map_err(|err| {
            Error::SecurityVerification(SecurityError::SignatureVerificationFailed(format!(
                "Failed to acquire GLib main context: {err}"
            )))
        })?;

        let authorization_result = {
            let closure_result = context.with_thread_default(|| {
                Subject::from_string(subject_spec).map_err(|err| {
                    Error::SecurityVerification(SecurityError::SignatureVerificationFailed(
                        format!("Invalid linux_subject '{subject_spec}': {err}"),
                    ))
                })
                .and_then(|subject| {
                    Authority::sync(None).map_err(|err| {
                        Error::SecurityVerification(SecurityError::SignatureVerificationFailed(
                            format!("Failed to connect to PolicyKit authority: {err}"),
                        ))
                    })
                    .and_then(|authority| {
                        authority
                            .check_authorization_sync(
                                &subject,
                                polkit_action,
                                None,
                                CheckAuthorizationFlags::NONE,
                                None,
                            )
                            .map_err(|err| {
                                Error::SecurityVerification(
                                    SecurityError::SignatureVerificationFailed(format!(
                                        "PolicyKit authorization check failed: {err}"
                                    )),
                                )
                            })
                    })
                })
            });

            match closure_result {
                Ok(result) => result?,
                Err(err) => {
                    return Err(Error::SecurityVerification(
                        SecurityError::SignatureVerificationFailed(format!(
                            "Failed to set thread default GLib context: {err}"
                        )),
                    ));
                },
            }
        };

        drop(acquire_guard);

        if authorization_result.is_authorized() {
            perform_gpg_verification(&metadata)
        } else {
            let message = if authorization_result.is_challenge() {
                format!(
                    "PolicyKit requires interactive authentication for action {polkit_action} and subject {subject_spec}"
                )
            } else {
                format!(
                    "PolicyKit authorization denied for action {polkit_action} and subject {subject_spec}"
                )
            };
            Err(Error::SecurityVerification(
                SecurityError::SignatureVerificationFailed(message),
            ))
        }
    }

    #[cfg(target_os = "windows")]
    fn verify_windows_authenticode(&self, manifest: &PluginManifest) -> Result<()> {
        let binary_path_value =
            manifest.environment.get("windows_binary_path").ok_or_else(|| {
                Error::SecurityVerification(SecurityError::SignatureVerificationFailed(
                    "Windows binary path not provided for signature verification".to_string(),
                ))
            })?;
        let binary_path = binary_path_value.trim();
        if binary_path.is_empty() {
            return Err(Error::SecurityVerification(
                SecurityError::SignatureVerificationFailed(
                    "Windows binary path is empty".to_string(),
                ),
            ));
        }

        let path = Path::new(binary_path);
        if !path.exists() {
            return Err(Error::SecurityVerification(SecurityError::SignatureVerificationFailed(
                format!("Windows binary path {binary_path} does not exist"),
            )));
        }

        let mut wide_path: Vec<u16> = path
            .as_os_str()
            .encode_wide()
            .collect();
        if !wide_path.ends_with(&[0]) {
            wide_path.push(0);
        }

        let mut file_info = WINTRUST_FILE_INFO::default();
        file_info.cbStruct = std::mem::size_of::<WINTRUST_FILE_INFO>() as u32;
        file_info.pcwszFilePath = wide_path.as_ptr();
        file_info.hFile = 0;
        file_info.pgKnownSubject = ptr::null_mut();

        let mut trust_data = WINTRUST_DATA::default();
        trust_data.cbStruct = std::mem::size_of::<WINTRUST_DATA>() as u32;
        trust_data.dwUnionChoice = WTD_CHOICE_FILE;
        trust_data.dwUIChoice = WTD_UI_NONE;
        trust_data.fdwRevocationChecks = WTD_REVOKE_WHOLECHAIN;
        trust_data.dwStateAction = WTD_STATEACTION_IGNORE;
        trust_data.dwProvFlags =
            WTD_REVOCATION_CHECK_END_CERT | WTD_USE_DEFAULT_OSVER_CHECK | WTD_SAFER_FLAG;
        trust_data.dwUIContext = WTD_UICONTEXT_EXECUTE;
        unsafe {
            trust_data.Anonymous = WINTRUST_DATA_0 { pFile: &mut file_info };
        }

        let mut action_id = WINTRUST_ACTION_GENERIC_VERIFY_V2;
        let status = unsafe {
            WinVerifyTrust(
                0,
                &mut action_id,
                (&mut trust_data as *mut WINTRUST_DATA).cast(),
            )
        };

        if status != 0 {
            close_wintrust_state(&mut trust_data);
            return Err(Error::SecurityVerification(
                SecurityError::SignatureVerificationFailed(format!(
                    "WinVerifyTrust failed for {binary_path} with status 0x{status:08x}",
                    status as u32
                )),
            ));
        }

        close_wintrust_state(&mut trust_data);
        Ok(())
    }
}

#[cfg(target_os = "macos")]
fn macos_status_message(
    status: i32,
    cf_error: Option<CFRetained<CFError>>,
) -> String {
    if let Some(error) = cf_error {
        let description = error.to_string();
        if !description.is_empty() {
            return description;
        }
    }

    unsafe {
        if let Some(cf_string) = SecCopyErrorMessageString(status, std::ptr::null_mut()) {
            let message = cf_string.to_string();
            if !message.is_empty() {
                return message;
            }
        }
    }

    format!("OSStatus({status})")
}

#[cfg(target_os = "windows")]
fn close_wintrust_state(data: &mut WINTRUST_DATA) {
    if data.hWVTStateData != 0 {
        let mut action_id = WINTRUST_ACTION_GENERIC_VERIFY_V2;
        data.dwStateAction = WTD_STATEACTION_CLOSE;
        unsafe {
            let _ = WinVerifyTrust(0, &mut action_id, (data as *mut WINTRUST_DATA).cast());
        }
        data.dwStateAction = WTD_STATEACTION_IGNORE;
        data.hWVTStateData = 0;
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
            let mut keys: Vec<String> = map.keys().cloned().collect();
            keys.sort();

            let mut ordered = serde_json::Map::with_capacity(keys.len());
            for key in keys {
                if let Some(mut val) = map.remove(&key) {
                    canonicalize_value(&mut val);
                    ordered.insert(key, val);
                }
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
