use std::collections::HashMap;
use std::time::{Duration, Instant, SystemTime};

use base64::Engine as _;
use base64::engine::general_purpose;
use bevy::prelude::*;
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderName, HeaderValue};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{debug, error, warn};

/// Authentication configuration for HTTP requests
#[derive(Debug, Clone, Resource)]
pub struct AuthConfig {
    /// Global authentication methods
    pub auth_methods: Vec<AuthMethod>,
    /// Per-domain authentication overrides
    pub domain_auth: HashMap<String, AuthMethod>,
    /// Token refresh configuration
    pub token_refresh_config: TokenRefreshConfig,
    /// Authentication timeouts
    pub auth_timeout: Duration,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            auth_methods: Vec::new(),
            domain_auth: HashMap::new(),
            token_refresh_config: TokenRefreshConfig::default(),
            auth_timeout: Duration::from_secs(30),
        }
    }
}

/// Supported authentication methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthMethod {
    /// Bearer token authentication
    Bearer {
        token: String,
        token_type: Option<String>,
    },
    /// API key authentication
    ApiKey {
        key: String,
        header_name: String,
        prefix: Option<String>,
    },
    /// Basic authentication
    Basic { username: String, password: String },
    /// OAuth 2.0 authentication
    OAuth {
        access_token: String,
        refresh_token: Option<String>,
        token_type: String,
        expires_at: Option<SystemTime>,
        scope: Option<String>,
    },
    /// Custom header authentication
    Custom { headers: HashMap<String, String> },
}

impl AuthMethod {
    /// Apply authentication to request headers
    #[inline]
    pub fn apply_to_headers(&self, headers: &mut HeaderMap) -> Result<(), AuthError> {
        match self {
            AuthMethod::Bearer { token, token_type } => {
                let token_type = token_type.as_deref().unwrap_or("Bearer");
                let auth_value = format!("{} {}", token_type, token);
                let header_value = HeaderValue::from_str(&auth_value)
                    .map_err(|e| AuthError::InvalidHeaderValue(e.to_string()))?;
                headers.insert(AUTHORIZATION, header_value);
            },
            AuthMethod::ApiKey {
                key,
                header_name,
                prefix,
            } => {
                let header_name = HeaderName::from_bytes(header_name.as_bytes())
                    .map_err(|e| AuthError::InvalidHeaderName(e.to_string()))?;

                let value = if let Some(prefix) = prefix {
                    format!("{} {}", prefix, key)
                } else {
                    key.clone()
                };

                let header_value = HeaderValue::from_str(&value)
                    .map_err(|e| AuthError::InvalidHeaderValue(e.to_string()))?;
                headers.insert(header_name, header_value);
            },
            AuthMethod::Basic { username, password } => {
                let credentials = format!("{}:{}", username, password);
                let encoded = general_purpose::STANDARD.encode(credentials);
                let auth_value = format!("Basic {}", encoded);
                let header_value = HeaderValue::from_str(&auth_value)
                    .map_err(|e| AuthError::InvalidHeaderValue(e.to_string()))?;
                headers.insert(AUTHORIZATION, header_value);
            },
            AuthMethod::OAuth {
                access_token,
                token_type,
                expires_at,
                ..
            } => {
                // Check token expiration
                if let Some(expires_at) = expires_at {
                    if SystemTime::now() > *expires_at {
                        return Err(AuthError::TokenExpired);
                    }
                }

                let auth_value = format!("{} {}", token_type, access_token);
                let header_value = HeaderValue::from_str(&auth_value)
                    .map_err(|e| AuthError::InvalidHeaderValue(e.to_string()))?;
                headers.insert(AUTHORIZATION, header_value);
            },
            AuthMethod::Custom {
                headers: custom_headers,
            } => {
                for (name, value) in custom_headers {
                    let header_name = HeaderName::from_bytes(name.as_bytes())
                        .map_err(|e| AuthError::InvalidHeaderName(e.to_string()))?;
                    let header_value = HeaderValue::from_str(value)
                        .map_err(|e| AuthError::InvalidHeaderValue(e.to_string()))?;
                    headers.insert(header_name, header_value);
                }
            },
        }
        Ok(())
    }

    /// Check if authentication method requires token refresh
    #[inline]
    pub fn needs_refresh(&self) -> bool {
        match self {
            AuthMethod::OAuth { expires_at, .. } => {
                expires_at
                    .map(|exp| {
                        SystemTime::now() > exp.checked_sub(Duration::from_secs(300)).unwrap_or(exp)
                    }) // 5 min buffer
                    .unwrap_or(false)
            },
            _ => false,
        }
    }

    /// Get token for refresh operations
    pub fn get_refresh_token(&self) -> Option<&str> {
        match self {
            AuthMethod::OAuth { refresh_token, .. } => refresh_token.as_deref(),
            _ => None,
        }
    }
}

/// Token refresh configuration
#[derive(Debug, Clone)]
pub struct TokenRefreshConfig {
    /// OAuth token endpoint URL
    pub token_endpoint: Option<String>,
    /// Client credentials for token refresh
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    /// Automatic token refresh enabled
    pub auto_refresh: bool,
    /// Refresh threshold before expiration
    pub refresh_threshold: Duration,
}

impl Default for TokenRefreshConfig {
    fn default() -> Self {
        Self {
            token_endpoint: None,
            client_id: None,
            client_secret: None,
            auto_refresh: true,
            refresh_threshold: Duration::from_secs(300), // 5 minutes
        }
    }
}

/// Authentication manager resource
#[derive(Debug, Resource)]
pub struct AuthManager {
    /// Active authentication sessions
    pub auth_sessions: HashMap<String, AuthSession>,
    /// Token cache for reuse
    pub token_cache: HashMap<String, CachedToken>,
}

impl Default for AuthManager {
    fn default() -> Self {
        Self {
            auth_sessions: HashMap::new(),
            token_cache: HashMap::new(),
        }
    }
}

impl AuthManager {
    /// Get authentication method for domain
    pub fn get_auth_for_domain(&self, domain: &str, config: &AuthConfig) -> Option<AuthMethod> {
        // Check domain-specific auth first
        if let Some(auth) = config.domain_auth.get(domain) {
            return Some(auth.clone());
        }

        // Fall back to global auth methods
        config.auth_methods.first().cloned()
    }

    /// Cache authenticated token
    pub fn cache_token(&mut self, key: String, token: String, expires_at: Option<SystemTime>) {
        self.token_cache.insert(key, CachedToken {
            token,
            expires_at,
            cached_at: SystemTime::now(),
        });
    }

    /// Get cached token if still valid
    pub fn get_cached_token(&self, key: &str) -> Option<&str> {
        match self.token_cache.get(key) {
            Some(cached) => {
                if let Some(expires_at) = cached.expires_at {
                    if SystemTime::now() > expires_at {
                        return None; // Expired
                    }
                }
                Some(cached.token.as_str())
            },
            None => None,
        }
    }

    /// Start authentication session
    pub fn start_auth_session(&mut self, session_id: String, auth_method: AuthMethod) {
        self.auth_sessions.insert(session_id, AuthSession {
            auth_method,
            started_at: SystemTime::now(),
            last_used: SystemTime::now(),
            request_count: 0,
        });
    }

    /// Update session usage
    pub fn update_session(&mut self, session_id: &str) {
        if let Some(session) = self.auth_sessions.get_mut(session_id) {
            session.last_used = SystemTime::now();
            session.request_count += 1;
        }
    }

    /// Clean up expired sessions and tokens
    pub fn cleanup_expired(&mut self) {
        let now = SystemTime::now();
        let session_timeout = Duration::from_secs(86400); // 24 hours

        // Remove expired sessions
        self.auth_sessions.retain(|_, session| {
            match now.duration_since(session.last_used) {
                Ok(duration) => duration < session_timeout,
                Err(_) => {
                    warn!("System time inconsistency detected in session cleanup");
                    true // Keep session when time is inconsistent
                },
            }
        });

        // Remove expired tokens
        self.token_cache.retain(|_, cached| {
            if let Some(expires_at) = cached.expires_at {
                now < expires_at
            } else {
                // Keep tokens without expiration, but check age
                match now.duration_since(cached.cached_at) {
                    Ok(age) => age < Duration::from_secs(86400), // 24 hours
                    Err(_) => {
                        warn!("System time inconsistency detected in token cache cleanup");
                        true // Keep token when time is inconsistent
                    },
                }
            }
        });
    }
}

/// Active authentication session
#[derive(Debug, Clone)]
pub struct AuthSession {
    pub auth_method: AuthMethod,
    pub started_at: SystemTime,
    pub last_used: SystemTime,
    pub request_count: u64,
}

/// Cached authentication token
#[derive(Debug, Clone)]
pub struct CachedToken {
    pub token: String,
    pub expires_at: Option<SystemTime>,
    pub cached_at: SystemTime,
}

/// Authentication events
#[derive(Debug, Clone, Event)]
pub struct TokenRefreshRequested {
    pub auth_method: AuthMethod,
    pub domain: String,
    pub requester: String,
}

#[derive(Debug, Clone, Event)]
pub struct TokenRefreshCompleted {
    pub domain: String,
    pub success: bool,
    pub new_token: Option<String>,
    pub expires_at: Option<SystemTime>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Event)]
pub struct AuthenticationFailed {
    pub domain: String,
    pub auth_method_type: String,
    pub error: AuthError,
    pub retry_allowed: bool,
}

/// Authentication errors
#[derive(Debug, Clone, Error, Serialize, Deserialize)]
pub enum AuthError {
    #[error("Invalid header name: {0}")]
    InvalidHeaderName(String),

    #[error("Invalid header value: {0}")]
    InvalidHeaderValue(String),

    #[error("Token expired")]
    TokenExpired,

    #[error("Authentication method not supported: {0}")]
    UnsupportedMethod(String),

    #[error("Missing authentication credentials")]
    MissingCredentials,

    #[error("Token refresh failed: {0}")]
    TokenRefreshFailed(String),

    #[error("Authentication timeout")]
    Timeout,

    #[error("Network error during authentication: {0}")]
    NetworkError(String),

    #[error("Invalid authentication response: {0}")]
    InvalidResponse(String),
}

/// Authentication middleware for request processing
pub struct AuthMiddleware;

impl AuthMiddleware {
    /// Apply authentication to HTTP request
    #[inline]
    pub fn apply_auth(
        headers: &mut HeaderMap,
        url: &str,
        auth_manager: &AuthManager,
        auth_config: &AuthConfig,
    ) -> Result<(), AuthError> {
        // Extract domain from URL
        let domain = url
            .parse::<url::Url>()
            .map_err(|e| AuthError::InvalidResponse(e.to_string()))?
            .host_str()
            .unwrap_or("unknown")
            .to_string();

        // Get authentication method for domain
        if let Some(auth_method) = auth_manager.get_auth_for_domain(&domain, auth_config) {
            // Check if token needs refresh
            if auth_method.needs_refresh() {
                return Err(AuthError::TokenExpired);
            }

            // Apply authentication to headers
            auth_method.apply_to_headers(headers)?;

            debug!("Applied authentication for domain: {}", domain);
        }

        Ok(())
    }

    /// Validate authentication response
    pub fn validate_auth_response(status: u16) -> bool {
        // Consider 2xx status codes as successful authentication
        (200..300).contains(&status)
    }

    /// Extract authentication challenges from response
    pub fn extract_auth_challenges(headers: &HeaderMap) -> Vec<String> {
        headers
            .get_all("www-authenticate")
            .iter()
            .filter_map(|value| value.to_str().ok())
            .map(|s| s.to_string())
            .collect()
    }
}

/// Helper functions for common authentication patterns
pub mod helpers {
    use super::*;

    /// Create Bearer token authentication
    #[inline]
    pub fn bearer_token(token: impl Into<String>) -> AuthMethod {
        AuthMethod::Bearer {
            token: token.into(),
            token_type: Some("Bearer".to_string()),
        }
    }

    /// Create API key authentication
    #[inline]
    pub fn api_key(key: impl Into<String>, header: impl Into<String>) -> AuthMethod {
        AuthMethod::ApiKey {
            key: key.into(),
            header_name: header.into(),
            prefix: None,
        }
    }

    /// Create API key authentication with prefix
    #[inline]
    pub fn api_key_with_prefix(
        key: impl Into<String>,
        header: impl Into<String>,
        prefix: impl Into<String>,
    ) -> AuthMethod {
        AuthMethod::ApiKey {
            key: key.into(),
            header_name: header.into(),
            prefix: Some(prefix.into()),
        }
    }

    /// Create basic authentication
    #[inline]
    pub fn basic_auth(username: impl Into<String>, password: impl Into<String>) -> AuthMethod {
        AuthMethod::Basic {
            username: username.into(),
            password: password.into(),
        }
    }

    /// Create OAuth authentication
    #[inline]
    pub fn oauth_token(
        access_token: impl Into<String>,
        token_type: impl Into<String>,
    ) -> AuthMethod {
        AuthMethod::OAuth {
            access_token: access_token.into(),
            refresh_token: None,
            token_type: token_type.into(),
            expires_at: None,
            scope: None,
        }
    }

    /// Create custom header authentication
    #[inline]
    pub fn custom_headers(headers: HashMap<String, String>) -> AuthMethod {
        AuthMethod::Custom { headers }
    }
}

// Re-export commonly used types
pub use helpers::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bearer_token_auth() {
        let auth = bearer_token("test-token");
        let mut headers = HeaderMap::new();

        auth.apply_to_headers(&mut headers).unwrap();

        assert_eq!(
            headers.get(AUTHORIZATION).unwrap().to_str().unwrap(),
            "Bearer test-token"
        );
    }

    #[test]
    fn test_api_key_auth() {
        let auth = api_key("test-key", "X-API-Key");
        let mut headers = HeaderMap::new();

        auth.apply_to_headers(&mut headers).unwrap();

        assert_eq!(
            headers.get("X-API-Key").unwrap().to_str().unwrap(),
            "test-key"
        );
    }

    #[test]
    fn test_basic_auth() {
        let auth = basic_auth("user", "pass");
        let mut headers = HeaderMap::new();

        auth.apply_to_headers(&mut headers).unwrap();

        let expected = format!("Basic {}", general_purpose::STANDARD.encode("user:pass"));
        assert_eq!(
            headers.get(AUTHORIZATION).unwrap().to_str().unwrap(),
            expected
        );
    }

    #[test]
    fn test_oauth_token_expiration() {
        let expired = SystemTime::now() - Duration::from_secs(100);
        let auth = AuthMethod::OAuth {
            access_token: "token".to_string(),
            refresh_token: None,
            token_type: "Bearer".to_string(),
            expires_at: Some(expired),
            scope: None,
        };

        let mut headers = HeaderMap::new();
        let result = auth.apply_to_headers(&mut headers);

        assert!(matches!(result, Err(AuthError::TokenExpired)));
    }

    #[test]
    fn test_auth_manager_domain_selection() {
        let mut auth_manager = AuthManager::default();
        let mut config = AuthConfig::default();

        // Add global auth
        config.auth_methods.push(bearer_token("global-token"));

        // Add domain-specific auth
        config.domain_auth.insert(
            "api.example.com".to_string(),
            api_key("domain-key", "X-API-Key"),
        );

        // Test global auth
        let global_auth = auth_manager.get_auth_for_domain("other.com", &config);
        assert!(matches!(global_auth, Some(AuthMethod::Bearer { .. })));

        // Test domain-specific auth
        let domain_auth = auth_manager.get_auth_for_domain("api.example.com", &config);
        assert!(matches!(domain_auth, Some(AuthMethod::ApiKey { .. })));
    }
}
