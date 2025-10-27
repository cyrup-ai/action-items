//! Tests for auth/mod.rs

use action_items_ecs_fetch::auth::*;
use base64::engine::general_purpose;
use base64::Engine;
use reqwest::header::{HeaderMap, AUTHORIZATION};
use std::time::{Duration, SystemTime};

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
    let auth_manager = AuthManager::default();
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
