//! Message Translation System
//!
//! Centralized translation logic for converting service bridge messages to ECS events
//! and ECS event responses back to service bridge messages. Follows ARCHITECTURE.md
//! event patterns with proper correlation tracking.

use std::collections::HashMap;

use action_items_ecs_clipboard::{
    ClipboardData, ClipboardFormat, ClipboardRequest, ClipboardResponse,
};
use action_items_ecs_search_aggregator::events::{SearchCompleted, SearchRequested};
use bevy::prelude::*;
use ecs_service_bridge::events::PluginMessageEvent;
use serde_json::Value;
use uuid::Uuid;

use super::entity_mapping::PluginEntityMap;
use super::payload_parsing::{
    parse_clipboard_format, parse_permission_type, validate_clipboard_data, PayloadParseError
};

// Minimal event definitions for services not available as dependencies
#[derive(Debug, Clone, Event)]
pub struct NotificationRequest {
    pub title: String,
    pub body: String,
    pub priority: ecs_service_bridge::types::MessagePriority,
    pub timestamp: ecs_service_bridge::types::TimeStamp,
    pub requester: Option<String>,
}

#[derive(Debug, Clone, Event)]
pub struct NotificationResponse {
    pub id: String,
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Event)]
pub struct HttpRequest {
    pub method: String,
    pub url: String,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

#[derive(Debug, Clone, Event)]
pub struct HttpResponse {
    pub status: u16,
    pub body: String,
    pub headers: HashMap<String, String>,
}

#[derive(Debug, Clone, Event)]
pub struct PermissionRequest {
    pub permission_type: String,
}

#[derive(Debug, Clone, Event)]
pub struct PermissionResponse {
    pub granted: bool,
    pub error: Option<String>,
}

/// Operation correlation ID for tracking requests and responses
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OperationId(pub Uuid);

impl OperationId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

/// Service bridge message types that can be translated to ECS events
#[derive(Debug, Clone)]
pub enum ServiceBridgeMessage {
    // Clipboard operations
    ClipboardRead {
        format: String,
    },
    ClipboardWrite {
        data: String,
        format: String,
    },
    ClipboardClear,
    ClipboardCheckFormat {
        format: String,
    },

    // Notification operations
    NotificationShow {
        title: String,
        message: String,
        duration: Option<u64>,
    },
    NotificationDismiss {
        id: String,
    },
    NotificationCheckAvailability,

    // HTTP operations
    HttpGet {
        url: String,
        headers: HashMap<String, String>,
    },
    HttpPost {
        url: String,
        body: String,
        headers: HashMap<String, String>,
    },
    HttpPut {
        url: String,
        body: String,
        headers: HashMap<String, String>,
    },
    HttpDelete {
        url: String,
        headers: HashMap<String, String>,
    },

    // Search operations
    SearchQuery {
        query: String,
        plugins: Vec<String>,
    },
    SearchCancel {
        search_id: String,
    },

    // Permission operations
    PermissionCheck {
        permission_type: String,
    },
    PermissionRequest {
        permission_type: String,
    },

    // Unknown message type
    Unknown {
        message_type: String,
        payload: Value,
    },
}

/// ECS event response that can be translated back to service bridge messages
#[derive(Debug, Clone)]
pub enum EcsEventResponse {
    // Clipboard responses
    ClipboardData {
        data: String,
        format: String,
    },
    ClipboardWriteSuccess,
    ClipboardClearSuccess,
    ClipboardFormatAvailable {
        available: bool,
    },
    ClipboardError {
        error: String,
    },

    // Notification responses
    NotificationShown {
        id: String,
    },
    NotificationDismissed {
        id: String,
    },
    NotificationAvailable {
        available: bool,
    },
    NotificationError {
        error: String,
    },

    // HTTP responses
    HttpSuccess {
        status: u16,
        body: String,
        headers: HashMap<String, String>,
    },
    HttpError {
        error: String,
        status: Option<u16>,
    },

    // Search responses
    SearchResults {
        results: Vec<SearchResult>,
        search_id: String,
    },
    SearchError {
        error: String,
        search_id: String,
    },

    // Permission responses
    PermissionStatus {
        granted: bool,
        status: String,
    },
    PermissionError {
        error: String,
    },

    // Generic error
    Error {
        message: String,
    },
}

/// Search result structure for translation
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub title: String,
    pub description: String,
    pub action: String,
    pub icon: Option<String>,
    pub score: f32,
    pub plugin_id: String,
}

/// Translate service bridge message to appropriate ECS events
///
/// This function analyzes the service bridge message and creates the corresponding
/// ECS events that should be sent to the appropriate services.
///
/// # Arguments
/// * `message_event` - The service bridge message event to translate
/// * `operation_id` - Correlation ID for tracking the operation
///
/// # Returns
/// * `Ok(Vec<Box<dyn Event>>)` - List of ECS events to send
/// * `Err(String)` - Translation error message
pub fn translate_message_to_ecs_events(
    message_event: &PluginMessageEvent,
    operation_id: OperationId,
    entity_map: &mut PluginEntityMap,
    commands: &mut Commands,
) -> Result<Vec<EcsEventType>, String> {
    let service_message = parse_service_bridge_message(message_event)?;

    match service_message {
        ServiceBridgeMessage::ClipboardRead { format } => {
            let clipboard_format = parse_clipboard_format(&format)
                .map_err(|e| format!("Invalid clipboard format: {}", e))?;
            Ok(vec![EcsEventType::ClipboardRequest(
                ClipboardRequest::Get {
                    format: clipboard_format,
                    requester: entity_map.convert_plugin_id_to_entity(&message_event.plugin_id, commands),
                },
            )])
        },

        ServiceBridgeMessage::ClipboardWrite { data, format } => {
            let validated_data = validate_clipboard_data(&data)
                .map_err(|e| format!("Invalid clipboard data: {}", e))?;
            let clipboard_format = parse_clipboard_format(&format)
                .map_err(|e| format!("Invalid clipboard format: {}", e))?;

            let clipboard_data = match clipboard_format {
                ClipboardFormat::Text => ClipboardData::Text(validated_data),
                ClipboardFormat::Html => ClipboardData::Html {
                    html: validated_data,
                    alt_text: None
                },
                ClipboardFormat::Files => {
                    // Parse newline-separated file paths
                    let paths: Vec<std::path::PathBuf> = validated_data
                        .lines()
                        .map(|line| std::path::PathBuf::from(line.trim()))
                        .collect();
                    ClipboardData::Files(paths)
                },
                #[cfg(feature = "image-data")]
                ClipboardFormat::Image => return Err("Image clipboard data not yet supported".to_string()),
            };
            Ok(vec![EcsEventType::ClipboardRequest(
                ClipboardRequest::Set {
                    data: clipboard_data,
                    requester: entity_map.convert_plugin_id_to_entity(&message_event.plugin_id, commands),
                },
            )])
        },

        ServiceBridgeMessage::ClipboardClear => {
            Ok(vec![EcsEventType::ClipboardRequest(
                ClipboardRequest::Clear {
                    requester: entity_map.convert_plugin_id_to_entity(&message_event.plugin_id, commands),
                },
            )])
        },

        ServiceBridgeMessage::NotificationShow {
            title,
            message,
            duration: _,
        } => Ok(vec![EcsEventType::NotificationRequest(
            NotificationRequest {
                title,
                body: message,
                priority: ecs_service_bridge::events::MessagePriority::Normal,
                timestamp: ecs_service_bridge::types::TimeStamp::now(),
                requester: Some(plugin_id.clone()),
            },
        )]),

        ServiceBridgeMessage::HttpGet { url, headers } => {
            Ok(vec![EcsEventType::HttpRequest(HttpRequest {
                method: "GET".to_string(),
                url,
                headers,
                body: None,
            })])
        },

        ServiceBridgeMessage::HttpPost { url, body, headers } => {
            Ok(vec![EcsEventType::HttpRequest(HttpRequest {
                method: "POST".to_string(),
                url,
                headers,
                body: Some(body),
            })])
        },

        ServiceBridgeMessage::SearchQuery { query, plugins } => {
            Ok(vec![EcsEventType::SearchRequest(SearchRequested::new(
                query,
                plugins.unwrap_or_default(),
            ))])
        },

        ServiceBridgeMessage::PermissionCheck { permission_type } => {
            Ok(vec![EcsEventType::PermissionRequest(PermissionRequest {
                permission_type,
            })])
        },

        ServiceBridgeMessage::PermissionRequest { permission_type } => {
            Ok(vec![EcsEventType::PermissionRequest(PermissionRequest {
                permission_type,
            })])
        },

        ServiceBridgeMessage::Unknown { message_type, .. } => {
            Err(format!("Unknown message type: {}", message_type))
        },

        _ => Err("Unsupported message type for ECS translation".to_string()),
    }
}

/// ECS event types that can be sent
#[derive(Debug)]
pub enum EcsEventType {
    ClipboardRequest(ClipboardRequest),
    NotificationRequest(NotificationRequest),
    HttpRequest(HttpRequest),
    SearchRequest(SearchRequested),
    PermissionRequest(PermissionRequest),
}

/// Translate ECS event response back to service bridge message
///
/// This function converts ECS service responses back into the format expected
/// by the service bridge infrastructure for routing back to plugins.
///
/// # Arguments
/// * `response` - The ECS event response to translate
/// * `operation_id` - Correlation ID for matching with original request
/// * `target_plugin` - Plugin ID to send the response to
///
/// # Returns
/// * `Ok(PluginMessageEvent)` - Service bridge message event to send
/// * `Err(String)` - Translation error message
pub fn translate_ecs_response_to_message(
    response: EcsEventResponse,
    operation_id: OperationId,
    target_plugin: String,
) -> Result<PluginMessageEvent, String> {
    let (message_type, payload) = match response {
        EcsEventResponse::ClipboardData { data, format } => {
            let mut payload = HashMap::new();
            payload.insert("data".to_string(), Value::String(data));
            payload.insert("format".to_string(), Value::String(format));
            payload.insert("success".to_string(), Value::Bool(true));
            (
                "clipboard_response".to_string(),
                Value::Object(serde_json::Map::from_iter(payload.into_iter())),
            )
        },

        EcsEventResponse::ClipboardWriteSuccess => {
            let mut payload = HashMap::new();
            payload.insert("success".to_string(), Value::Bool(true));
            (
                "clipboard_response".to_string(),
                Value::Object(serde_json::Map::from_iter(payload.into_iter())),
            )
        },

        EcsEventResponse::ClipboardError { error } => {
            let mut payload = HashMap::new();
            payload.insert("success".to_string(), Value::Bool(false));
            payload.insert("error".to_string(), Value::String(error));
            (
                "clipboard_response".to_string(),
                Value::Object(serde_json::Map::from_iter(payload.into_iter())),
            )
        },

        EcsEventResponse::NotificationShown { id } => {
            let mut payload = HashMap::new();
            payload.insert("notification_id".to_string(), Value::String(id));
            payload.insert("success".to_string(), Value::Bool(true));
            (
                "notification_response".to_string(),
                Value::Object(serde_json::Map::from_iter(payload.into_iter())),
            )
        },

        EcsEventResponse::NotificationError { error } => {
            let mut payload = HashMap::new();
            payload.insert("success".to_string(), Value::Bool(false));
            payload.insert("error".to_string(), Value::String(error));
            (
                "notification_response".to_string(),
                Value::Object(serde_json::Map::from_iter(payload.into_iter())),
            )
        },

        EcsEventResponse::HttpSuccess {
            status,
            body,
            headers,
        } => {
            let mut payload = HashMap::new();
            payload.insert("status".to_string(), Value::Number(status.into()));
            payload.insert("body".to_string(), Value::String(body));
            payload.insert(
                "headers".to_string(),
                serde_json::to_value(headers).unwrap_or(Value::Null),
            );
            payload.insert("success".to_string(), Value::Bool(true));
            (
                "http_response".to_string(),
                Value::Object(serde_json::Map::from_iter(payload.into_iter())),
            )
        },

        EcsEventResponse::HttpError { error, status } => {
            let mut payload = HashMap::new();
            payload.insert("success".to_string(), Value::Bool(false));
            payload.insert("error".to_string(), Value::String(error));
            if let Some(status) = status {
                payload.insert("status".to_string(), Value::Number(status.into()));
            }
            (
                "http_response".to_string(),
                Value::Object(serde_json::Map::from_iter(payload.into_iter())),
            )
        },

        EcsEventResponse::SearchResults { results, search_id } => {
            let mut payload = HashMap::new();
            let results_json: Vec<Value> = results
                .into_iter()
                .map(|r| {
                    serde_json::json!({
                        "title": r.title,
                        "description": r.description,
                        "action": r.action,
                        "icon": r.icon,
                        "score": r.score,
                        "plugin_id": r.plugin_id
                    })
                })
                .collect();
            payload.insert("results".to_string(), Value::Array(results_json));
            payload.insert("search_id".to_string(), Value::String(search_id));
            payload.insert("success".to_string(), Value::Bool(true));
            (
                "search_response".to_string(),
                Value::Object(serde_json::Map::from_iter(payload.into_iter())),
            )
        },

        EcsEventResponse::PermissionStatus { granted, status } => {
            let mut payload = HashMap::new();
            payload.insert("granted".to_string(), Value::Bool(granted));
            payload.insert("status".to_string(), Value::String(status));
            payload.insert("success".to_string(), Value::Bool(true));
            (
                "permission_response".to_string(),
                Value::Object(serde_json::Map::from_iter(payload.into_iter())),
            )
        },

        EcsEventResponse::Error { message } => {
            let mut payload = HashMap::new();
            payload.insert("success".to_string(), Value::Bool(false));
            payload.insert("error".to_string(), Value::String(message));
            (
                "error_response".to_string(),
                Value::Object(serde_json::Map::from_iter(payload.into_iter())),
            )
        },

        _ => {
            return Err("Unsupported ECS response type for service bridge translation".to_string());
        },
    };

    // Add operation correlation ID to payload
    let mut final_payload = match payload {
        Value::Object(mut map) => {
            map.insert(
                "operation_id".to_string(),
                Value::String(operation_id.0.to_string()),
            );
            Value::Object(map)
        },
        other => {
            let mut map = serde_json::Map::new();
            map.insert("data".to_string(), other);
            map.insert(
                "operation_id".to_string(),
                Value::String(operation_id.0.to_string()),
            );
            Value::Object(map)
        },
    };

    Ok(PluginMessageEvent {
        from: "ecs_service_bridge".to_string(),
        to: target_plugin.clone(),
        message_type,
        payload: final_payload,
        correlation_id: Some(operation_id.0.to_string()),
        priority: ecs_service_bridge::events::MessagePriority::Normal,
        plugin_id: target_plugin,
        timestamp: ecs_service_bridge::types::TimeStamp::now(),
        request_id: Some(operation_id.0.to_string()),
    })
}

/// Parse service bridge message event into structured message type
fn parse_service_bridge_message(
    message_event: &PluginMessageEvent,
) -> Result<ServiceBridgeMessage, String> {
    let payload = &message_event.payload;
    match payload {
        Value::Object(obj) => match message_event.message_type.as_str() {
            "clipboard_read" => {
                let format = obj.get("format").and_then(|v| v.as_str()).unwrap_or("text");
                Ok(ServiceBridgeMessage::ClipboardRead {
                    format: format.to_string(),
                })
            },
            "clipboard_write" => {
                let data = obj
                    .get("data")
                    .and_then(|v| v.as_str())
                    .ok_or("Missing data field")?;
                let format = obj.get("format").and_then(|v| v.as_str()).unwrap_or("text");
                Ok(ServiceBridgeMessage::ClipboardWrite {
                    data: data.to_string(),
                    format: format.to_string(),
                })
            },
            "clipboard_clear" => Ok(ServiceBridgeMessage::ClipboardClear),

            "notification_show" => {
                let title = obj
                    .get("title")
                    .and_then(|v| v.as_str())
                    .ok_or("Missing title field")?;
                let message = obj
                    .get("message")
                    .and_then(|v| v.as_str())
                    .ok_or("Missing message field")?;
                let duration = obj
                    .get("duration")
                    .and_then(|v| v.as_u64());
                Ok(ServiceBridgeMessage::NotificationShow {
                    title: title.to_string(),
                    message: message.to_string(),
                    duration,
                })
            },
            "http_get" => {
                let url = obj
                    .get("url")
                    .and_then(|v| v.as_str())
                    .ok_or("Missing url field")?;
                let headers = parse_headers(&message_event.payload)?;
                Ok(ServiceBridgeMessage::HttpGet {
                    url: url.to_string(),
                    headers,
                })
            },
            "http_post" => {
                let url = obj
                    .get("url")
                    .and_then(|v| v.as_str())
                    .ok_or("Missing url field")?;
                let body = obj.get("body").and_then(|v| v.as_str()).unwrap_or("");
                let headers = parse_headers(&message_event.payload)?;
                Ok(ServiceBridgeMessage::HttpPost {
                    url: url.to_string(),
                    body: body.to_string(),
                    headers,
                })
            },
            "search" => {
                let query = obj
                    .get("query")
                    .and_then(|v| v.as_str())
                    .ok_or("Missing query field")?;
                let plugins = obj
                    .get("plugins")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str())
                            .map(|s| s.to_string())
                            .collect()
                    })
                    .unwrap_or_default();
                Ok(ServiceBridgeMessage::SearchQuery {
                    query: query.to_string(),
                    plugins,
                })
            },
            "permission_check" => {
                let permission_type = obj
                    .get("permission_type")
                    .and_then(|v| v.as_str())
                    .ok_or("Missing permission_type field")?;
                Ok(ServiceBridgeMessage::PermissionCheck {
                    permission_type: permission_type.to_string(),
                })
            },
            "permission_request" => {
                let permission_type = obj
                    .get("permission_type")
                    .and_then(|v| v.as_str())
                    .ok_or("Missing permission_type field")?;
                Ok(ServiceBridgeMessage::PermissionRequest {
                    permission_type: permission_type.to_string(),
                })
            },
            _ => Ok(ServiceBridgeMessage::Unknown {
                message_type: message_event.message_type.clone(),
                payload: serde_json::to_value(&message_event.payload).unwrap_or(Value::Null),
            }),
        },
        _ => Err("Invalid payload format".to_string()),
    }
}

/// Parse headers from message payload
fn parse_headers(payload: &Value) -> Result<HashMap<String, String>, String> {
    let mut headers = HashMap::new();

    if let Some(headers_value) = payload.get("headers") {
        if let Some(headers_obj) = headers_value.as_object() {
            for (key, value) in headers_obj {
                if let Some(value_str) = value.as_str() {
                    headers.insert(key.clone(), value_str.to_string());
                }
            }
        }
    }

    Ok(headers)
}

/// Parse permission type string to ECS PermissionType enum
fn parse_permission_type(permission_str: &str) -> Result<String, String> {
    Ok(permission_str.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clipboard_read_translation() {
        let mut payload = HashMap::new();
        payload.insert("format".to_string(), Value::String("text".to_string()));

        let message_event = PluginMessageEvent {
            from: "test_plugin".to_string(),
            to: "service_bridge".to_string(),
            message_type: "clipboard_read".to_string(),
            payload: Value::Object(serde_json::Map::from_iter(payload)),
            correlation_id: None,
            priority: ecs_service_bridge::events::MessagePriority::Normal,
            plugin_id: "test_plugin".to_string(),
            timestamp: ecs_service_bridge::types::TimeStamp::now(),
            request_id: uuid::Uuid::new_v4().to_string(),
        };

        let operation_id = OperationId::new();
        let result = translate_message_to_ecs_events(&message_event, operation_id);

        assert!(result.is_ok());
        let events = result.unwrap();
        assert_eq!(events.len(), 1);

        match &events[0] {
            EcsEventType::ClipboardRequest(req) => {
                assert_eq!(req.requester, "test_plugin");
            },
            other => {
                assert!(false, "Expected ClipboardRequest event, got: {:?}", other);
            },
        }
    }

    #[test]
    fn test_notification_show_translation() {
        let mut payload = HashMap::new();
        payload.insert("title".to_string(), Value::String("Test Title".to_string()));
        payload.insert(
            "message".to_string(),
            Value::String("Test Message".to_string()),
        );
        payload.insert("duration".to_string(), Value::Number(5.into()));

        let message_event = PluginMessageEvent {
            from: "test_plugin".to_string(),
            to: "service_bridge".to_string(),
            message_type: "notification_show".to_string(),
            payload: serde_json::Value::Object(serde_json::Map::from_iter(payload)),
            correlation_id: None,
            priority: ecs_service_bridge::events::MessagePriority::Normal,
            plugin_id: "test_plugin".to_string(),
            timestamp: ecs_service_bridge::types::TimeStamp::now(),
            request_id: uuid::Uuid::new_v4().to_string(),
        };

        let operation_id = OperationId::new();
        let result = translate_message_to_ecs_events(&message_event, operation_id);

        assert!(result.is_ok());
        let events = result.unwrap();
        assert_eq!(events.len(), 1);

        match &events[0] {
            EcsEventType::NotificationRequest(req) => {
                assert_eq!(req.title, "Test Title");
                assert_eq!(req.message, "Test Message");
                assert_eq!(req.duration, Some(std::time::Duration::from_secs(5)));
            },
            other => {
                assert!(false, "Expected NotificationRequest event, got: {:?}", other);
            },
        }
    }

    #[test]
    fn test_response_translation() {
        let operation_id = OperationId::new();
        let response = EcsEventResponse::ClipboardData {
            data: "test data".to_string(),
            format: "text".to_string(),
        };

        let result = translate_ecs_response_to_message(
            response,
            operation_id.clone(),
            "test_plugin".to_string(),
        );

        assert!(result.is_ok());
        let message = result.unwrap();
        assert_eq!(message.to, "test_plugin");
        assert_eq!(message.message_type, "clipboard_response");
        assert_eq!(message.payload.get("success"), Some(&Value::Bool(true)));
        assert_eq!(
            message.payload.get("data"),
            Some(&Value::String("test data".to_string()))
        );
    }
}
