# Task: Implement TLS Domain Constraint Validation

## OBJECTIVE
Replace "For now, allow all valid domains if CA is valid" at line 208 in `packages/ecs-tls/src/tls/builder/authority.rs` with proper X.509 name constraint validation according to RFC 5280 Section 4.2.1.10.

## PRIORITY
P1 - CRITICAL - Security issue, certificate validation incomplete

## CURRENT STATE (VERIFIED)

### File Location
**packages/ecs-tls/src/tls/builder/authority.rs:207-209**

Current code:
```rust
// Check if CA has domain constraints (if implemented)
// For now, allow all valid domains if CA is valid
true
```

This is in the `can_sign_for_domain(&self, domain: &str) -> bool` method of the `CertificateAuthority` struct.

### What's Already Working
- Domain format validation (length, structure, characters)
- CA validity checking (time-based)
- Certificate parsing infrastructure via x509-parser 0.18
- Error handling framework in [packages/ecs-tls/src/tls/errors.rs](../packages/ecs-tls/src/tls/errors.rs)

### What's Missing
X.509 Name Constraints validation - the critical security check that ensures a CA can only issue certificates for domains it's authorized to sign.

---

## TECHNICAL BACKGROUND

### X.509 Name Constraints (RFC 5280 Section 4.2.1.10)

Name Constraints is a certificate extension (OID 2.5.29.30) that restricts the namespace within which a CA can issue certificates. It has two components:

1. **Permitted Subtrees** (optional): Whitelist of allowed namespaces
   - If present, the CA can ONLY sign for domains matching these patterns
   - If absent, all names are permitted (unless excluded)

2. **Excluded Subtrees** (optional): Blacklist of forbidden namespaces  
   - Takes precedence over permitted subtrees
   - If a domain matches an exclusion, it MUST be rejected

### DNS Name Matching Rules (RFC 5280)

Per RFC 5280:
- Constraint ".example.com" matches "www.example.com", "api.example.com", "example.com"
- Constraint "example.com" matches ONLY "example.com" (not subdomains)
- Leading dot indicates subdomain matching
- Matching is case-insensitive
- Excluded subtrees have priority over permitted subtrees

---

## LIBRARY SUPPORT CONFIRMED

### x509-parser 0.18 (Already in Cargo.toml)

The x509-parser library **already provides** full NameConstraints support:

**Key Types** (see [./tmp/x509-parser/src/extensions/name_constraints.rs](./tmp/x509-parser/src/extensions/name_constraints.rs)):

```rust
pub struct NameConstraints<'a> {
    pub permitted_subtrees: Option<Vec<GeneralSubtree<'a>>>,
    pub excluded_subtrees: Option<Vec<GeneralSubtree<'a>>>,
}

pub struct GeneralSubtree<'a> {
    pub base: GeneralName<'a>,
    pub minimum: u32,
    pub maximum: Option<u32>,
}

pub enum GeneralName<'a> {
    DNSName(&'a str),
    IPAddress(&'a [u8]),
    RFC822Name(&'a str),
    // ... other name types
}
```

**Extraction Method** (see [./tmp/x509-parser/src/certificate.rs:537](./tmp/x509-parser/src/certificate.rs#L537)):

```rust
pub fn name_constraints(&self) 
    -> Result<Option<BasicExtension<&NameConstraints<'_>>>, X509Error>
```

**No new dependencies needed!** The infrastructure is already in place.

---

## IMPLEMENTATION PLAN

### STEP 1: Add New Error Variants

**File**: `packages/ecs-tls/src/tls/errors.rs`

**Action**: Add these two variants to the `TlsError` enum (after line 48):

```rust
#[error("Domain explicitly excluded by CA name constraints: {domain}")]
DomainExcluded { domain: String },

#[error("Domain not permitted by CA name constraints: {domain}")]
DomainConstraintViolation { domain: String },
```

**Why**: These provide clear, specific error messages for the two failure modes of name constraint validation.

---

### STEP 2: Create Name Constraint Extraction Helper

**File**: `packages/ecs-tls/src/tls/builder/authority.rs`

**Location**: Add this function after `extract_key_size` (around line 130)

**Action**: Create helper to extract and parse name constraints from CA certificate

```rust
/// Extract name constraints from CA certificate PEM
/// Returns None if no name constraints extension present
fn extract_name_constraints(
    cert_pem: &str,
) -> Result<Option<x509_parser::extensions::NameConstraints<'static>>, TlsError> {
    use x509_parser::prelude::*;
    
    // Parse the certificate from PEM
    let (_, pem) = parse_x509_pem(cert_pem.as_bytes())
        .map_err(|e| TlsError::CertificateParsing(
            format!("Failed to parse CA certificate PEM: {}", e)
        ))?;
    
    let cert = pem.parse_x509()
        .map_err(|e| TlsError::CertificateParsing(
            format!("Failed to parse X.509 certificate: {}", e)
        ))?;
    
    // Extract name constraints extension using x509-parser's built-in method
    match cert.name_constraints() {
        Ok(Some(ext)) => {
            // Clone the name constraints to extend lifetime to 'static
            // This is safe because we're working with owned String data
            let constraints = ext.value.clone();
            Ok(Some(constraints))
        },
        Ok(None) => {
            // No name constraints extension = no restrictions
            Ok(None)
        },
        Err(e) => Err(TlsError::CertificateParsing(
            format!("Failed to extract name constraints: {}", e)
        )),
    }
}
```

**References**:
- See [./tmp/x509-parser/src/certificate.rs:537](./tmp/x509-parser/src/certificate.rs#L537) for name_constraints() method
- See [./tmp/x509-parser/src/extensions/name_constraints.rs](./tmp/x509-parser/src/extensions/name_constraints.rs) for types

---

### STEP 3: Implement DNS Name Matching Logic

**File**: `packages/ecs-tls/src/tls/builder/authority.rs`

**Location**: Add after `extract_name_constraints` function

**Action**: Implement RFC 5280 compliant DNS matching

```rust
/// Check if domain matches a DNS name constraint pattern per RFC 5280
/// 
/// Rules:
/// - Pattern ".example.com" matches "sub.example.com" AND "example.com"
/// - Pattern "example.com" matches ONLY "example.com" (no subdomains)
/// - Matching is case-insensitive
fn dns_name_matches(domain: &str, pattern: &str) -> bool {
    let domain_lower = domain.to_lowercase();
    let pattern_lower = pattern.to_lowercase();
    
    // Exact match
    if domain_lower == pattern_lower {
        return true;
    }
    
    // Subdomain match: pattern starts with '.'
    if pattern_lower.starts_with('.') {
        // Domain must end with the pattern (including the leading dot)
        // OR domain must equal the pattern without the leading dot
        let suffix = &pattern_lower[1..]; // Remove leading dot
        
        if domain_lower == suffix {
            // Exact match with base domain (pattern ".example.com" matches "example.com")
            return true;
        }
        
        if domain_lower.ends_with(&pattern_lower) {
            // Subdomain match (pattern ".example.com" matches "www.example.com")
            return true;
        }
    }
    
    false
}
```

**Why**: RFC 5280 Section 4.2.1.10 specifies these exact matching rules. The leading dot is the critical indicator for subdomain matching.

---

### STEP 4: Implement Constraint Validation Logic

**File**: `packages/ecs-tls/src/tls/builder/authority.rs`

**Location**: Add after `dns_name_matches` function

**Action**: Create the core validation function

```rust
/// Validate that a domain satisfies CA name constraints
/// 
/// Per RFC 5280:
/// 1. Check excluded subtrees first (they take precedence)
/// 2. If excluded subtrees present and domain matches any, REJECT
/// 3. If permitted subtrees present, domain must match at least one
/// 4. If no permitted subtrees, all names are permitted (unless excluded)
fn validate_domain_against_constraints(
    domain: &str,
    constraints: &x509_parser::extensions::NameConstraints,
) -> Result<(), TlsError> {
    use x509_parser::extensions::GeneralName;
    
    // STEP 1: Check excluded subtrees (RFC 5280: exclusions take precedence)
    if let Some(excluded) = &constraints.excluded_subtrees {
        for subtree in excluded {
            if let GeneralName::DNSName(pattern) = &subtree.base {
                if dns_name_matches(domain, pattern) {
                    return Err(TlsError::DomainExcluded {
                        domain: domain.to_string(),
                    });
                }
            }
        }
    }
    
    // STEP 2: Check permitted subtrees
    if let Some(permitted) = &constraints.permitted_subtrees {
        // If permitted subtrees exist, domain MUST match at least one
        let mut found_match = false;
        
        for subtree in permitted {
            if let GeneralName::DNSName(pattern) = &subtree.base {
                if dns_name_matches(domain, pattern) {
                    found_match = true;
                    break;
                }
            }
        }
        
        if !found_match {
            return Err(TlsError::DomainConstraintViolation {
                domain: domain.to_string(),
            });
        }
    }
    
    // STEP 3: Domain passed all checks
    // Either: no excluded match AND (no permitted list OR matched permitted)
    Ok(())
}
```

**Why**: This implements the exact RFC 5280 logic with proper precedence and fallback behavior.

---

### STEP 5: Integrate into can_sign_for_domain Method

**File**: `packages/ecs-tls/src/tls/builder/authority.rs`

**Location**: Lines 207-209 (the current TODO comment)

**Action**: Replace the comment and `true` with actual validation

**BEFORE** (lines 181-209):
```rust
pub fn can_sign_for_domain(&self, domain: &str) -> bool {
    if !self.is_valid() {
        return false;
    }

    // Validate domain format
    if domain.is_empty() || domain.len() > 255 {
        return false;
    }

    // Check for valid domain characters and structure
    let parts: Vec<&str> = domain.split('.').collect();
    if parts.len() < 2 {
        return false; // Domain must have at least two parts
    }

    for part in &parts {
        if part.is_empty() || part.len() > 63 {
            return false;
        }

        // Check that part contains only valid domain characters
        if !part.chars().all(|c| c.is_alphanumeric() || c == '-') {
            return false;
        }

        // Cannot start or end with hyphen
        if part.starts_with('-') || part.ends_with('-') {
            return false;
        }
    }

    // Check if CA has domain constraints (if implemented)
    // For now, allow all valid domains if CA is valid
    true
}
```

**AFTER**:
```rust
pub fn can_sign_for_domain(&self, domain: &str) -> bool {
    if !self.is_valid() {
        return false;
    }

    // Validate domain format
    if domain.is_empty() || domain.len() > 255 {
        return false;
    }

    // Check for valid domain characters and structure
    let parts: Vec<&str> = domain.split('.').collect();
    if parts.len() < 2 {
        return false; // Domain must have at least two parts
    }

    for part in &parts {
        if part.is_empty() || part.len() > 63 {
            return false;
        }

        // Check that part contains only valid domain characters
        if !part.chars().all(|c| c.is_alphanumeric() || c == '-') {
            return false;
        }

        // Cannot start or end with hyphen
        if part.starts_with('-') || part.ends_with('-') {
            return false;
        }
    }

    // Validate domain against CA name constraints per RFC 5280
    match extract_name_constraints(&self.certificate_pem) {
        Ok(Some(constraints)) => {
            // Name constraints present, validate domain
            match validate_domain_against_constraints(domain, &constraints) {
                Ok(()) => true,  // Domain satisfies constraints
                Err(e) => {
                    tracing::warn!(
                        "Domain '{}' rejected by CA '{}' name constraints: {}",
                        domain,
                        self.name,
                        e
                    );
                    false
                }
            }
        },
        Ok(None) => {
            // No name constraints = all domains permitted
            true
        },
        Err(e) => {
            // Failed to extract constraints (malformed CA cert)
            tracing::error!(
                "Failed to extract name constraints from CA '{}': {}",
                self.name,
                e
            );
            false  // Fail closed for security
        }
    }
}
```

**Why**: 
- Preserves all existing domain format validation
- Adds name constraint checking using the helper functions
- Logs validation failures for debugging
- Fails closed (returns false) on errors for security
- Maintains backward compatibility (no constraints = all domains OK)

---

## INTEGRATION POINTS

### Where This Gets Called

The `can_sign_for_domain` method is used when determining if a CA should sign a certificate for a specific domain. Key call sites:

1. **Certificate Generation**: Before generating a new certificate
2. **CA Selection**: When choosing which CA to use for a domain
3. **Validation**: When verifying CA authority

### Error Propagation

The implementation returns `bool` (not `Result`) to maintain the existing API. Errors are:
- Logged via `tracing::warn!` for debugging
- Converted to `false` (rejection)
- Fail-closed for security

If detailed error information is needed by callers, a future refactor could change the return type to `Result<bool, TlsError>`.

---

## DEFINITION OF DONE

**Code Changes**:
- [ ] Two new error variants added to `TlsError` enum
- [ ] `extract_name_constraints()` function implemented
- [ ] `dns_name_matches()` function implemented  
- [ ] `validate_domain_against_constraints()` function implemented
- [ ] `can_sign_for_domain()` method updated to use name constraints
- [ ] Lines 207-209 comment removed, replaced with actual implementation

**Compilation**:
- [ ] Code compiles without errors: `cargo check --package ecs-tls`
- [ ] No new warnings introduced

**Correctness**:
- [ ] Follows RFC 5280 Section 4.2.1.10 specification exactly
- [ ] Uses x509-parser's native NameConstraints types
- [ ] Handles both permitted and excluded subtrees
- [ ] Implements case-insensitive DNS matching
- [ ] Excluded subtrees take precedence over permitted
- [ ] No name constraints = all domains permitted
- [ ] Fails closed on parsing errors

**Code Quality**:
- [ ] Logging added for rejected domains (tracing::warn!)
- [ ] Error messages are clear and actionable
- [ ] No unsafe code required
- [ ] No blocking/synchronous operations (all parsing is in-memory)

---

## IMPORTANT NOTES

### What NOT to Do

- ❌ Do NOT add unit tests (separate task)
- ❌ Do NOT add integration tests  
- ❌ Do NOT add benchmarks
- ❌ Do NOT write extensive documentation beyond code comments
- ❌ Do NOT change the method signature of `can_sign_for_domain`
- ❌ Do NOT add new dependencies (x509-parser already included)

### Scope Boundaries

This task is ONLY about:
✅ Implementing the name constraint validation logic
✅ Integrating it into the existing `can_sign_for_domain` method
✅ Adding necessary error types
✅ Making the code compile

Out of scope:
- Testing (handled by QA team)
- Performance optimization (premature at this stage)
- Supporting non-DNS name types (IP, email, etc.)
- Caching parsed constraints (future optimization)

---

## REFERENCE MATERIALS

### RFC 5280 Section 4.2.1.10
https://datatracker.ietf.org/doc/html/rfc5280#section-4.2.1.10

Key excerpts:
> "Restrictions apply to the subject distinguished name and apply to subject alternative names. Restrictions of the form directoryName MUST apply to the subject field in the certificate and to the subject alternative name extensions of type directoryName. Restrictions of the form dNSName MUST be applied to subject alternative names of type dNSName and to the subject distinguished name fields where it is present."

> "Excluded subtrees take precedence over permitted subtrees. A name is within an excluded subtrees if it is within one of the excluded subtrees but not within one of the permitted subtrees."

### x509-parser Documentation
- Crate docs: https://docs.rs/x509-parser/0.18/
- Source code: [./tmp/x509-parser/](./tmp/x509-parser/)
- NameConstraints type: [./tmp/x509-parser/src/extensions/name_constraints.rs](./tmp/x509-parser/src/extensions/name_constraints.rs)
- Certificate extraction: [./tmp/x509-parser/src/certificate.rs:537](./tmp/x509-parser/src/certificate.rs#L537)

### Local Code References  
- CA authority code: [../packages/ecs-tls/src/tls/builder/authority.rs](../packages/ecs-tls/src/tls/builder/authority.rs)
- Error types: [../packages/ecs-tls/src/tls/errors.rs](../packages/ecs-tls/src/tls/errors.rs)
- Certificate validation: [../packages/ecs-tls/src/tls/certificate/validation.rs](../packages/ecs-tls/src/tls/certificate/validation.rs)
- Certificate parsing: [../packages/ecs-tls/src/tls/certificate/parsing.rs](../packages/ecs-tls/src/tls/certificate/parsing.rs)

---

## VERIFICATION CHECKLIST

Before considering this task complete, verify:

1. **Compilation**: `cargo check --package ecs-tls` passes
2. **No Warnings**: No new compiler warnings introduced
3. **Code Location**: Changes only in authority.rs and errors.rs
4. **No Breaking Changes**: Method signature unchanged
5. **Security**: Fails closed on errors (returns false)
6. **RFC Compliance**: Logic matches RFC 5280 exactly
7. **No TODO Comments**: All "for now" comments removed
8. **Imports Added**: Necessary use statements added for x509_parser types
9. **Error Handling**: All Result types properly handled
10. **Logging**: Appropriate tracing::warn/error calls added

---

## EXAMPLE TEST CASES (For Manual Verification)

These are examples for understanding the logic - DO NOT implement as tests:

**Scenario 1: No Name Constraints**
- CA cert: No name constraints extension
- Domain: "anything.com"  
- Expected: true (all domains permitted)

**Scenario 2: Permitted Subtree Match**
- CA cert: permitted = [".example.com"]
- Domain: "www.example.com"
- Expected: true (matches permitted)

**Scenario 3: Permitted Subtree No Match**
- CA cert: permitted = [".example.com"]
- Domain: "other.com"
- Expected: false (not in permitted list)

**Scenario 4: Excluded Subtree**
- CA cert: excluded = [".bad.example.com"]
- Domain: "www.bad.example.com"
- Expected: false (explicitly excluded)

**Scenario 5: Excluded Takes Precedence**
- CA cert: permitted = [".example.com"], excluded = [".bad.example.com"]
- Domain: "www.bad.example.com"
- Expected: false (exclusion wins)

**Scenario 6: Base Domain Match**
- CA cert: permitted = [".example.com"]
- Domain: "example.com"
- Expected: true (leading dot allows base domain)

---

## ESTIMATED IMPLEMENTATION TIME

- Error types: 5 minutes
- extract_name_constraints(): 15 minutes  
- dns_name_matches(): 10 minutes
- validate_domain_against_constraints(): 20 minutes
- Integration into can_sign_for_domain(): 15 minutes
- Compilation fixes and imports: 10 minutes

**Total: ~75 minutes** for experienced Rust developer familiar with the codebase.
