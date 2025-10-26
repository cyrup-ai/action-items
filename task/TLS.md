# Task: Implement TLS Domain Constraint Validation

## OBJECTIVE
Replace "For now, domain constraints are not checked" with proper X.509 name constraint validation according to RFC 5280.

## PRIORITY
P1 - CRITICAL - Security issue, certificate validation incomplete

## FILE LOCATION
`packages/ecs-tls/src/tls/builder/authority.rs:209`

## SUBTASK 1: Read Current Implementation

Read the authority.rs file to understand:
- What the current certificate validation does
- Where domain constraint checking should happen
- What certificate types are being validated
- What the existing error types are

## SUBTASK 2: Understand X.509 Name Constraints

Study RFC 5280 Section 4.2.1.10:
- Name constraints extension format
- Permitted subtrees
- Excluded subtrees
- DNS name matching rules
- How to extract constraints from certificate

Reference: https://datatracker.ietf.org/doc/html/rfc5280#section-4.2.1.10

## SUBTASK 3: Check Existing Rust Libraries

Investigate if existing dependencies provide name constraint parsing:
- Check if `rustls` provides this
- Check if `x509-parser` provides this
- Check if `webpki` provides this
- Determine if we need to add a dependency or implement manually

## SUBTASK 4: Implement Constraint Extraction

Create function to extract name constraints from certificate:

```rust
use x509_parser::prelude::*;

/// Extract name constraints extension from X.509 certificate
fn extract_name_constraints(cert: &Certificate) -> Result<NameConstraints, TlsError> {
    // Parse certificate
    let (_, parsed_cert) = X509Certificate::from_der(cert.as_ref())
        .map_err(|e| TlsError::CertificateParseError(e.to_string()))?;
    
    // Find name constraints extension (OID 2.5.29.30)
    for ext in parsed_cert.extensions() {
        if ext.oid == oid!(2.5.29.30) {
            return parse_name_constraints(ext.value)?;
        }
    }
    
    // No name constraints extension = all names permitted
    Ok(NameConstraints::default())
}

struct NameConstraints {
    permitted_subtrees: Vec<GeneralSubtree>,
    excluded_subtrees: Vec<GeneralSubtree>,
}
```

## SUBTASK 5: Implement Domain Matching

Create function to check if domain matches constraints:

```rust
impl NameConstraints {
    /// Check if domain is permitted by name constraints
    fn is_permitted(&self, domain: &str) -> bool {
        // If no permitted subtrees, all names are permitted
        if self.permitted_subtrees.is_empty() {
            return true;
        }
        
        // Domain must match at least one permitted subtree
        self.permitted_subtrees.iter()
            .any(|subtree| subtree.matches_domain(domain))
    }
    
    /// Check if domain is explicitly excluded
    fn is_excluded(&self, domain: &str) -> bool {
        self.excluded_subtrees.iter()
            .any(|subtree| subtree.matches_domain(domain))
    }
}

impl GeneralSubtree {
    fn matches_domain(&self, domain: &str) -> bool {
        match &self.base {
            GeneralName::DNSName(pattern) => {
                // Implement DNS name matching per RFC 5280
                dns_name_matches(domain, pattern)
            }
            _ => false, // Only DNS names for now
        }
    }
}

fn dns_name_matches(domain: &str, pattern: &str) -> bool {
    // Exact match
    if domain.eq_ignore_ascii_case(pattern) {
        return true;
    }
    
    // Subdomain match (pattern starts with .)
    if pattern.starts_with('.') {
        let suffix = &pattern[1..];
        if domain.ends_with(suffix) || domain.eq_ignore_ascii_case(suffix) {
            return true;
        }
    }
    
    false
}
```

## SUBTASK 6: Implement Validation Function

Replace the comment at line 209 with:

```rust
/// Validate domain against certificate name constraints
pub fn validate_domain_constraints(
    cert: &Certificate,
    domain: &str,
) -> Result<(), TlsError> {
    let constraints = extract_name_constraints(cert)?;
    
    // Check excluded subtrees first (takes precedence)
    if constraints.is_excluded(domain) {
        return Err(TlsError::DomainExcluded {
            domain: domain.to_string(),
        });
    }
    
    // Check permitted subtrees
    if !constraints.is_permitted(domain) {
        return Err(TlsError::DomainConstraintViolation {
            domain: domain.to_string(),
        });
    }
    
    Ok(())
}
```

## SUBTASK 7: Add Error Types

Add new error variants to TlsError enum:

```rust
pub enum TlsError {
    // ... existing variants ...
    
    /// Domain is explicitly excluded by name constraints
    DomainExcluded {
        domain: String,
    },
    
    /// Domain doesn't match permitted name constraints
    DomainConstraintViolation {
        domain: String,
    },
    
    /// Failed to parse certificate for constraint checking
    CertificateParseError(String),
}
```

## SUBTASK 8: Integrate into Validation Flow

Find where this validation should be called and add:

```rust
// In certificate validation flow:
validate_domain_constraints(&cert, &domain)?;
```

## DEFINITION OF DONE
- [ ] Line 209 comment removed
- [ ] Name constraint extraction implemented
- [ ] Domain matching logic implemented (DNS names)
- [ ] Validation function integrated into cert validation flow
- [ ] Error types added to TlsError enum
- [ ] Code compiles without warnings
- [ ] Implementation follows RFC 5280 spec
- [ ] No "for now" comments remain

## CONSTRAINTS
- DO NOT write unit tests (testing team handles this)
- DO NOT write benchmarks
- DO use existing certificate parsing libraries where possible
- DO follow RFC 5280 specification exactly
- DO handle both permitted and excluded subtrees
- DO NOT use blocking operations

## RESEARCH NOTES
- RFC 5280 Section 4.2.1.10 defines name constraints
- Name constraints extension OID: 2.5.29.30
- DNS name matching is case-insensitive
- Excluded subtrees take precedence over permitted
- If no permitted subtrees, all names are permitted
- Wildcard matching follows specific rules in RFC

## DOCUMENTATION LOCATIONS
- TLS builder: `packages/ecs-tls/src/tls/builder/`
- RFC 5280: https://datatracker.ietf.org/doc/html/rfc5280#section-4.2.1.10
- x509-parser crate: https://docs.rs/x509-parser/
- rustls: Check if name constraint validation exists
