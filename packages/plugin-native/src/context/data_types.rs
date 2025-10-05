use std::collections::HashMap;

use bevy::prelude::*;

/// Request to perform HTTP operation
#[derive(serde::Deserialize, serde::Serialize, Debug, Event, Clone)]
pub struct HttpRequest {
    pub plugin_id: String,
    pub request_id: String,
    pub url: String,
    pub method: HttpMethod,
    pub headers: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Head,
    Patch,
}

impl std::fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpMethod::Get => write!(f, "GET"),
            HttpMethod::Post => write!(f, "POST"),
            HttpMethod::Put => write!(f, "PUT"),
            HttpMethod::Delete => write!(f, "DELETE"),
            HttpMethod::Head => write!(f, "HEAD"),
            HttpMethod::Patch => write!(f, "PATCH"),
        }
    }
}

#[derive(serde::Serialize, Debug, Clone)]
pub struct HttpResponseData {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}
